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

function normalizeEnvironmentConfig(config: EnvironmentConfig): EnvironmentConfig {
  const payload: EnvironmentConfig = {};

  if (config.msaa) {
    payload.msaa = {
      enabled: config.msaa.enabled,
      sampleCount: config.msaa.sampleCount,
    };
  }

  if (config.skybox) {
    payload.skybox = {
      mode: config.skybox.mode,
      intensity: config.skybox.intensity,
      rotation: config.skybox.rotation,
      groundColor: config.skybox.groundColor
        ? toVec3(config.skybox.groundColor)
        : undefined,
      horizonColor: config.skybox.horizonColor
        ? toVec3(config.skybox.horizonColor)
        : undefined,
      skyColor: config.skybox.skyColor ? toVec3(config.skybox.skyColor) : undefined,
      horizonGroundThreshold: config.skybox.horizonGroundThreshold,
      horizonSkyThreshold: config.skybox.horizonSkyThreshold,
      directionalLights: config.skybox.directionalLights?.map((light) => ({
        lightId: light.lightId,
        solidSize: light.solidSize,
        gradientSize: light.gradientSize,
      })),
      cubemapTextureId: config.skybox.cubemapTextureId,
    };
  }

  if (config.clearColor) {
    payload.clearColor = toVec4(config.clearColor);
  }

  if (config.post) {
    payload.post = {
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
    };
  }

  return payload;
}

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
      const payload = normalizeEnvironmentConfig(intent.config as EnvironmentConfig);
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
      enqueueCommand(context.worldId, 'cmd-shadow-configure', {
        windowId,
        config: intent.config || {},
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
