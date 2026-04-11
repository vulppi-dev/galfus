import { Primitive, type Accessor, type Material, type Texture } from '@gltf-transform/core';
import {
  create3DGeometry,
  create3DMaterial,
  create3DTexture,
  type GeometryId,
  type MaterialId,
  type TextureId
} from '@vulfram/engine/world3d';
import { vec4 } from '@vulfram/engine/math';
import type { GeometryPrimitiveEntry } from '@vulfram/engine/types';
import { uploadBytes } from './context';
import {
  accessorToIndexBytes,
  accessorToStreamBytes,
  samplerFromTextureInfo,
  semanticToPrimitiveType,
  toVec4
} from './convert';
import type { LoaderContext } from './types';

function uploadVertexAccessor(ctx: LoaderContext, accessor: Accessor, semantic: string): number {
  const cached = ctx.uploadedVertexByAccessor.get(accessor);
  if (cached !== undefined) return cached;

  const bytes = accessorToStreamBytes(accessor, semantic);
  const bufferId = uploadBytes(ctx, 'vertex-data', bytes);
  ctx.uploadedVertexByAccessor.set(accessor, bufferId);
  return bufferId;
}

function uploadIndexAccessor(ctx: LoaderContext, accessor: Accessor): number {
  const cached = ctx.uploadedIndexByAccessor.get(accessor);
  if (cached !== undefined) return cached;

  const bytes = accessorToIndexBytes(accessor);
  const bufferId = uploadBytes(ctx, 'index-data', bytes);
  ctx.uploadedIndexByAccessor.set(accessor, bufferId);
  return bufferId;
}

/** Ensures a core texture exists for source texture. */
export function ensureTexture(
  ctx: LoaderContext,
  texture: Texture,
  srgb: boolean
): TextureId | null {
  const existing = ctx.textureBySource.get(texture);
  if (existing !== undefined) {
    const previous = ctx.textureColorSpaceHint.get(texture);
    if (previous !== undefined && previous !== srgb) {
      ctx.warnings.push(
        `Texture "${texture.getName() || 'unnamed'}" reused with mixed color-space expectations; keeping first value (srgb=${previous}).`
      );
    }
    return existing;
  }

  const image = texture.getImage();
  if (!image) {
    ctx.warnings.push(
      `Texture "${texture.getName() || 'unnamed'}" has no image bytes and was skipped.`
    );
    return null;
  }

  const bufferId = uploadBytes(ctx, 'image-data', image);
  const textureId = create3DTexture(ctx.worldId, {
    label: `${ctx.labelPrefix}:tex:${texture.getName() || 'unnamed'}`,
    source: { type: 'buffer', bufferId },
    srgb,
    mode: 'standalone'
  });

  ctx.textureBySource.set(texture, textureId);
  ctx.textureColorSpaceHint.set(texture, srgb);
  ctx.createdTextureIds.add(textureId);
  ctx.counters.textures += 1;
  return textureId;
}

function alphaModeToSurfaceType(alphaMode: string): 'opaque' | 'transparent' | 'masked' {
  if (alphaMode === 'BLEND') return 'transparent';
  if (alphaMode === 'MASK') return 'masked';
  return 'opaque';
}

function ensureDefaultMaterial(ctx: LoaderContext): MaterialId {
  if (ctx.defaultMaterialId !== undefined) return ctx.defaultMaterialId;

  if (ctx.materialMode === 'standard') {
    const id = create3DMaterial(ctx.worldId, {
      kind: 'standard',
      label: `${ctx.labelPrefix}:mat:default`,
      options: {
        type: 'standard',
        content: {
          baseColor: vec4.fromValues(1, 1, 1, 1),
          surfaceType: 'opaque',
          flags: 0
        }
      }
    });

    ctx.defaultMaterialId = id;
    ctx.createdMaterialIds.add(id);
    ctx.counters.materials += 1;
    return id;
  }

  const id = create3DMaterial(ctx.worldId, {
    kind: 'pbr',
    label: `${ctx.labelPrefix}:mat:default`,
    options: {
      type: 'pbr',
      content: {
        baseColor: vec4.fromValues(1, 1, 1, 1),
        surfaceType: 'opaque',
        emissiveColor: vec4.fromValues(0, 0, 0, 1),
        metallic: 0,
        roughness: 1,
        ao: 1,
        normalScale: 1,
        flags: 0
      }
    }
  });

  ctx.defaultMaterialId = id;
  ctx.createdMaterialIds.add(id);
  ctx.counters.materials += 1;
  return id;
}

/** Ensures a core material exists for source material. */
export function ensureMaterial(ctx: LoaderContext, material: Material | null): MaterialId {
  if (!material) return ensureDefaultMaterial(ctx);

  const existing = ctx.materialBySource.get(material);
  if (existing !== undefined) return existing;

  const baseTexture = material.getBaseColorTexture();
  const normalTexture = material.getNormalTexture();
  const mrTexture = material.getMetallicRoughnessTexture();
  const emissiveTexture = material.getEmissiveTexture();
  const aoTexture = material.getOcclusionTexture();

  const baseTexId = baseTexture ? ensureTexture(ctx, baseTexture, true) : null;
  const normalTexId = normalTexture ? ensureTexture(ctx, normalTexture, false) : null;
  const mrTexId = mrTexture ? ensureTexture(ctx, mrTexture, false) : null;
  const emissiveTexId = emissiveTexture ? ensureTexture(ctx, emissiveTexture, true) : null;
  const aoTexId = aoTexture ? ensureTexture(ctx, aoTexture, false) : null;

  const baseSampler = samplerFromTextureInfo(material.getBaseColorTextureInfo());
  const normalSampler = samplerFromTextureInfo(material.getNormalTextureInfo());
  const mrSampler = samplerFromTextureInfo(material.getMetallicRoughnessTextureInfo());
  const emissiveSampler = samplerFromTextureInfo(material.getEmissiveTextureInfo());
  const aoSampler = samplerFromTextureInfo(material.getOcclusionTextureInfo());
  const emissiveFactor = material.getEmissiveFactor();

  const materialId = create3DMaterial(ctx.worldId, {
    kind: ctx.materialMode === 'pbr' ? 'pbr' : 'standard',
    label: `${ctx.labelPrefix}:mat:${material.getName() || 'unnamed'}`,
    options: {
      ...(ctx.materialMode === 'pbr'
        ? {
            type: 'pbr' as const,
            content: {
              baseColor: toVec4(material.getBaseColorFactor(), 1),
              surfaceType: alphaModeToSurfaceType(material.getAlphaMode()),
              emissiveColor: vec4.fromValues(
                emissiveFactor[0] ?? 0,
                emissiveFactor[1] ?? 0,
                emissiveFactor[2] ?? 0,
                1
              ),
              metallic: material.getMetallicFactor(),
              roughness: material.getRoughnessFactor(),
              ao: material.getOcclusionStrength(),
              normalScale: material.getNormalScale(),
              baseTexId,
              baseSampler,
              normalTexId,
              normalSampler,
              metallicRoughnessTexId: mrTexId,
              metallicRoughnessSampler: mrSampler,
              emissiveTexId,
              emissiveSampler,
              aoTexId,
              aoSampler,
              flags: material.getDoubleSided() ? 1 : 0
            }
          }
        : {
            type: 'standard' as const,
            content: {
              baseColor: toVec4(material.getBaseColorFactor(), 1),
              surfaceType: alphaModeToSurfaceType(material.getAlphaMode()),
              specColor: vec4.fromValues(
                material.getMetallicFactor(),
                material.getRoughnessFactor(),
                material.getOcclusionStrength(),
                1
              ),
              specPower: 32,
              baseTexId,
              baseSampler,
              normalTexId,
              normalSampler,
              specTexId: mrTexId,
              specSampler: mrSampler,
              flags: material.getDoubleSided() ? 1 : 0
            }
          })
    }
  });

  ctx.materialBySource.set(material, materialId);
  ctx.createdMaterialIds.add(materialId);
  ctx.counters.materials += 1;
  return materialId;
}

/** Ensures a core geometry exists for source primitive. */
export function ensurePrimitiveGeometry(
  ctx: LoaderContext,
  primitive: Primitive
): GeometryId | null {
  const existing = ctx.geometryByPrimitive.get(primitive);
  if (existing !== undefined) return existing;

  if (primitive.getMode() !== Primitive.Mode.TRIANGLES) {
    ctx.warnings.push(
      `Primitive mode ${primitive.getMode()} is not supported in v1 loader (triangles only). Primitive skipped.`
    );
    return null;
  }

  const entries: GeometryPrimitiveEntry[] = [];
  const indexAccessor = primitive.getIndices();
  if (indexAccessor) {
    entries.push({
      primitiveType: 'index',
      bufferId: uploadIndexAccessor(ctx, indexAccessor)
    });
  }

  const orderedSemantics = [
    'POSITION',
    'NORMAL',
    'TANGENT',
    'COLOR_0',
    'TEXCOORD_0',
    'TEXCOORD_1'
  ] as const;

  let hasPosition = false;

  for (const semantic of orderedSemantics) {
    const accessor = primitive.getAttribute(semantic);
    if (!accessor) continue;

    const primitiveType = semanticToPrimitiveType(semantic);
    if (!primitiveType) {
      ctx.warnings.push(`Unsupported semantic "${semantic}" ignored.`);
      continue;
    }

    if (semantic === 'POSITION') hasPosition = true;

    entries.push({
      primitiveType,
      bufferId: uploadVertexAccessor(ctx, accessor, semantic)
    });
  }

  if (!hasPosition) {
    ctx.warnings.push('Primitive without POSITION stream skipped.');
    return null;
  }

  const geometryId = create3DGeometry(ctx.worldId, {
    type: 'custom',
    label: `${ctx.labelPrefix}:geo:${primitive.getName() || 'unnamed'}`,
    entries
  });

  ctx.geometryByPrimitive.set(primitive, geometryId);
  ctx.createdGeometryIds.add(geometryId);
  ctx.counters.geometries += 1;
  return geometryId;
}
