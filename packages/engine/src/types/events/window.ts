import type { CursorGrabMode, WindowState } from '../kinds';

/** Payload for window create event. */
export interface WindowEventOnCreateData {
  windowId: number;
}

/** Payload for window resize event. */
export interface WindowEventOnResizeData {
  windowId: number;
  width: number;
  height: number;
}

/** Payload for window move event. */
export interface WindowEventOnMoveData {
  windowId: number;
  position: [number, number];
}

/** Payload for window close request event. */
export interface WindowEventOnCloseRequestData {
  windowId: number;
}

/** Payload for window destroy event. */
export interface WindowEventOnDestroyData {
  windowId: number;
}

/** Payload for window focus event. */
export interface WindowEventOnFocusData {
  windowId: number;
  focused: boolean;
}

/** Payload for window scale factor change event. */
export interface WindowEventOnScaleFactorChangeData {
  windowId: number;
  scaleFactor: number;
  newWidth: number;
  newHeight: number;
}

/** Payload for window occlusion event. */
export interface WindowEventOnOccludeData {
  windowId: number;
  occluded: boolean;
}

/** Payload for window redraw request event. */
export interface WindowEventOnRedrawRequestData {
  windowId: number;
}

/** Payload for file drop event. */
export interface WindowEventOnFileDropData {
  windowId: number;
  path: string;
  position: [number, number];
}

/** Payload for file hover event. */
export interface WindowEventOnFileHoverData {
  windowId: number;
  path: string;
  position: [number, number];
}

/** Payload for file hover cancel event. */
export interface WindowEventOnFileHoverCancelData {
  windowId: number;
}

/** Payload for theme change event. */
export interface WindowEventOnThemeChangeData {
  windowId: number;
  darkMode: boolean;
}

/** Payload for lifecycle state change event. */
export interface WindowEventOnStateChangeData {
  windowId: number;
  state: WindowState;
}

/** Pointer capture snapshot from window runtime. */
export interface WindowPointerCaptureState {
  mode: CursorGrabMode;
  active: boolean;
  reason?: string;
}

/** Payload for pointer capture mode/activation updates. */
export interface WindowEventOnPointerCaptureChangeData {
  windowId: number;
  capture: WindowPointerCaptureState;
}

/** Discriminated union of window events. */
export type WindowEvent =
  | { event: 'on-create'; data: WindowEventOnCreateData }
  | { event: 'on-resize'; data: WindowEventOnResizeData }
  | { event: 'on-move'; data: WindowEventOnMoveData }
  | { event: 'on-close-request'; data: WindowEventOnCloseRequestData }
  | { event: 'on-destroy'; data: WindowEventOnDestroyData }
  | { event: 'on-focus'; data: WindowEventOnFocusData }
  | {
      event: 'on-scale-factor-change';
      data: WindowEventOnScaleFactorChangeData;
    }
  | { event: 'on-occlude'; data: WindowEventOnOccludeData }
  | { event: 'on-redraw-request'; data: WindowEventOnRedrawRequestData }
  | { event: 'on-file-drop'; data: WindowEventOnFileDropData }
  | { event: 'on-file-hover'; data: WindowEventOnFileHoverData }
  | { event: 'on-file-hover-cancel'; data: WindowEventOnFileHoverCancelData }
  | { event: 'on-theme-change'; data: WindowEventOnThemeChangeData }
  | { event: 'on-state-change'; data: WindowEventOnStateChangeData }
  | {
      event: 'on-pointer-capture-change';
      data: WindowEventOnPointerCaptureChangeData;
    };
