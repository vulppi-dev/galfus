import type { TextureCreateMode } from '../../types/kinds';
import type {
  PrimitiveOptions,
} from '../../types/cmds/geometry';
import { enqueueCommand } from '../bridge/dispatch';
import type { GeometryProps, System } from '../ecs';
import { normalizeMaterialOptions, normalizePrimitiveOptions } from './utils';

const RESOURCE_INTENT_TYPES = [
  'create-material',
  'dispose-material',
  'create-geometry',
  'dispose-geometry',
  'create-texture',
  'dispose-texture',
] as const;

function buildPrimitiveOptions(
  primitiveProps: Extract<GeometryProps, { type: 'primitive' }>,
): PrimitiveOptions {
  const { shape, options } = primitiveProps;
  if (shape === 'cube') {
    return {
      type: 'cube',
      content: {
        size: options?.size ?? [1.0, 1.0, 1.0],
        subdivisions: 1,
      },
    };
  }
  if (shape === 'plane') {
    return {
      type: 'plane',
      content: {
        size: options?.size ?? [1.0, 1.0, 1.0],
        subdivisions: options?.subdivisions ?? 1,
      },
    };
  }
  if (shape === 'sphere') {
    return {
      type: 'sphere',
      content: {
        radius: options?.radius ?? 0.5,
        sectors: options?.sectors ?? 32,
        stacks: options?.stacks ?? 16,
      },
    };
  }
  if (shape === 'cylinder') {
    return {
      type: 'cylinder',
      content: {
        radius: options?.radius ?? 0.5,
        height: options?.height ?? 1.0,
        sectors: options?.sectors ?? options?.segments ?? 32,
      },
    };
  }
  if (shape === 'torus') {
    return {
      type: 'torus',
      content: {
        majorRadius: options?.majorRadius ?? 0.5,
        minorRadius: options?.minorRadius ?? 0.25,
        majorSegments: options?.majorSegments ?? options?.radialSegments ?? 32,
        minorSegments: options?.minorSegments ?? options?.tubularSegments ?? 16,
      },
    };
  }
  if (shape === 'pill') {
    return {
      type: 'pill',
      content: {
        radius: options?.radius ?? 0.25,
        height: options?.height ?? 0.5,
        sectors: options?.sectors ?? 32,
        stacks: options?.stacks ?? 8,
      },
    };
  }
  return {
    type: 'pyramid',
    content: {
      size: options?.size ?? [1.0, 1.0, 1.0],
      subdivisions: options?.subdivisions ?? 1,
    },
  };
}

/**
 * Processes resource intents and emits corresponding core resource commands.
 *
 * Covered resources:
 * - material
 * - geometry (custom and primitive)
 * - texture (from buffer or solid color)
 */
export const ResourceUploadSystem: System = (world, context) => {
  const intents = world.intentStore.takeMany(RESOURCE_INTENT_TYPES);

  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (!intent) continue;

    if (intent.type === 'create-material') {
      const options = normalizeMaterialOptions(intent.props.options) || {
        type: 'standard',
        content: {
          baseColor: [1, 1, 1, 1],
          surfaceType: 'opaque',
          flags: 0,
        },
      };

      enqueueCommand(context.worldId, 'cmd-material-upsert', {
        materialId: intent.resourceId,
        label: intent.props.label,
        kind: intent.props.kind ?? 'standard',
        options: options,
      });
    } else if (intent.type === 'dispose-material') {
      enqueueCommand(context.worldId, 'cmd-material-dispose', {
        materialId: intent.resourceId,
      });
    } else if (intent.type === 'create-geometry') {
      if (intent.props.type === 'primitive') {
        const options = normalizePrimitiveOptions(
          buildPrimitiveOptions(intent.props),
        );

        enqueueCommand(context.worldId, 'cmd-primitive-geometry-create', {
          geometryId: intent.resourceId,
          label: intent.props.label,
          shape: intent.props.shape,
          options: options,
        });
      } else {
        enqueueCommand(context.worldId, 'cmd-geometry-upsert', {
          geometryId: intent.resourceId,
          label: intent.props.label,
          entries: intent.props.entries,
        });
      }
    } else if (intent.type === 'dispose-geometry') {
      enqueueCommand(context.worldId, 'cmd-geometry-dispose', {
        geometryId: intent.resourceId,
      });
    } else if (intent.type === 'create-texture') {
      if (intent.props.source.type === 'color') {
        const cmd: {
          textureId: number;
          label?: string;
          color: [number, number, number, number];
          srgb?: boolean;
          mode?: TextureCreateMode;
          atlasOptions?: { tilePx?: number; layers?: number };
        } = {
          textureId: intent.resourceId,
          label: intent.props.label,
          color: Array.from(intent.props.source.color) as [
            number,
            number,
            number,
            number,
          ],
          srgb: intent.props.srgb,
        };
        if (intent.props.mode !== undefined) {
          cmd.mode = intent.props.mode;
        }
        if (intent.props.atlasOptions !== undefined) {
          cmd.atlasOptions = intent.props.atlasOptions;
        }
        enqueueCommand(context.worldId, 'cmd-texture-create-solid-color', cmd);
      } else {
        const cmd: {
          textureId: number;
          label?: string;
          bufferId: number;
          srgb?: boolean;
          mode?: TextureCreateMode;
          atlasOptions?: { tilePx?: number; layers?: number };
        } = {
          textureId: intent.resourceId,
          label: intent.props.label,
          bufferId: intent.props.source.bufferId,
          srgb: intent.props.srgb,
        };
        if (intent.props.mode !== undefined) {
          cmd.mode = intent.props.mode;
        }
        if (intent.props.atlasOptions !== undefined) {
          cmd.atlasOptions = intent.props.atlasOptions;
        }
        enqueueCommand(context.worldId, 'cmd-texture-create-from-buffer', cmd);
      }
    } else if (intent.type === 'dispose-texture') {
      enqueueCommand(context.worldId, 'cmd-texture-dispose', {
        textureId: intent.resourceId,
      });
    }
  }
};
