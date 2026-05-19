import type { Vec2 as vec2 } from '../../math/index';
import type { WindowState, CursorGrabMode, CursorIcon, UserAttentionType } from '../kinds';

/** Command payload for creating a window. */
export interface CmdWindowCreateArgs {
  windowId: number;
  title?: string;
  size?: vec2;
  position?: vec2;
  canvasId?: string;
  borderless?: boolean;
  resizable?: boolean;
  transparent?: boolean;
  initialState?: WindowState;
}

/** Result payload for window create. */
export interface CmdResultWindowCreate {
  success: boolean;
  message: string;
  realmId?: number;
}

/** Command payload for closing a window. */
export interface CmdWindowCloseArgs {
  windowId: number;
}

/** Result payload for window close. */
export interface CmdResultWindowClose {
  success: boolean;
  message: string;
}

/** Command payload for changing and/or querying window measurements. */
export interface CmdWindowMeasurementArgs {
  windowId: number;
  position?: vec2;
  size?: vec2;
  getPosition?: boolean;
  getSize?: boolean;
  getOuterSize?: boolean;
}

/** Result payload for window measurement command. */
export interface CmdResultWindowMeasurement {
  success: boolean;
  message: string;
  position?: vec2;
  size?: vec2;
  outerSize?: vec2;
}

/** Command payload for cursor updates. */
export interface CmdWindowCursorArgs {
  windowId: number;
  visible?: boolean;
  mode?: CursorGrabMode;
  icon?: CursorIcon;
}

/** Result payload for cursor command. */
export interface CmdResultWindowCursor {
  success: boolean;
  message: string;
}

export type WindowStateAction = 'focus' | 'request-attention';

/** Command payload for state updates/queries. */
export interface CmdWindowStateArgs {
  windowId: number;
  title?: string;
  state?: WindowState;
  iconBufferId?: number;
  decorations?: boolean;
  resizable?: boolean;
  action?: WindowStateAction;
  attentionType?: UserAttentionType;
  getState?: boolean;
  getDecorations?: boolean;
  getResizable?: boolean;
}

/** Result payload for state command. */
export interface CmdResultWindowState {
  success: boolean;
  message: string;
  state?: WindowState;
  decorations?: boolean;
  resizable?: boolean;
}
