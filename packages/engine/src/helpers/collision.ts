import { mat4, vec3, vec4 } from 'gl-matrix';
import { getWorldOrThrow, requireInitialized } from '../engine/bridge/guards';
import { getResolvedEntityTransformMatrix } from '../engine/systems/utils';
import {
  create3DEntity,
  create3DTag,
  draw3DGizmoAabb,
  draw3DGizmoLine,
  remove3DEntity,
  set3DParent,
  update3DTransform,
} from '../engine/world/world3d';
import type { EntityId, World3DId } from '../engine/world/types';
import {
  createPointerRayFromEvent,
  intersectRayAabb,
  type PointerEventRaycastInput,
  type Ray3,
  type RayHit,
} from './raycast';

export interface CollisionRelativeTransform {
  position?: [number, number, number];
  rotation?: [number, number, number, number];
  scale?: [number, number, number];
}

export interface CollisionEntityBase {
  kind: 'aabb' | 'sphere' | 'plane';
  worldId: World3DId;
  entityId: EntityId;
  modelEntityId: EntityId;
  relativeTransform: Required<CollisionRelativeTransform>;
}

export interface CollisionAabbEntity extends CollisionEntityBase {
  kind: 'aabb';
  halfExtents: [number, number, number];
  debugGizmoAabb: boolean;
  debugGizmoColor: [number, number, number, number];
}

export interface CollisionAabbWorldBounds {
  min: [number, number, number];
  max: [number, number, number];
}

type CollisionAabbWorldCorners = [
  [number, number, number],
  [number, number, number],
  [number, number, number],
  [number, number, number],
  [number, number, number],
  [number, number, number],
  [number, number, number],
  [number, number, number],
];

export interface CollisionSphereEntity extends CollisionEntityBase {
  kind: 'sphere';
  radius: number;
}

export interface CollisionPlaneEntity extends CollisionEntityBase {
  kind: 'plane';
  normal: [number, number, number];
}

export type CollisionEntity =
  | CollisionAabbEntity
  | CollisionSphereEntity
  | CollisionPlaneEntity;

export interface CollisionAttachBaseArgs {
  worldId: World3DId;
  modelEntityId: EntityId;
  relativeTransform?: CollisionRelativeTransform;
  name?: string;
  labels?: string[];
}

export interface AttachCollisionAabbArgs extends CollisionAttachBaseArgs {
  halfExtents: [number, number, number];
  debugGizmoAabb?: boolean;
  debugGizmoColor?: [number, number, number, number];
}

export interface AttachCollisionSphereArgs extends CollisionAttachBaseArgs {
  radius: number;
}

export interface AttachCollisionPlaneArgs extends CollisionAttachBaseArgs {
  normal?: [number, number, number];
}

function normalizeTransform(
  transform?: CollisionRelativeTransform,
): Required<CollisionRelativeTransform> {
  return {
    position: transform?.position ?? [0, 0, 0],
    rotation: transform?.rotation ?? [0, 0, 0, 1],
    scale: transform?.scale ?? [1, 1, 1],
  };
}

function buildLabels(
  kind: CollisionEntity['kind'],
  labels?: string[],
): string[] {
  const base = [`collision`, `collision:${kind}`];
  if (!labels || labels.length === 0) {
    return base;
  }
  return [...base, ...labels];
}

function attachCollisionEntityBase(
  kind: CollisionEntity['kind'],
  args: CollisionAttachBaseArgs,
): Pick<CollisionEntityBase, 'entityId' | 'relativeTransform'> {
  const entityId = create3DEntity(args.worldId);
  const relativeTransform = normalizeTransform(args.relativeTransform);

  set3DParent(args.worldId, entityId, args.modelEntityId);
  update3DTransform(args.worldId, entityId, relativeTransform);
  create3DTag(args.worldId, entityId, {
    name: args.name ?? `collision:${kind}`,
    labels: buildLabels(kind, args.labels),
  });

  return { entityId, relativeTransform };
}

/**
 * Creates an AABB collider entity parented to a model entity.
 */
export function attachCollisionAabb(
  args: AttachCollisionAabbArgs,
): CollisionAabbEntity {
  const { entityId, relativeTransform } = attachCollisionEntityBase(
    'aabb',
    args,
  );
  return {
    kind: 'aabb',
    worldId: args.worldId,
    entityId,
    modelEntityId: args.modelEntityId,
    relativeTransform,
    halfExtents: args.halfExtents,
    debugGizmoAabb: args.debugGizmoAabb ?? false,
    debugGizmoColor: args.debugGizmoColor ?? [0.2, 0.9, 0.2, 1],
  };
}

/**
 * Emits one frame of debug AABB gizmo for the collision entity.
 *
 * Note:
 * This uses the collision relative transform center and half extents.
 * If parent rotation is significant, this stays axis-aligned.
 */
export function drawCollisionAabbGizmo(collision: CollisionAabbEntity): void {
  if (!collision.debugGizmoAabb) return;
  const corners = getCollisionAabbWorldCorners(collision);
  const edges: [number, number][] = [
    [0, 1],
    [1, 3],
    [3, 2],
    [2, 0],
    [4, 5],
    [5, 7],
    [7, 6],
    [6, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
  ];
  for (let i = 0; i < edges.length; i++) {
    const [a, b] = edges[i]!;
    draw3DGizmoLine(collision.worldId, {
      start: corners[a]!,
      end: corners[b]!,
      color: collision.debugGizmoColor,
    });
  }
}

/**
 * Computes world-space AABB bounds from the collision entity resolved transform.
 *
 * This includes parent constraints (hierarchy) and runtime transform updates.
 */
export function getCollisionAabbWorldBounds(
  collision: CollisionAabbEntity,
): CollisionAabbWorldBounds {
  const corners = getCollisionAabbWorldCorners(collision);
  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let minZ = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;
  let maxZ = Number.NEGATIVE_INFINITY;

  for (let i = 0; i < corners.length; i++) {
    const [x, y, z] = corners[i]!;
    if (x < minX) minX = x;
    if (y < minY) minY = y;
    if (z < minZ) minZ = z;
    if (x > maxX) maxX = x;
    if (y > maxY) maxY = y;
    if (z > maxZ) maxZ = z;
  }

  return {
    min: [minX, minY, minZ],
    max: [maxX, maxY, maxZ],
  };
}

/**
 * Returns world transform matrix for the collision entity (with hierarchy solved).
 */
export function getCollisionWorldTransformMatrix(
  collision: CollisionEntity,
): mat4 {
  requireInitialized();
  const world = getWorldOrThrow(collision.worldId as number);
  return getResolvedEntityTransformMatrix(
    world,
    collision.entityId as number,
  );
}

/**
 * Returns all 8 world-space corners for an AABB collider oriented by entity transform.
 */
export function getCollisionAabbWorldCorners(
  collision: CollisionAabbEntity,
): CollisionAabbWorldCorners {
  const worldTransform = getCollisionWorldTransformMatrix(collision);
  const [hx, hy, hz] = collision.halfExtents;

  const localCorners: [number, number, number][] = [
    [-hx, -hy, -hz],
    [-hx, -hy, hz],
    [-hx, hy, -hz],
    [-hx, hy, hz],
    [hx, -hy, -hz],
    [hx, -hy, hz],
    [hx, hy, -hz],
    [hx, hy, hz],
  ];

  const corners: [number, number, number][] = [];
  for (let i = 0; i < localCorners.length; i++) {
    const [x, y, z] = localCorners[i]!;
    const worldCorner = vec4.transformMat4(
      vec4.create(),
      vec4.fromValues(x, y, z, 1),
      worldTransform as mat4,
    );
    corners.push([worldCorner[0]!, worldCorner[1]!, worldCorner[2]!]);
  }
  return corners as CollisionAabbWorldCorners;
}

/**
 * Raycasts an oriented AABB collider (local AABB transformed by entity world matrix).
 */
export function raycastCollisionAabb(
  ray: Ray3,
  collision: CollisionAabbEntity,
  localPadding = 0,
): RayHit | null {
  const world = getCollisionWorldTransformMatrix(collision);
  const inverse = mat4.invert(mat4.create(), world);
  if (!inverse) return null;

  const localOrigin4 = vec4.transformMat4(
    vec4.create(),
    vec4.fromValues(ray.origin[0]!, ray.origin[1]!, ray.origin[2]!, 1),
    inverse,
  );
  const localDirection4 = vec4.transformMat4(
    vec4.create(),
    vec4.fromValues(ray.direction[0]!, ray.direction[1]!, ray.direction[2]!, 0),
    inverse,
  );
  const localDirection = vec3.normalize(
    vec3.create(),
    vec3.fromValues(localDirection4[0]!, localDirection4[1]!, localDirection4[2]!),
  );
  const localRay: Ray3 = {
    origin: vec3.fromValues(
      localOrigin4[0]!,
      localOrigin4[1]!,
      localOrigin4[2]!,
    ),
    direction: localDirection,
  };

  const [hx, hy, hz] = collision.halfExtents;
  const px = Math.max(0, localPadding);
  const py = Math.max(0, localPadding);
  const pz = Math.max(0, localPadding);
  const localHit = intersectRayAabb(
    localRay,
    [-hx - px, -hy - py, -hz - pz],
    [hx + px, hy + py, hz + pz],
  );
  if (!localHit) return null;

  const worldPoint4 = vec4.transformMat4(
    vec4.create(),
    vec4.fromValues(
      localHit.point[0]!,
      localHit.point[1]!,
      localHit.point[2]!,
      1,
    ),
    world,
  );
  const worldPoint = vec3.fromValues(
    worldPoint4[0]!,
    worldPoint4[1]!,
    worldPoint4[2]!,
  );

  return {
    distance: vec3.distance(worldPoint, ray.origin),
    point: worldPoint,
  };
}

export interface PointerCollisionAabbInput extends PointerEventRaycastInput {
  collision: CollisionAabbEntity;
  /**
   * Optional border tolerance in screen pixels (default: 0.5).
   * Helps avoid subpixel edge misses where the rendered cube appears hit,
   * but strict math falls just outside due precision/rasterization boundaries.
   */
  edgePaddingPixels?: number;
}

function resolvePointerViewportSize(
  input: PointerEventRaycastInput,
): [number, number] | null {
  const event = input.pointerEvent;
  if (
    event.positionTarget &&
    typeof event.targetWidth === 'number' &&
    typeof event.targetHeight === 'number' &&
    event.targetWidth > 0 &&
    event.targetHeight > 0
  ) {
    return [event.targetWidth, event.targetHeight];
  }
  if (
    typeof event.windowWidth === 'number' &&
    typeof event.windowHeight === 'number' &&
    event.windowWidth > 0 &&
    event.windowHeight > 0
  ) {
    return [event.windowWidth, event.windowHeight];
  }
  if (input.fallbackViewportSize) {
    const w = input.fallbackViewportSize[0] ?? 0;
    const h = input.fallbackViewportSize[1] ?? 0;
    if (w > 0 && h > 0) return [w, h];
  }
  return null;
}

function computeLocalEdgePadding(
  input: PointerCollisionAabbInput,
  ray: Ray3,
): number {
  const viewport = resolvePointerViewportSize(input);
  if (!viewport) return 0;
  const [, viewportHeight] = viewport;
  if (viewportHeight <= 0) return 0;

  const projection = input.projectionMatrix;
  const focalY = Math.abs(projection[5] ?? 0);
  if (focalY <= 1e-8) return 0;

  const world = getCollisionWorldTransformMatrix(input.collision);
  const center = vec3.fromValues(world[12]!, world[13]!, world[14]!);
  const depth = vec3.distance(center, ray.origin);
  if (!Number.isFinite(depth) || depth <= 0) return 0;

  const pixelPadding = Math.max(0, input.edgePaddingPixels ?? 0.5);
  const worldPadding = ((2 * depth) / (focalY * viewportHeight)) * pixelPadding;
  if (!Number.isFinite(worldPadding) || worldPadding <= 0) return 0;

  const sx = Math.hypot(world[0]!, world[1]!, world[2]!);
  const sy = Math.hypot(world[4]!, world[5]!, world[6]!);
  const sz = Math.hypot(world[8]!, world[9]!, world[10]!);
  const minScale = Math.max(1e-8, Math.min(sx, sy, sz));

  return worldPadding / minScale;
}

/**
 * Raycasts pointer event coordinates against an AABB collision entity.
 *
 * This uses target/window dimensions carried by pointer events to normalize
 * coordinates with the real drawn area.
 */
export function raycastPointerCollisionAabb(
  input: PointerCollisionAabbInput,
): RayHit | null {
  const ray = createPointerRayFromEvent(input);
  if (!ray) return null;
  const localPadding = computeLocalEdgePadding(input, ray);
  return raycastCollisionAabb(ray, input.collision, localPadding);
}

/**
 * Fast boolean check for pointer collision against an AABB entity.
 */
export function isPointerCollidingAabb(
  input: PointerCollisionAabbInput,
): boolean {
  return raycastPointerCollisionAabb(input) !== null;
}

/**
 * Creates a sphere collider entity parented to a model entity.
 */
export function attachCollisionSphere(
  args: AttachCollisionSphereArgs,
): CollisionSphereEntity {
  const { entityId, relativeTransform } = attachCollisionEntityBase(
    'sphere',
    args,
  );
  return {
    kind: 'sphere',
    worldId: args.worldId,
    entityId,
    modelEntityId: args.modelEntityId,
    relativeTransform,
    radius: args.radius,
  };
}

/**
 * Creates a plane collider entity parented to a model entity.
 */
export function attachCollisionPlane(
  args: AttachCollisionPlaneArgs,
): CollisionPlaneEntity {
  const { entityId, relativeTransform } = attachCollisionEntityBase(
    'plane',
    args,
  );
  return {
    kind: 'plane',
    worldId: args.worldId,
    entityId,
    modelEntityId: args.modelEntityId,
    relativeTransform,
    normal: args.normal ?? [0, 1, 0],
  };
}

/**
 * Updates only the relative transform of an attached collision entity.
 */
export function updateCollisionRelativeTransform(
  collision: CollisionEntity,
  relativeTransform: CollisionRelativeTransform,
): void {
  const normalized = normalizeTransform(relativeTransform);
  update3DTransform(collision.worldId, collision.entityId, normalized);
  collision.relativeTransform = normalized;
}

/**
 * Removes an attached collision entity.
 */
export function disposeCollisionEntity(collision: CollisionEntity): void {
  remove3DEntity(collision.worldId, collision.entityId);
}
