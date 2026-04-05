import type { EngineCmd } from '../../types/cmds';
import { enqueueCommand } from '../bridge/dispatch';
import type { System } from '../ecs';

/**
 * Handles diagnostic intents that query core-side resource lists.
 *
 * This system is intentionally narrow: it transforms
 * `request-resource-list` intents into typed `cmd-*-list` commands.
 */
export const DiagnosticsSystem: System = (world, context) => {
  const intents = world.intentStore.take('request-resource-list');
  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (intent?.type === 'request-resource-list') {
      let windowId = world.primaryWindowId;
      if (windowId === undefined) {
        for (const boundWindowId of world.targetWindowBindings.values()) {
          windowId = boundWindowId;
          break;
        }
      }
      if (windowId === undefined) {
        continue;
      }
      const type = intent.resourceType;
      const cmdType = `cmd-${type}-list` as EngineCmd['type'];
      enqueueCommand(context.worldId, cmdType, {
        windowId,
      });
    }
  }
};
