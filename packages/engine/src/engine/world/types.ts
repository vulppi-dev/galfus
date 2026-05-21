/**
 * Generic branded numeric IDs used by the public API to avoid mixing domains.
 *
 * Example:
 * `type WindowId = number & { __WINDOW_ID__: null }`
 */
export type WorldId = number & { __WORLD_ID__: null };
export type World3DId = WorldId & { __WORLD_3D_ID__: null };
export type WindowId = number & { __WINDOW_ID__: null };
export type TargetId = number & { __TARGET_ID__: null };
export type EntityId = number & { __ENTITY_ID__: null };
export type MaterialId = number & { __MATERIAL_ID__: null };
export type GeometryId = number & { __GEOMETRY_ID__: null };
export type TextureId = number & { __TEXTURE_ID__: null };
export type CommandId = number & { __COMMAND_ID__: null };

/** Casts a numeric value to a branded world id. */
export function asWorldId(value: number): WorldId {
  return value as WorldId;
}

/** Casts a numeric value to a branded 3D world id. */
export function asWorld3DId(value: number): World3DId {
  return value as World3DId;
}

/** Casts a numeric value to a branded window id. */
export function asWindowId(value: number): WindowId {
  return value as WindowId;
}

/** Casts a numeric value to a branded target id. */
export function asTargetId(value: number): TargetId {
  return value as TargetId;
}

/** Casts a numeric value to a branded entity id. */
export function asEntityId(value: number): EntityId {
  return value as EntityId;
}

/** Casts a numeric value to a branded material id. */
export function asMaterialId(value: number): MaterialId {
  return value as MaterialId;
}

/** Casts a numeric value to a branded geometry id. */
export function asGeometryId(value: number): GeometryId {
  return value as GeometryId;
}

/** Casts a numeric value to a branded texture id. */
export function asTextureId(value: number): TextureId {
  return value as TextureId;
}

/** Casts a numeric value to a branded command id. */
export function asCommandId(value: number): CommandId {
  return value as CommandId;
}

/** Returns raw numeric world id from branded world id variants. */
export function asWorldNumber(worldId: WorldId | World3DId | number): number {
  return worldId as number;
}
