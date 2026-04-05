import { createWorldUI as createWorldUIRaw } from '../api';
import type { UiFocusCycleMode } from '../ecs';
import type {
  CmdRealmRenderGraphBindArgs,
  CmdRenderGraphDisposeArgs,
  CmdRenderGraphListArgs,
  CmdRenderGraphUpsertArgs,
} from '../../types/cmds/render-graph';
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
  CmdUiThemeDisposeArgs,
} from '../../types/cmds/ui';
import type { UiEvent } from '../../types/events/ui';
import { getUiEvents } from '../input/api';
import { EngineError } from '../errors';
import { getWorldOrThrow } from '../bridge/guards';
import {
  getWorldRealmId as getWorldRealmIdRaw,
  uiAccessKitActionRequest as uiAccessKitActionRequestRaw,
  uiApplyOps as uiApplyOpsRaw,
  uiClipboardPaste as uiClipboardPasteRaw,
  uiCreateDocument as uiCreateDocumentRaw,
  uiCreateImageFromBuffer as uiCreateImageFromBufferRaw,
  uiDefineTheme as uiDefineThemeRaw,
  uiDisposeDocument as uiDisposeDocumentRaw,
  uiDisposeImage as uiDisposeImageRaw,
  uiDisposeTheme as uiDisposeThemeRaw,
  uiFieldsetDispose as uiFieldsetDisposeRaw,
  uiFieldsetUpsert as uiFieldsetUpsertRaw,
  uiFocusableDispose as uiFocusableDisposeRaw,
  uiFocusableUpsert as uiFocusableUpsertRaw,
  uiFocusNext as uiFocusNextRaw,
  uiFormDispose as uiFormDisposeRaw,
  uiFormUpsert as uiFormUpsertRaw,
  bindRealmRenderGraph as bindRealmRenderGraphRaw,
  disposeRenderGraph as disposeRenderGraphRaw,
  uiGetDocumentTree as uiGetDocumentTreeRaw,
  uiGetFocus as uiGetFocusRaw,
  uiGetLayoutRects as uiGetLayoutRectsRaw,
  listRenderGraphs as listRenderGraphsRaw,
  uiScreenshotReply as uiScreenshotReplyRaw,
  uiSetDebug as uiSetDebugRaw,
  uiSetDocumentRect as uiSetDocumentRectRaw,
  uiSetDocumentTheme as uiSetDocumentThemeRaw,
  uiSetEventTrace as uiSetEventTraceRaw,
  uiSetFocus as uiSetFocusRaw,
  upsertRenderGraph as upsertRenderGraphRaw,
} from './entities';
import type { CommandId, WorldUIId } from './types';
import { asCommandId, asWorldNumber, asWorldUIId } from './types';

type CreateUIWorldOptions = {
  importance?: number;
  cachePolicy?: number;
  flags?: number;
};

type UIFormUpsertArgs = {
  formId: string;
  documentId: number;
  disabled?: boolean;
  cycleMode?: UiFocusCycleMode;
  activeFieldsetId?: string;
};

type UIFocusNextArgs = {
  backwards?: boolean;
  formId?: string;
};

function resolveWorldUIContext(worldId: number): {
  windowId: number;
  realmId: number;
} {
  const world = getWorldOrThrow(worldId);
  const realmId = world.coreRealmId ?? world.worldId;

  if (world.primaryWindowId !== undefined) {
    return { windowId: world.primaryWindowId, realmId };
  }
  for (const windowId of world.targetWindowBindings.values()) {
    return { windowId, realmId };
  }
  throw new EngineError(
    'WindowNotFound',
    `World ${worldId} has no window binding available for UI operation.`,
  );
}

/**
 * Creates a UI world.
 *
 * The world is realm-backed internally, but realm details are hidden from this API.
 * Use `Mount.mountWorld(...)` to present this world in one or more targets.
 */
export function createUIWorld(options?: CreateUIWorldOptions): WorldUIId {
  return asWorldUIId(createWorldUIRaw(options));
}

/** Returns resolved core realm id for this UI world, or `null` if not ready yet. */
export function getUIWorldRealmId(worldId: WorldUIId): number | null {
  return getWorldRealmIdRaw(asWorldNumber(worldId));
}

/** Creates or updates a render graph definition in core. */
export function upsertUIRenderGraph(
  worldId: WorldUIId,
  args: CmdRenderGraphUpsertArgs,
): CommandId {
  return asCommandId(upsertRenderGraphRaw(asWorldNumber(worldId), args));
}

/** Disposes a render graph definition from core. */
export function disposeUIRenderGraph(
  worldId: WorldUIId,
  args: CmdRenderGraphDisposeArgs,
): CommandId {
  return asCommandId(disposeRenderGraphRaw(asWorldNumber(worldId), args));
}

/** Requests render graph catalog from core. */
export function listUIRenderGraphs(
  worldId: WorldUIId,
  args: CmdRenderGraphListArgs = {},
): CommandId {
  return asCommandId(listRenderGraphsRaw(asWorldNumber(worldId), args));
}

/** Binds realm to a render graph id. */
export function bindUIRealmRenderGraph(
  worldId: WorldUIId,
  args: CmdRealmRenderGraphBindArgs,
): CommandId {
  return asCommandId(bindRealmRenderGraphRaw(asWorldNumber(worldId), args));
}

/** Defines a theme in a UI world. */
export function defineUITheme(
  worldId: WorldUIId,
  args: CmdUiThemeDefineArgs,
): void {
  uiDefineThemeRaw(asWorldNumber(worldId), args);
}

/** Disposes a theme in a UI world. */
export function disposeUITheme(
  worldId: WorldUIId,
  args: CmdUiThemeDisposeArgs,
): void {
  uiDisposeThemeRaw(asWorldNumber(worldId), args);
}

/** Creates a document in a UI world. */
export function createUIDocument(
  worldId: WorldUIId,
  args: CmdUiDocumentCreateArgs,
): void {
  uiCreateDocumentRaw(asWorldNumber(worldId), args);
}

/** Disposes a document in a UI world. */
export function disposeUIDocument(
  worldId: WorldUIId,
  args: CmdUiDocumentDisposeArgs,
): void {
  uiDisposeDocumentRaw(asWorldNumber(worldId), args);
}

/** Sets document rect in a UI world. */
export function setUIDocumentRect(
  worldId: WorldUIId,
  args: CmdUiDocumentSetRectArgs,
): void {
  uiSetDocumentRectRaw(asWorldNumber(worldId), args);
}

/** Sets document theme in a UI world. */
export function setUIDocumentTheme(
  worldId: WorldUIId,
  args: CmdUiDocumentSetThemeArgs,
): void {
  uiSetDocumentThemeRaw(asWorldNumber(worldId), args);
}

/** Applies UI ops batch in a UI world. */
export function applyUIOps(worldId: WorldUIId, args: CmdUiApplyOpsArgs): void {
  uiApplyOpsRaw(asWorldNumber(worldId), args);
}

/** Requests UI document tree. */
export function getUIDocumentTree(
  worldId: WorldUIId,
  args: CmdUiDocumentGetTreeArgs,
): void {
  uiGetDocumentTreeRaw(asWorldNumber(worldId), args);
}

/** Requests UI layout rects. */
export function getUILayoutRects(
  worldId: WorldUIId,
  args: CmdUiDocumentGetLayoutRectsArgs,
): void {
  uiGetLayoutRectsRaw(asWorldNumber(worldId), args);
}

/** Configures UI debug flags. */
export function setUIDebug(worldId: WorldUIId, args: CmdUiDebugSetArgs): void {
  uiSetDebugRaw(asWorldNumber(worldId), args);
}

/** Sets focus in a UI world. */
export function setUIFocus(worldId: WorldUIId, args: CmdUiFocusSetArgs): void {
  uiSetFocusRaw(asWorldNumber(worldId), args);
}

/** Requests focus state in a UI world. */
export function getUIFocus(
  worldId: WorldUIId,
  args: CmdUiFocusGetArgs = {},
): void {
  uiGetFocusRaw(asWorldNumber(worldId), args);
}

/** Configures UI event trace in a UI world. */
export function setUIEventTrace(
  worldId: WorldUIId,
  args: CmdUiEventTraceSetArgs,
): void {
  uiSetEventTraceRaw(asWorldNumber(worldId), args);
}

/** Creates UI image from a previously uploaded buffer. */
export function createUIImageFromBuffer(
  worldId: WorldUIId,
  args: CmdUiImageCreateFromBufferArgs,
): void {
  uiCreateImageFromBufferRaw(asWorldNumber(worldId), args);
}

/** Disposes UI image resource. */
export function disposeUIImage(
  worldId: WorldUIId,
  args: CmdUiImageDisposeArgs,
): void {
  uiDisposeImageRaw(asWorldNumber(worldId), args);
}

/** Sends clipboard paste payload to UI world. */
export function pasteUIClipboard(
  worldId: WorldUIId,
  args: CmdUiClipboardPasteArgs,
): void {
  uiClipboardPasteRaw(asWorldNumber(worldId), args);
}

/** Replies to UI screenshot request. */
export function replyUIScreenshot(
  worldId: WorldUIId,
  args: CmdUiScreenshotReplyArgs,
): void {
  uiScreenshotReplyRaw(asWorldNumber(worldId), args);
}

/** Sends AccessKit action request. */
export function requestUIAccessKitAction(
  worldId: WorldUIId,
  args: CmdUiAccessKitActionRequestArgs,
): void {
  uiAccessKitActionRequestRaw(asWorldNumber(worldId), args);
}

/**
 * Upserts UI form metadata.
 *
 * `windowId` and `realmId` are resolved internally from world bindings.
 */
export function upsertUIForm(worldId: WorldUIId, form: UIFormUpsertArgs): void {
  const rawWorldId = asWorldNumber(worldId);
  const context = resolveWorldUIContext(rawWorldId);
  uiFormUpsertRaw(rawWorldId, {
    ...form,
    windowId: context.windowId,
    realmId: context.realmId,
  });
}

/** Disposes UI form metadata. */
export function disposeUIForm(worldId: WorldUIId, formId: string): void {
  uiFormDisposeRaw(asWorldNumber(worldId), formId);
}

/** Upserts UI fieldset metadata. */
export function upsertUIFieldset(
  worldId: WorldUIId,
  args: Parameters<typeof uiFieldsetUpsertRaw>[1],
): void {
  uiFieldsetUpsertRaw(asWorldNumber(worldId), args);
}

/** Disposes UI fieldset metadata. */
export function disposeUIFieldset(
  worldId: WorldUIId,
  args: { formId: string; fieldsetId: string },
): void {
  uiFieldsetDisposeRaw(asWorldNumber(worldId), args.formId, args.fieldsetId);
}

/** Upserts focusable metadata. */
export function upsertUIFocusable(
  worldId: WorldUIId,
  args: Parameters<typeof uiFocusableUpsertRaw>[1],
): void {
  uiFocusableUpsertRaw(asWorldNumber(worldId), args);
}

/** Disposes focusable metadata. */
export function disposeUIFocusable(worldId: WorldUIId, nodeId: number): void {
  uiFocusableDisposeRaw(asWorldNumber(worldId), nodeId);
}

/**
 * Requests moving focus to next/previous focusable.
 *
 * `windowId` is resolved internally from world bindings.
 */
export function focusUINext(
  worldId: WorldUIId,
  args: UIFocusNextArgs = {},
): void {
  const rawWorldId = asWorldNumber(worldId);
  const context = resolveWorldUIContext(rawWorldId);
  uiFocusNextRaw(rawWorldId, {
    ...args,
    windowId: context.windowId,
  });
}

/**
 * Returns UI events mirrored from the latest processed frame.
 *
 * This accessor is read-only from the host perspective; events are replaced each tick.
 */
export function getUIEvents(worldId: WorldUIId): UiEvent[] {
  return getUiEvents(asWorldNumber(worldId));
}
