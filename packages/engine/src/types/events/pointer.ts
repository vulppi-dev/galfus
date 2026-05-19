import type { Vec2 as vec2 } from '../../math/index';
import type { ElementState, TouchPhase } from '../kinds';

export type ScrollDelta = { type: 'line'; value: vec2 } | { type: 'pixel'; value: vec2 };

export type PointerTraceStage =
  | 'root-window'
  | 'capture'
  | 'focus-fallback'
  | 'connector-hit'
  | 'realm-hit'
  | 'hop-forward'
  | 'stop-no-hit'
  | 'stop-cycle'
  | 'stop-step-budget';

export interface PointerTraceHop {
  stage: PointerTraceStage;
  realmId?: number;
  targetId?: number;
  layerRealmId?: number;
  connectorId?: number;
  cameraId?: number;
  uv?: vec2;
}

export interface PointerEventTrace {
  windowId: number;
  realmId: number;
  targetId?: number;
  connectorId?: number;
  sourceRealmId?: number;
  uv?: vec2;
  hops: PointerTraceHop[];
}

export interface PointerEventOnMoveData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  pointerType: number;
  pointerId: number;
  position: vec2;
  positionTarget?: vec2;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnEnterData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  pointerType: number;
  pointerId: number;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnLeaveData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  pointerType: number;
  pointerId: number;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnButtonData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  pointerType: number;
  pointerId: number;
  button: MouseButton;
  state: ElementState;
  position: vec2;
  positionTarget?: vec2;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnScrollData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  delta: ScrollDelta;
  phase: TouchPhase;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnTouchData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  pointerId: number;
  phase: TouchPhase;
  position: vec2;
  positionTarget?: vec2;
  targetWidth?: number;
  targetHeight?: number;
  pressure?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnPinchGestureData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  delta: number;
  phase: TouchPhase;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnPanGestureData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  delta: vec2;
  phase: TouchPhase;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnRotationGestureData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  delta: number;
  phase: TouchPhase;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export interface PointerEventOnDoubleTapGestureData {
  windowId: number;
  windowWidth?: number;
  windowHeight?: number;
  targetWidth?: number;
  targetHeight?: number;
  trace?: PointerEventTrace;
}

export type PointerEvent =
  | { event: 'on-move'; data: PointerEventOnMoveData }
  | { event: 'on-enter'; data: PointerEventOnEnterData }
  | { event: 'on-leave'; data: PointerEventOnLeaveData }
  | { event: 'on-button'; data: PointerEventOnButtonData }
  | { event: 'on-scroll'; data: PointerEventOnScrollData }
  | { event: 'on-touch'; data: PointerEventOnTouchData }
  | { event: 'on-pinch-gesture'; data: PointerEventOnPinchGestureData }
  | { event: 'on-pan-gesture'; data: PointerEventOnPanGestureData }
  | { event: 'on-rotation-gesture'; data: PointerEventOnRotationGestureData }
  | {
      event: 'on-double-tap-gesture';
      data: PointerEventOnDoubleTapGestureData;
    };

export enum MouseButton {
  Left = 0,
  Right = 1,
  Middle = 2,
  Back = 3,
  Forward = 4
}
