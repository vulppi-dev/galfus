import { quat, vec3, vec4 } from 'gl-matrix';
import { EngineError } from '../../errors';
import { getWorldOrThrow, requireInitialized } from '../../bridge/guards';
import { engineState } from '../../state';
import type {
  CameraProps,
  GeometryProps,
  LightProps,
  MaterialProps,
  ModelProps,
  TagProps,
  TextureProps,
  TransformComponent,
  TransformProps
} from '../../ecs';
import { allocateGlobalId } from './common';
import { emitIntent } from './intents';

function hasTransformPatch(props: TransformProps): boolean {
  return (
    props.position !== undefined ||
    props.rotation !== undefined ||
    props.scale !== undefined ||
    props.layerMask !== undefined ||
    props.visible !== undefined
  );
}

/** Draws a debug gizmo line for one frame. */
export function drawGizmoLine(
  worldId: number,
  props: {
    start: vec3;
    end: vec3;
    color?: vec4;
    thickness?: number;
  }
): void {
  emitIntent(worldId, {
    type: 'gizmo-draw-line',
    start: props.start,
    end: props.end,
    color: props.color || vec4.fromValues(1, 1, 1, 1),
    thickness: props.thickness
  });
}

/** Draws a debug gizmo AABB for one frame. */
export function drawGizmoAabb(
  worldId: number,
  props: {
    min: vec3;
    max: vec3;
    color?: vec4;
    thickness?: number;
  }
): void {
  emitIntent(worldId, {
    type: 'gizmo-draw-aabb',
    min: props.min,
    max: props.max,
    color: props.color || vec4.fromValues(1, 1, 1, 1),
    thickness: props.thickness
  });
}

/** Draws a debug polyline gizmo for one frame. */
export function drawGizmoPolyline(
  worldId: number,
  props: {
    points: vec3[];
    color?: vec4;
    closed?: boolean;
    thickness?: number;
  }
): void {
  emitIntent(worldId, {
    type: 'gizmo-draw-polyline',
    points: props.points,
    color: props.color || vec4.fromValues(1, 1, 1, 1),
    closed: props.closed,
    thickness: props.thickness
  });
}

/**
 * Creates a new entity in the specified world.
 * Returns the entity ID immediately, but the entity is actually created in the next tick.
 */
export function createEntity(worldId: number): number {
  requireInitialized();
  const entityId = engineState.nextEntityId++;

  emitIntent(worldId, {
    type: 'create-entity',
    worldId,
    entityId
  });

  return entityId;
}

/** Removes an entity and all its components in the next tick. */
export function removeEntity(worldId: number, entityId: number): void {
  emitIntent(worldId, {
    type: 'remove-entity',
    entityId
  });
}

/** Attaches a camera component to an entity via Intent. */
export function createCamera(worldId: number, entityId: number, props: CameraProps = {}): void {
  emitIntent(worldId, {
    type: 'attach-camera',
    entityId,
    props
  });
}

/** Attaches a light component to an entity via Intent. */
export function createLight(worldId: number, entityId: number, props: LightProps = {}): void {
  emitIntent(worldId, {
    type: 'attach-light',
    entityId,
    props
  });
}

/** Attaches a model component to an entity via Intent. */
export function createModel(worldId: number, entityId: number, props: ModelProps): void {
  emitIntent(worldId, {
    type: 'attach-model',
    entityId,
    props
  });
}

/** Updates an entity's transform via Intent. */
export function updateTransform(worldId: number, entityId: number, props: TransformProps): void {
  if (!hasTransformPatch(props)) {
    return;
  }

  // Apply immediately to local ECS snapshot so same-frame queries
  // (for example gizmo/collision helpers) observe latest transform.
  // Intent is still emitted to keep the standard system pipeline authoritative.
  const world = getWorldOrThrow(worldId);
  const store = world.components.get(entityId);
  const transform = store?.get('Transform') as TransformComponent | undefined;
  if (transform) {
    if (props.position) {
      transform.position = vec3.fromValues(
        props.position[0] ?? transform.position[0],
        props.position[1] ?? transform.position[1],
        props.position[2] ?? transform.position[2]
      );
    }
    if (props.rotation) {
      transform.rotation = quat.fromValues(
        props.rotation[0] ?? transform.rotation[0],
        props.rotation[1] ?? transform.rotation[1],
        props.rotation[2] ?? transform.rotation[2],
        props.rotation[3] ?? transform.rotation[3]
      );
    }
    if (props.scale) {
      transform.scale = vec3.fromValues(
        props.scale[0] ?? transform.scale[0],
        props.scale[1] ?? transform.scale[1],
        props.scale[2] ?? transform.scale[2]
      );
    }
    if (props.layerMask !== undefined) {
      transform.layerMask = props.layerMask;
    }
    if (props.visible !== undefined) {
      transform.visible = props.visible;
    }
    world.constraintDirtyEntities.add(entityId);
  }

  emitIntent(worldId, {
    type: 'update-transform',
    entityId,
    props
  });
}

/** Attaches a tag component to an entity via Intent. */
export function createTag(worldId: number, entityId: number, props: TagProps): void {
  emitIntent(worldId, {
    type: 'attach-tag',
    entityId,
    props
  });
}

/** Sets the parent of an entity via Intent. */
export function setParent(worldId: number, entityId: number, parentId: number | null): void {
  if (parentId !== null && parentId === entityId) {
    throw new EngineError('InvalidParent', `Entity ${entityId} cannot be parent of itself.`);
  }
  emitIntent(worldId, {
    type: 'set-parent',
    entityId,
    parentId
  });
}

/** Creates a material resource and returns its ID. */
export function createMaterial(worldId: number, props: MaterialProps): number {
  requireInitialized();
  getWorldOrThrow(worldId);
  const resourceId = allocateGlobalId();

  emitIntent(worldId, {
    type: 'create-material',
    resourceId,
    props
  });

  return resourceId;
}

/** Creates a geometry resource and returns its ID. */
export function createGeometry(worldId: number, props: GeometryProps): number {
  requireInitialized();
  getWorldOrThrow(worldId);
  const resourceId = allocateGlobalId();

  emitIntent(worldId, {
    type: 'create-geometry',
    resourceId,
    props
  });

  return resourceId;
}

/** Creates a texture resource and returns its ID. */
export function createTexture(worldId: number, props: TextureProps): number {
  requireInitialized();
  getWorldOrThrow(worldId);
  const resourceId = allocateGlobalId();

  emitIntent(worldId, {
    type: 'create-texture',
    resourceId,
    props
  });

  return resourceId;
}
