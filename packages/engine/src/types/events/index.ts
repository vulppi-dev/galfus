import type { WindowEvent } from './window';
import type { PointerEvent } from './pointer';
import type { KeyboardEvent } from './keyboard';
import type { GamepadEvent } from './gamepad';
import type { SystemEvent } from './system';

export * from './window';
export * from './pointer';
export * from './keyboard';
export * from './gamepad';
export * from './system';

/** Discriminated union of all engine events. */
export type EngineEvent =
  | { type: 'window'; content: WindowEvent }
  | { type: 'pointer'; content: PointerEvent }
  | { type: 'keyboard'; content: KeyboardEvent }
  | { type: 'gamepad'; content: GamepadEvent }
  | { type: 'system'; content: SystemEvent };
