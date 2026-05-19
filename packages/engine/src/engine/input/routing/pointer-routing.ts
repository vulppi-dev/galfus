import type { PointerEvent } from '../../../types/events/pointer';
import type { InputStateComponent } from '../../ecs/components';

export function clearRoutedPointerState(_inputState: InputStateComponent): void {
  // Routed per-target pointer snapshot was removed; global pointer stream only.
}

export function resetRoutedPointerFrame(_inputState: InputStateComponent): void {
  // Routed per-target pointer snapshot was removed; global pointer stream only.
}

export function applyRoutedPointerEvent(
  _inputState: InputStateComponent,
  _pointerEvent: PointerEvent
): void {
  // Routed per-target pointer snapshot was removed; global pointer stream only.
}
