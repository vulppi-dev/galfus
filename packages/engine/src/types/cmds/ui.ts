export type UiThemeValue = boolean | number | string;
export type UiByteArray = Uint8Array | number[];

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

export type UiLayoutDirection = 'row' | 'row-reverse' | 'column' | 'column-reverse' | 'grid';

export type UiAlign = 'start' | 'center' | 'end' | 'stretch';

export type UiLength = { type: 'auto' } | { type: 'fill' } | { type: 'px'; content: number };

export type UiPanelKind = 'side-left' | 'side-right' | 'top' | 'bottom' | 'central';

export type UiSplitDirection = 'horizontal' | 'vertical';

export interface UiStroke {
  width: number;
  color: UiColor;
}

export interface UiWindowAnchor {
  x: number;
  y: number;
}

export interface UiSize {
  width: UiLength;
  height: UiLength;
}

export interface UiLayout {
  direction: UiLayoutDirection;
  align?: UiAlign;
  justify?: UiAlign;
  gap?: number;
  columns?: number;
  wrap?: boolean;
  wrapLimit?: number;
}

export interface UiPadding {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface UiColor {
  r: number;
  g: number;
  b: number;
  a: number;
}

export type UiTextAlign =
  | 'left-top'
  | 'left-center'
  | 'left-bottom'
  | 'center-top'
  | 'center-center'
  | 'center-bottom'
  | 'right-top'
  | 'right-center'
  | 'right-bottom';

export interface UiPaintStroke {
  width: number;
  color: UiColor;
  join?: string;
  cap?: string;
}

export type UiPaintOp =
  | {
      type: 'line-segment';
      content: {
        from: [number, number];
        to: [number, number];
        stroke: UiPaintStroke;
      };
    }
  | {
      type: 'polyline';
      content: {
        points: [number, number][];
        stroke: UiPaintStroke;
      };
    }
  | {
      type: 'rect';
      content: {
        min: [number, number];
        max: [number, number];
        rounding?: number;
        stroke: UiPaintStroke;
      };
    }
  | {
      type: 'rect-filled';
      content: {
        min: [number, number];
        max: [number, number];
        rounding?: number;
        fill: UiColor;
      };
    }
  | {
      type: 'circle';
      content: {
        center: [number, number];
        radius: number;
        stroke: UiPaintStroke;
      };
    }
  | {
      type: 'circle-filled';
      content: {
        center: [number, number];
        radius: number;
        fill: UiColor;
      };
    }
  | {
      type: 'convex-polygon';
      content: {
        points: [number, number][];
        fill: UiColor;
        stroke?: UiPaintStroke;
      };
    }
  | {
      type: 'quadratic-bezier';
      content: {
        from: [number, number];
        ctrl: [number, number];
        to: [number, number];
        steps?: number;
        stroke: UiPaintStroke;
      };
    }
  | {
      type: 'cubic-bezier';
      content: {
        from: [number, number];
        ctrl1: [number, number];
        ctrl2: [number, number];
        to: [number, number];
        steps?: number;
        stroke: UiPaintStroke;
      };
    }
  | {
      type: 'text';
      content: {
        position: [number, number];
        text: string;
        size?: number;
        color: UiColor;
        align?: UiTextAlign;
      };
    };

export type UiImageSource =
  | { type: 'ui-image'; content: number }
  | { type: 'target'; content: number };

export type UiAnimEasing = 'linear' | 'ease-in-out';

export interface UiAnimSpec {
  from: number;
  to: number;
  durationMs: number;
  easing?: UiAnimEasing;
}

export interface UiAnim {
  opacity?: UiAnimSpec;
  translateY?: UiAnimSpec;
}

export type UiNodeProps =
  | {
      type: 'container';
      content: {
        layout?: UiLayout;
        padding?: UiPadding;
        size?: UiSize;
        scrollX?: boolean;
        scrollY?: boolean;
      };
    }
  | {
      type: 'window';
      content: {
        title: string;
        open?: boolean;
        movable?: boolean;
        resizable?: boolean;
        collapsible?: boolean;
        anchored?: UiWindowAnchor;
        size?: UiSize;
      };
    }
  | {
      type: 'panel';
      content: {
        kind: UiPanelKind;
        resizable?: boolean;
        size?: UiSize;
        minSize?: number;
        maxSize?: number;
      };
    }
  | {
      type: 'split-pane';
      content: {
        direction: UiSplitDirection;
        ratio?: number;
        resizable?: boolean;
        minA?: number;
        maxA?: number;
        minB?: number;
        maxB?: number;
      };
    }
  | {
      type: 'area';
      content: {
        label?: string;
        x?: number;
        y?: number;
        draggable?: boolean;
        size?: UiSize;
      };
    }
  | {
      type: 'frame';
      content: {
        padding?: UiPadding;
        fill?: UiColor;
        stroke?: UiStroke;
        rounding?: number;
        size?: UiSize;
      };
    }
  | {
      type: 'scroll-area';
      content: {
        scrollX?: boolean;
        scrollY?: boolean;
        autoShrink?: boolean;
        size?: UiSize;
      };
    }
  | {
      type: 'grid';
      content: {
        columns?: number;
        striped?: boolean;
        minColWidth?: number;
        size?: UiSize;
      };
    }
  | {
      type: 'popup';
      content: {
        title?: string;
        open?: boolean;
        size?: UiSize;
      };
    }
  | { type: 'tooltip'; content: { text: string } }
  | {
      type: 'modal';
      content: {
        title: string;
        open?: boolean;
        size?: UiSize;
      };
    }
  | {
      type: 'resize';
      content: {
        size?: UiSize;
        minSize?: UiSize;
        maxSize?: UiSize;
      };
    }
  | {
      type: 'scene';
      content: {
        size?: UiSize;
        zoomMin?: number;
        zoomMax?: number;
        panEnabled?: boolean;
      };
    }
  | {
      type: 'canvas';
      content: {
        ops: UiPaintOp[];
        size?: UiSize;
        clip?: boolean;
      };
    }
  | {
      type: 'text';
      content: {
        text: string;
        size?: number;
        color?: UiColor;
      };
    }
  | {
      type: 'rich-text';
      content: {
        text: string;
        size?: number;
        color?: UiColor;
        strong?: boolean;
        italics?: boolean;
        underline?: boolean;
        strikethrough?: boolean;
        monospace?: boolean;
      };
    }
  | {
      type: 'link';
      content: {
        label: string;
        enabled?: boolean;
      };
    }
  | {
      type: 'hyperlink';
      content: {
        label: string;
        url: string;
        enabled?: boolean;
      };
    }
  | {
      type: 'button';
      content: {
        label: string;
        enabled?: boolean;
      };
    }
  | {
      type: 'checkbox';
      content: {
        label: string;
        checked: boolean;
        enabled?: boolean;
      };
    }
  | {
      type: 'radio';
      content: {
        label: string;
        selected: boolean;
        enabled?: boolean;
      };
    }
  | {
      type: 'selectable-label';
      content: {
        label: string;
        selected: boolean;
        enabled?: boolean;
      };
    }
  | {
      type: 'toggle';
      content: {
        label: string;
        value: boolean;
        enabled?: boolean;
      };
    }
  | {
      type: 'slider';
      content: {
        value: number;
        min: number;
        max: number;
        step?: number;
        enabled?: boolean;
        label?: string;
      };
    }
  | {
      type: 'drag-value';
      content: {
        value: number;
        speed?: number;
        min?: number;
        max?: number;
        prefix?: string;
        suffix?: string;
        enabled?: boolean;
      };
    }
  | {
      type: 'progress-bar';
      content: {
        value: number;
        text?: string;
        animate?: boolean;
        showPercentage?: boolean;
      };
    }
  | {
      type: 'combo-box';
      content: {
        label: string;
        selected: string;
        options: string[];
        enabled?: boolean;
      };
    }
  | {
      type: 'menu-button';
      content: {
        label: string;
        enabled?: boolean;
      };
    }
  | {
      type: 'collapsing-header';
      content: {
        label: string;
        open?: boolean;
        enabled?: boolean;
      };
    }
  | {
      type: 'image-button';
      content: {
        source: UiImageSource;
        size?: UiSize;
        enabled?: boolean;
      };
    }
  | {
      type: 'spinner';
      content: {
        size?: number;
      };
    }
  | {
      type: 'text-edit';
      content: {
        value: string;
        placeholder?: string;
        multiline?: boolean;
        password?: boolean;
        charLimit?: number;
        enabled?: boolean;
      };
    }
  | {
      type: 'input';
      content: {
        value: string;
        placeholder?: string;
        enabled?: boolean;
      };
    }
  | {
      type: 'image';
      content: {
        source: UiImageSource;
        size?: UiSize;
      };
    }
  | {
      type: 'widget-realm-viewport';
      content: {
        targetId: number;
        size?: UiSize;
      };
    }
  | { type: 'separator' }
  | {
      type: 'spacer';
      content: {
        width?: number;
        height?: number;
      };
    };

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
  fontData?: Record<string, UiByteArray>;
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
  rgba: UiByteArray;
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
