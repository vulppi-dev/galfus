import { KeyCode } from '../../types/events/keyboard';
import { enqueueCommand } from '../bridge/dispatch';
import type {
  InputStateComponent,
  Intent,
  System,
  UiFieldsetScope,
  UiFocusableNode,
  UiFormScope,
  UiStateComponent
} from '../ecs';

const WORLD_ENTITY_ID = 0;
const UI_INTENT_TYPES = [
  'ui-theme-define',
  'ui-theme-dispose',
  'ui-document-create',
  'ui-document-dispose',
  'ui-document-set-rect',
  'ui-document-set-theme',
  'ui-document-get-tree',
  'ui-document-get-layout-rects',
  'ui-apply-ops',
  'ui-debug-set',
  'ui-focus-set',
  'ui-focus-get',
  'ui-event-trace-set',
  'ui-image-create-from-buffer',
  'ui-image-dispose',
  'ui-clipboard-paste',
  'ui-screenshot-reply',
  'ui-access-kit-action-request',
  'ui-form-upsert',
  'ui-form-dispose',
  'ui-fieldset-upsert',
  'ui-fieldset-dispose',
  'ui-focusable-upsert',
  'ui-focusable-dispose',
  'ui-focus-next'
] as const;

function ensureUiState(world: Parameters<System>[0]): UiStateComponent {
  let worldStore = world.components.get(WORLD_ENTITY_ID);
  if (!worldStore) {
    worldStore = new Map();
    world.components.set(WORLD_ENTITY_ID, worldStore);
    world.entities.add(WORLD_ENTITY_ID);
  }

  let uiState = worldStore.get('UiState') as UiStateComponent | undefined;
  if (!uiState) {
    uiState = {
      type: 'UiState',
      forms: new Map(),
      fieldsets: new Map(),
      nodes: new Map(),
      focusByWindow: new Map()
    };
    worldStore.set('UiState', uiState);
  }

  return uiState;
}

function getInputState(world: Parameters<System>[0]): InputStateComponent | undefined {
  const worldStore = world.components.get(WORLD_ENTITY_ID);
  return worldStore?.get('InputState') as InputStateComponent | undefined;
}

function fieldsetKey(formId: string, fieldsetId: string): string {
  return `${formId}::${fieldsetId}`;
}

function canFocusNode(
  form: UiFormScope,
  node: UiFocusableNode,
  fieldsets: Map<string, UiFieldsetScope>
): boolean {
  if (form.disabled || node.disabled || node.tabIndex < 0) {
    return false;
  }

  if (form.activeFieldsetId && node.fieldsetId !== form.activeFieldsetId) {
    return false;
  }

  if (node.fieldsetId) {
    const fieldset = fieldsets.get(fieldsetKey(form.formId, node.fieldsetId));
    if (fieldset?.disabled && fieldset.legendNodeId !== node.nodeId) {
      return false;
    }
  }

  return true;
}

function sortFocusables(a: UiFocusableNode, b: UiFocusableNode): number {
  const aPositive = a.tabIndex > 0;
  const bPositive = b.tabIndex > 0;
  if (aPositive !== bPositive) return aPositive ? -1 : 1;

  if (aPositive && bPositive && a.tabIndex !== b.tabIndex) {
    return a.tabIndex - b.tabIndex;
  }

  if (a.orderHint !== b.orderHint) {
    return a.orderHint - b.orderHint;
  }

  return a.nodeId - b.nodeId;
}

function resolveTargetForm(
  uiState: UiStateComponent,
  windowId: number,
  formId?: string
): UiFormScope | undefined {
  if (formId) {
    const form = uiState.forms.get(formId);
    if (form && form.windowId === windowId && !form.disabled) {
      return form;
    }
  }

  const focused = uiState.focusByWindow.get(windowId);
  if (focused) {
    const form = uiState.forms.get(focused.formId);
    if (form && !form.disabled) {
      return form;
    }
  }

  const forms = Array.from(uiState.forms.values())
    .filter((f) => f.windowId === windowId && !f.disabled)
    .sort((a, b) => a.formId.localeCompare(b.formId));

  return forms[0];
}

function resolveNextNode(
  uiState: UiStateComponent,
  windowId: number,
  backwards: boolean,
  formId?: string
): { form: UiFormScope; nodeId: number } | null {
  const form = resolveTargetForm(uiState, windowId, formId);
  if (!form) return null;

  const candidates = Array.from(uiState.nodes.values())
    .filter((node) => node.formId === form.formId)
    .filter((node) => canFocusNode(form, node, uiState.fieldsets))
    .sort(sortFocusables);

  if (candidates.length === 0) {
    return null;
  }

  const focused = uiState.focusByWindow.get(windowId);
  const activeNodeId = focused?.formId === form.formId ? focused.nodeId : form.activeNodeId;
  const currentIndex = candidates.findIndex((node) => node.nodeId === activeNodeId);

  let nextIndex = currentIndex;
  if (currentIndex < 0) {
    nextIndex = backwards ? candidates.length - 1 : 0;
  } else {
    nextIndex = backwards ? currentIndex - 1 : currentIndex + 1;
  }

  const cycleMode = form.cycleMode;
  if (nextIndex < 0 || nextIndex >= candidates.length) {
    if (cycleMode === 'clamp') {
      return null;
    }
    nextIndex = nextIndex < 0 ? candidates.length - 1 : 0;
  }

  const target = candidates[nextIndex];
  if (!target) return null;
  return { form, nodeId: target.nodeId };
}

function applyFocus(
  uiState: UiStateComponent,
  windowId: number,
  formId: string,
  nodeId: number
): void {
  uiState.focusByWindow.set(windowId, { formId, nodeId });
  const form = uiState.forms.get(formId);
  if (form) {
    form.activeNodeId = nodeId;
  }
}

function processUiIntent(worldId: number, uiState: UiStateComponent, intent: Intent): boolean {
  switch (intent.type) {
    case 'ui-theme-define':
      enqueueCommand(worldId, 'cmd-ui-theme-define', intent.args);
      return true;
    case 'ui-theme-dispose':
      enqueueCommand(worldId, 'cmd-ui-theme-dispose', intent.args);
      return true;
    case 'ui-document-create':
      enqueueCommand(worldId, 'cmd-ui-document-create', intent.args);
      return true;
    case 'ui-document-dispose':
      enqueueCommand(worldId, 'cmd-ui-document-dispose', intent.args);
      return true;
    case 'ui-document-set-rect':
      enqueueCommand(worldId, 'cmd-ui-document-set-rect', intent.args);
      return true;
    case 'ui-document-set-theme':
      enqueueCommand(worldId, 'cmd-ui-document-set-theme', intent.args);
      return true;
    case 'ui-document-get-tree':
      enqueueCommand(worldId, 'cmd-ui-document-get-tree', intent.args);
      return true;
    case 'ui-document-get-layout-rects':
      enqueueCommand(worldId, 'cmd-ui-document-get-layout-rects', intent.args);
      return true;
    case 'ui-apply-ops':
      enqueueCommand(worldId, 'cmd-ui-apply-ops', intent.args);
      return true;
    case 'ui-debug-set':
      enqueueCommand(worldId, 'cmd-ui-debug-set', intent.args);
      return true;
    case 'ui-focus-set': {
      enqueueCommand(worldId, 'cmd-ui-focus-set', intent.args);
      const form = resolveTargetForm(uiState, intent.args.windowId);
      if (
        form &&
        form.realmId === intent.args.realmId &&
        form.documentId === intent.args.documentId
      ) {
        applyFocus(uiState, intent.args.windowId, form.formId, intent.args.nodeId ?? 0);
      }
      return true;
    }
    case 'ui-focus-get':
      enqueueCommand(worldId, 'cmd-ui-focus-get', intent.args);
      return true;
    case 'ui-event-trace-set':
      enqueueCommand(worldId, 'cmd-ui-event-trace-set', intent.args);
      return true;
    case 'ui-image-create-from-buffer':
      enqueueCommand(worldId, 'cmd-ui-image-create-from-buffer', intent.args);
      return true;
    case 'ui-image-dispose':
      enqueueCommand(worldId, 'cmd-ui-image-dispose', intent.args);
      return true;
    case 'ui-clipboard-paste':
      enqueueCommand(worldId, 'cmd-ui-clipboard-paste', intent.args);
      return true;
    case 'ui-screenshot-reply':
      enqueueCommand(worldId, 'cmd-ui-screenshot-reply', intent.args);
      return true;
    case 'ui-access-kit-action-request':
      enqueueCommand(worldId, 'cmd-ui-access-kit-action-request', intent.args);
      return true;
    case 'ui-form-upsert': {
      const existing = uiState.forms.get(intent.form.formId);
      uiState.forms.set(intent.form.formId, {
        formId: intent.form.formId,
        windowId: intent.form.windowId,
        realmId: intent.form.realmId,
        documentId: intent.form.documentId,
        disabled: intent.form.disabled ?? false,
        cycleMode: intent.form.cycleMode ?? 'wrap',
        activeFieldsetId: intent.form.activeFieldsetId,
        activeNodeId: existing?.activeNodeId
      });
      return true;
    }
    case 'ui-form-dispose': {
      uiState.forms.delete(intent.formId);
      for (const [key, fieldset] of uiState.fieldsets) {
        if (fieldset.formId === intent.formId) {
          uiState.fieldsets.delete(key);
        }
      }
      for (const [nodeId, node] of uiState.nodes) {
        if (node.formId === intent.formId) {
          uiState.nodes.delete(nodeId);
        }
      }
      for (const [windowId, focus] of uiState.focusByWindow) {
        if (focus.formId === intent.formId) {
          uiState.focusByWindow.delete(windowId);
        }
      }
      return true;
    }
    case 'ui-fieldset-upsert': {
      uiState.fieldsets.set(fieldsetKey(intent.fieldset.formId, intent.fieldset.fieldsetId), {
        formId: intent.fieldset.formId,
        fieldsetId: intent.fieldset.fieldsetId,
        disabled: intent.fieldset.disabled ?? false,
        legendNodeId: intent.fieldset.legendNodeId
      });
      return true;
    }
    case 'ui-fieldset-dispose':
      uiState.fieldsets.delete(fieldsetKey(intent.formId, intent.fieldsetId));
      return true;
    case 'ui-focusable-upsert':
      uiState.nodes.set(intent.focusable.nodeId, {
        formId: intent.focusable.formId,
        nodeId: intent.focusable.nodeId,
        tabIndex: intent.focusable.tabIndex ?? 0,
        fieldsetId: intent.focusable.fieldsetId,
        disabled: intent.focusable.disabled ?? false,
        orderHint: intent.focusable.orderHint ?? intent.focusable.nodeId
      });
      return true;
    case 'ui-focusable-dispose':
      uiState.nodes.delete(intent.nodeId);
      return true;
    case 'ui-focus-next': {
      const next = resolveNextNode(
        uiState,
        intent.windowId,
        intent.backwards ?? false,
        intent.formId
      );
      if (!next) return true;

      enqueueCommand(worldId, 'cmd-ui-focus-set', {
        windowId: next.form.windowId,
        realmId: next.form.realmId,
        documentId: next.form.documentId,
        nodeId: next.nodeId
      });
      applyFocus(uiState, next.form.windowId, next.form.formId, next.nodeId);
      return true;
    }
    default:
      return false;
  }
}

/**
 * Bridges UI-oriented intents into core UI commands and maintains focus state.
 *
 * It also provides default keyboard Tab navigation when explicit
 * `ui-focus-next` intents are not emitted by the application.
 */
export const UiBridgeSystem: System = (world, context) => {
  const uiState = ensureUiState(world);
  let explicitFocusNavigation = false;
  const intents = world.intentStore.takeMany(UI_INTENT_TYPES);

  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (!intent) continue;

    if (intent.type === 'ui-focus-next') {
      explicitFocusNavigation = true;
    }

    processUiIntent(context.worldId, uiState, intent);
  }

  if (!explicitFocusNavigation) {
    const inputState = getInputState(world);
    if (inputState?.keysJustPressed.has(KeyCode.Tab)) {
      const backwards =
        inputState.keysPressed.has(KeyCode.ShiftLeft) ||
        inputState.keysPressed.has(KeyCode.ShiftRight);
      const focusWindowId = world.primaryWindowId;
      if (focusWindowId !== undefined) {
        const next = resolveNextNode(uiState, focusWindowId, backwards);
        if (next) {
          enqueueCommand(context.worldId, 'cmd-ui-focus-set', {
            windowId: next.form.windowId,
            realmId: next.form.realmId,
            documentId: next.form.documentId,
            nodeId: next.nodeId
          });
          applyFocus(uiState, next.form.windowId, next.form.formId, next.nodeId);
        }
      }
    }
  }
};
