import type { EnvironmentConfig } from '../../types/cmds/environment';
import type { ShadowConfig } from '../../types/cmds/shadow';
import { enqueueCommand } from '../bridge/dispatch';
import type { System } from '../ecs';
import { toVec3, toVec4 } from './utils';

const WORLD_LIFECYCLE_INTENT_TYPES = [
  'configure-environment',
  'configure-shadows',
  'send-notification',
] as const;

/**
 * Applies world-scoped lifecycle/configuration intents.
 *
 * Covered intents:
 * - environment configuration
 * - shadow configuration
 * - notification dispatch
 */
export const WorldLifecycleSystem: System = (world, context) => {
  const intents = world.intentStore.takeMany(WORLD_LIFECYCLE_INTENT_TYPES);

  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (!intent) continue;

    if (intent.type === 'configure-environment') {
      const config = intent.config as EnvironmentConfig;
      const payload: EnvironmentConfig = {
        msaa: {
          enabled: config.msaa.enabled,
          sampleCount: config.msaa.sampleCount,
        },
        skybox: {
          mode: config.skybox.mode,
          intensity: config.skybox.intensity,
          rotation: config.skybox.rotation,
          groundColor: toVec3(config.skybox.groundColor),
          horizonColor: toVec3(config.skybox.horizonColor),
          skyColor: toVec3(config.skybox.skyColor),
          cubemapTextureId: config.skybox.cubemapTextureId ?? null,
        },
        clearColor: toVec4(config.clearColor ?? [0, 0, 0, 0]),
        post: {
          filterEnabled: config.post.filterEnabled,
          filterExposure: config.post.filterExposure,
          filterGamma: config.post.filterGamma,
          filterSaturation: config.post.filterSaturation,
          filterContrast: config.post.filterContrast,
          filterVignette: config.post.filterVignette,
          filterGrain: config.post.filterGrain,
          filterChromaticAberration: config.post.filterChromaticAberration,
          filterBlur: config.post.filterBlur,
          filterSharpen: config.post.filterSharpen,
          filterTonemapMode: config.post.filterTonemapMode,
          outlineEnabled: config.post.outlineEnabled,
          outlineStrength: config.post.outlineStrength,
          outlineThreshold: config.post.outlineThreshold,
          outlineWidth: config.post.outlineWidth,
          outlineQuality: config.post.outlineQuality,
          filterPosterizeSteps: config.post.filterPosterizeSteps,
          cellShading: config.post.cellShading,
          ssaoEnabled: config.post.ssaoEnabled,
          ssaoStrength: config.post.ssaoStrength,
          ssaoRadius: config.post.ssaoRadius,
          ssaoBias: config.post.ssaoBias,
          ssaoPower: config.post.ssaoPower,
          ssaoBlurRadius: config.post.ssaoBlurRadius,
          ssaoBlurDepthThreshold: config.post.ssaoBlurDepthThreshold,
          bloomEnabled: config.post.bloomEnabled,
          bloomThreshold: config.post.bloomThreshold,
          bloomKnee: config.post.bloomKnee,
          bloomIntensity: config.post.bloomIntensity,
          bloomScatter: config.post.bloomScatter,
        },
      };
      enqueueCommand(context.worldId, 'cmd-environment-upsert', {
        environmentId: context.worldId,
        config: payload,
      });
      // Keep current realm->target bindings synchronized with this environment.
      for (const binding of world.targetLayerBindings.values()) {
        binding.environmentId = context.worldId;
        if (world.coreRealmId === undefined) continue;
        enqueueCommand(context.worldId, 'cmd-target-layer-upsert', {
          realmId: world.coreRealmId,
          targetId: binding.targetId,
          layout: binding.layout,
          cameraId: binding.cameraId,
          environmentId: context.worldId,
        });
      }
    } else if (intent.type === 'configure-shadows') {
      let windowId = world.primaryWindowId;
      if (windowId === undefined) {
        for (const boundWindowId of world.targetWindowBindings.values()) {
          windowId = boundWindowId;
          break;
        }
      }
      if (windowId === undefined) continue;
      const c = intent.config || {};
      const config: ShadowConfig = {
        tileResolution: c.tileResolution ?? 1024,
        atlasTilesW: c.atlasTilesW ?? 8,
        atlasTilesH: c.atlasTilesH ?? 8,
        atlasLayers: c.atlasLayers ?? 2,
        virtualGridSize: c.virtualGridSize ?? 1,
        smoothing: c.smoothing ?? 2,
        normalBias: c.normalBias ?? 0.01,
      };
      enqueueCommand(context.worldId, 'cmd-shadow-configure', {
        windowId,
        config,
      });
    } else if (intent.type === 'send-notification') {
      enqueueCommand(context.worldId, 'cmd-notification-send', {
        id: `notif-${Date.now()}`,
        level: intent.level,
        title: intent.title,
        body: intent.message,
        timeout: 5000,
      });
    }
  }
};
