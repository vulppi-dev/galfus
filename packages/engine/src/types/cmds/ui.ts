import type { JsonObject, JsonValue } from '../json';

export type UiThemeValue = JsonValue;

export type UiNodeKind =
  | 'container'
  | 'window'
  | 'panel'
  | 'split-pane'
  | 'area'
  | 'frame'
  | 'scroll-area'
  | 'grid'
  | 'popup'
  | 'tooltip'
  | 'modal'
  | 'resize'
  | 'scene'
  | 'canvas'
  | 'text'
  | 'rich-text'
  | 'link'
  | 'hyperlink'
  | 'button'
  | 'checkbox'
  | 'radio'
  | 'selectable-label'
  | 'toggle'
  | 'slider'
  | 'drag-value'
  | 'progress-bar'
  | 'combo-box'
  | 'menu-button'
  | 'collapsing-header'
  | 'image-button'
  | 'spinner'
  | 'text-edit'
  | 'input'
  | 'image'
  | 'widget-realm-viewport'
  | 'separator'
  | 'spacer';

export type UiNodeProps = JsonObject;

export interface UiAnim {
  opacity?: number;
  translateY?: number;
}

export interface UiNode {
  id: number;
  kind: UiNodeKind;
  props: UiNodeProps;
  tooltip?: string;
  contextMenu?: string[];
  anim?: UiAnim;
  display?: boolean;
  visible?: boolean;
  opacity?: number;
  zIndex?: number;
}

export type UiOp =
  | {
      type: 'add';
      content: { parent?: number | null; node: UiNode; index?: number | null };
    }
  | { type: 'remove'; content: { node_id: number } }
  | { type: 'clear'; content: { parent?: number | null } }
  | { type: 'set'; content: { node_id: number; props: UiNodeProps } }
  | {
      type: 'move';
      content: {
        node_id: number;
        new_parent?: number | null;
        index?: number | null;
      };
    };

export interface CmdUiThemeDefineArgs {
  themeId: number;
  version?: number;
  data?: Record<string, UiThemeValue>;
  fontData?: Record<string, number[]>;
  fontFamilies?: Record<string, string[]>;
}

export interface CmdResultUiThemeDefine {
  success: boolean;
  message: string;
  themeId?: number;
  version?: number;
}

export interface CmdUiThemeDisposeArgs {
  themeId: number;
}

export interface CmdResultUiThemeDispose {
  success: boolean;
  message: string;
}

export interface CmdUiDocumentCreateArgs {
  documentId: number;
  realmId: number;
  rect: [number, number, number, number];
  themeId?: number;
}

export interface CmdResultUiDocumentCreate {
  success: boolean;
  message: string;
  documentId?: number;
}

export interface CmdUiDocumentDisposeArgs {
  documentId: number;
}

export interface CmdResultUiDocumentDispose {
  success: boolean;
  message: string;
}

export interface CmdUiDocumentSetRectArgs {
  documentId: number;
  rect: [number, number, number, number];
}

export interface CmdResultUiDocumentSetRect {
  success: boolean;
  message: string;
}

export interface CmdUiDocumentSetThemeArgs {
  documentId: number;
  themeId?: number;
}

export interface CmdResultUiDocumentSetTheme {
  success: boolean;
  message: string;
}

export interface CmdUiDocumentGetTreeArgs {
  documentId: number;
}

export interface UiDocumentTreeNode {
  nodeId: number;
  kind: UiNodeKind;
  zIndex: number;
  children: UiDocumentTreeNode[];
}

export interface CmdResultUiDocumentGetTree {
  success: boolean;
  message: string;
  documentId?: number;
  version?: number;
  rootNodes: UiDocumentTreeNode[];
}

export interface CmdUiDocumentGetLayoutRectsArgs {
  documentId: number;
}

export interface UiNodeLayoutRect {
  nodeId: number;
  rect: [number, number, number, number];
}

export interface CmdResultUiDocumentGetLayoutRects {
  success: boolean;
  message: string;
  documentId?: number;
  version?: number;
  rects: UiNodeLayoutRect[];
}

export interface CmdUiApplyOpsArgs {
  documentId: number;
  version: number;
  ops: UiOp[];
}

export interface CmdResultUiApplyOps {
  success: boolean;
  message: string;
  version?: number;
}

export interface CmdUiDebugSetArgs {
  enabled: boolean;
  showBounds?: boolean;
  showIds?: boolean;
  showProfile?: boolean;
}

export interface CmdResultUiDebugSet {
  success: boolean;
  message: string;
}

export interface CmdUiFocusSetArgs {
  windowId: number;
  realmId: number;
  documentId: number;
  nodeId?: number;
}

export interface CmdUiFocusGetArgs {
  windowId?: number;
}

export interface CmdResultUiFocusSet {
  success: boolean;
  message: string;
}

export interface UiFocusEntry {
  windowId: number;
  realmId: number;
  documentId: number;
  nodeId: number;
}

export interface CmdResultUiFocusGet {
  success: boolean;
  message: string;
  entries: UiFocusEntry[];
}

export type UiCmdPointerTraceLevel = 'off' | 'errors' | 'basic' | 'full';

export interface CmdUiEventTraceSetArgs {
  level?: UiCmdPointerTraceLevel;
  samplingPercent?: number;
}

export interface CmdResultUiEventTraceSet {
  success: boolean;
  message: string;
  level?: UiCmdPointerTraceLevel;
  samplingPercent?: number;
}

export interface CmdUiImageCreateFromBufferArgs {
  imageId: number;
  bufferId: number;
  label?: string;
}

export interface CmdResultUiImageCreateFromBuffer {
  success: boolean;
  message: string;
  pending: boolean;
}

export interface CmdUiImageDisposeArgs {
  imageId: number;
}

export interface CmdResultUiImageDispose {
  success: boolean;
  message: string;
}

export interface CmdUiClipboardPasteArgs {
  windowId: number;
  text: string;
}

export interface CmdUiScreenshotReplyArgs {
  windowId: number;
  realmId?: number;
  width: number;
  height: number;
  rgba: number[];
}

export interface CmdUiAccessKitActionRequestArgs {
  windowId: number;
  realmId?: number;
  action: string;
}

export interface CmdResultUiInputEvent {
  success: boolean;
  message: string;
}
