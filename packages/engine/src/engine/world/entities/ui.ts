import type {
  CmdUiAccessKitActionRequestArgs,
  CmdUiApplyOpsArgs,
  CmdUiClipboardPasteArgs,
  CmdUiDebugSetArgs,
  CmdUiDocumentCreateArgs,
  CmdUiDocumentDisposeArgs,
  CmdUiDocumentGetLayoutRectsArgs,
  CmdUiDocumentGetTreeArgs,
  CmdUiDocumentSetRectArgs,
  CmdUiDocumentSetThemeArgs,
  CmdUiEventTraceSetArgs,
  CmdUiFocusGetArgs,
  CmdUiFocusSetArgs,
  CmdUiImageCreateFromBufferArgs,
  CmdUiImageDisposeArgs,
  CmdUiScreenshotReplyArgs,
  CmdUiThemeDefineArgs,
  CmdUiThemeDisposeArgs
} from '../../../types/cmds/ui';
import type { UiFocusCycleMode } from '../../ecs';
import { emitIntent } from './intents';

/** Defines or updates a UI theme. */
export function uiDefineTheme(worldId: number, args: CmdUiThemeDefineArgs): void {
  emitIntent(worldId, { type: 'ui-theme-define', args });
}

/** Disposes a UI theme. */
export function uiDisposeTheme(worldId: number, args: CmdUiThemeDisposeArgs): void {
  emitIntent(worldId, { type: 'ui-theme-dispose', args });
}

/** Creates a UI document. */
export function uiCreateDocument(worldId: number, args: CmdUiDocumentCreateArgs): void {
  emitIntent(worldId, { type: 'ui-document-create', args });
}

/** Disposes a UI document. */
export function uiDisposeDocument(worldId: number, args: CmdUiDocumentDisposeArgs): void {
  emitIntent(worldId, { type: 'ui-document-dispose', args });
}

/** Updates document rectangle. */
export function uiSetDocumentRect(worldId: number, args: CmdUiDocumentSetRectArgs): void {
  emitIntent(worldId, { type: 'ui-document-set-rect', args });
}

/** Updates document theme. */
export function uiSetDocumentTheme(worldId: number, args: CmdUiDocumentSetThemeArgs): void {
  emitIntent(worldId, { type: 'ui-document-set-theme', args });
}

/** Applies document ops. */
export function uiApplyOps(worldId: number, args: CmdUiApplyOpsArgs): void {
  emitIntent(worldId, { type: 'ui-apply-ops', args });
}

/** Requests UI document tree for introspection. */
export function uiGetDocumentTree(worldId: number, args: CmdUiDocumentGetTreeArgs): void {
  emitIntent(worldId, { type: 'ui-document-get-tree', args });
}

/** Requests UI layout rects for introspection. */
export function uiGetLayoutRects(worldId: number, args: CmdUiDocumentGetLayoutRectsArgs): void {
  emitIntent(worldId, { type: 'ui-document-get-layout-rects', args });
}

/** Enables/disables runtime UI debug overlays. */
export function uiSetDebug(worldId: number, args: CmdUiDebugSetArgs): void {
  emitIntent(worldId, { type: 'ui-debug-set', args });
}

/** Sets focused UI node. */
export function uiSetFocus(worldId: number, args: CmdUiFocusSetArgs): void {
  emitIntent(worldId, { type: 'ui-focus-set', args });
}

/** Requests current UI focus state. */
export function uiGetFocus(worldId: number, args: CmdUiFocusGetArgs = {}): void {
  emitIntent(worldId, { type: 'ui-focus-get', args });
}

/** Configures UI event trace level/sampling. */
export function uiSetEventTrace(worldId: number, args: CmdUiEventTraceSetArgs): void {
  emitIntent(worldId, { type: 'ui-event-trace-set', args });
}

/** Creates a UI image from uploaded bytes. */
export function uiCreateImageFromBuffer(
  worldId: number,
  args: CmdUiImageCreateFromBufferArgs
): void {
  emitIntent(worldId, { type: 'ui-image-create-from-buffer', args });
}

/** Disposes a UI image. */
export function uiDisposeImage(worldId: number, args: CmdUiImageDisposeArgs): void {
  emitIntent(worldId, { type: 'ui-image-dispose', args });
}

/** Delivers host clipboard paste event to UI. */
export function uiClipboardPaste(worldId: number, args: CmdUiClipboardPasteArgs): void {
  emitIntent(worldId, { type: 'ui-clipboard-paste', args });
}

/** Delivers screenshot response bytes to UI. */
export function uiScreenshotReply(worldId: number, args: CmdUiScreenshotReplyArgs): void {
  emitIntent(worldId, { type: 'ui-screenshot-reply', args });
}

/** Delivers AccessKit action request to UI. */
export function uiAccessKitActionRequest(
  worldId: number,
  args: CmdUiAccessKitActionRequestArgs
): void {
  emitIntent(worldId, { type: 'ui-access-kit-action-request', args });
}

/** Registers or updates a UI form scope used for tab navigation. */
export function uiFormUpsert(
  worldId: number,
  form: {
    formId: string;
    windowId: number;
    realmId: number;
    documentId: number;
    disabled?: boolean;
    cycleMode?: UiFocusCycleMode;
    activeFieldsetId?: string;
  }
): void {
  emitIntent(worldId, { type: 'ui-form-upsert', form });
}

/** Disposes a registered UI form scope. */
export function uiFormDispose(worldId: number, formId: string): void {
  emitIntent(worldId, { type: 'ui-form-dispose', formId });
}

/** Registers or updates fieldset metadata. */
export function uiFieldsetUpsert(
  worldId: number,
  fieldset: {
    formId: string;
    fieldsetId: string;
    disabled?: boolean;
    legendNodeId?: number;
  }
): void {
  emitIntent(worldId, { type: 'ui-fieldset-upsert', fieldset });
}

/** Disposes fieldset metadata. */
export function uiFieldsetDispose(worldId: number, formId: string, fieldsetId: string): void {
  emitIntent(worldId, { type: 'ui-fieldset-dispose', formId, fieldsetId });
}

/** Registers or updates focusable node metadata. */
export function uiFocusableUpsert(
  worldId: number,
  focusable: {
    formId: string;
    nodeId: number;
    tabIndex?: number;
    fieldsetId?: string;
    disabled?: boolean;
    orderHint?: number;
  }
): void {
  emitIntent(worldId, { type: 'ui-focusable-upsert', focusable });
}

/** Disposes focusable node metadata. */
export function uiFocusableDispose(worldId: number, nodeId: number): void {
  emitIntent(worldId, { type: 'ui-focusable-dispose', nodeId });
}

/** Advances focus within a form scope using tab ordering. */
export function uiFocusNext(
  worldId: number,
  args: { windowId: number; backwards?: boolean; formId?: string }
): void {
  emitIntent(worldId, { type: 'ui-focus-next', ...args });
}
