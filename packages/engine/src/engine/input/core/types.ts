import type {
  GamepadStateComponent,
  InputStateComponent,
  SystemEventStateComponent,
  UiEventStateComponent,
  WindowStateComponent
} from '../../ecs/components';

export type CanonicalInputState = InputStateComponent;
export type CanonicalWindowState = WindowStateComponent;
export type CanonicalGamepadState = GamepadStateComponent;
export type CanonicalSystemEventState = SystemEventStateComponent;
export type CanonicalUiEventState = UiEventStateComponent;

export type InputFrameResetPolicy = {
  clearKeyEdgesAtFrameStart: true;
  clearPointerEdgesAtFrameStart: true;
  resetPointerDeltaAtFrameStart: true;
  resetScrollDeltaAtFrameStart: true;
  resetImeCommitAtFrameStart: true;
  resetWindowTransientFlagsAtFrameStart: true;
  resetEventsThisFrameAtFrameStart: true;
};

export const INPUT_FRAME_RESET_POLICY: InputFrameResetPolicy = {
  clearKeyEdgesAtFrameStart: true,
  clearPointerEdgesAtFrameStart: true,
  resetPointerDeltaAtFrameStart: true,
  resetScrollDeltaAtFrameStart: true,
  resetImeCommitAtFrameStart: true,
  resetWindowTransientFlagsAtFrameStart: true,
  resetEventsThisFrameAtFrameStart: true
};
