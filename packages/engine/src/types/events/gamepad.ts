import type { ElementState } from '../kinds';

/** Payload for gamepad connect event. */
export interface GamepadEventOnConnectData {
  gamepadId: number;
  name: string;
}

/** Payload for gamepad disconnect event. */
export interface GamepadEventOnDisconnectData {
  gamepadId: number;
}

/** Payload for gamepad button event. */
export interface GamepadEventOnButtonData {
  gamepadId: number;
  button: number;
  state: ElementState;
  value: number;
}

/** Payload for gamepad axis event. */
export interface GamepadEventOnAxisData {
  gamepadId: number;
  axis: number;
  value: number;
}

/** Discriminated union of gamepad events. */
export type GamepadEvent =
  | { event: 'on-connect'; data: GamepadEventOnConnectData }
  | { event: 'on-disconnect'; data: GamepadEventOnDisconnectData }
  | { event: 'on-button'; data: GamepadEventOnButtonData }
  | { event: 'on-axis'; data: GamepadEventOnAxisData };
