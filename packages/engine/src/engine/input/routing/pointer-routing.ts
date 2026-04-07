import { vec2, type vec2 as Vec2 } from 'gl-matrix';
import type { PointerEvent } from '../../../types/events/pointer';
import type { InputStateComponent } from '../../ecs/components';

function resolveTargetSize(data: {
  targetWidth?: number;
  targetHeight?: number;
}): Vec2 | undefined {
  if (typeof data.targetWidth === 'number' && typeof data.targetHeight === 'number') {
    return vec2.fromValues(data.targetWidth, data.targetHeight);
  }
  return undefined;
}

export function clearRoutedPointerState(inputState: InputStateComponent): void {
  inputState.pointerPositionTarget = undefined;
  inputState.pointerTargetDelta = undefined;
  inputState.pointerTargetId = undefined;
  inputState.pointerTargetSize = undefined;
  inputState.pointerTargetUv = undefined;
}

export function resetRoutedPointerFrame(inputState: InputStateComponent): void {
  inputState.pointerTargetDelta = inputState.pointerPositionTarget ? vec2.create() : undefined;
}

function applyRoutedPointerPosition(
  inputState: InputStateComponent,
  targetPosition?: Vec2,
  targetId?: number,
  targetUv?: Vec2,
  targetSize?: Vec2
): void {
  if (!targetPosition) {
    clearRoutedPointerState(inputState);
    return;
  }

  const previousTargetId = inputState.pointerTargetId;
  const previousTargetPosition = inputState.pointerPositionTarget;
  if (previousTargetId === targetId && previousTargetPosition !== undefined) {
    inputState.pointerTargetDelta = vec2.fromValues(
      targetPosition[0] - previousTargetPosition[0],
      targetPosition[1] - previousTargetPosition[1]
    );
  } else {
    inputState.pointerTargetDelta = vec2.create();
  }

  inputState.pointerPositionTarget = targetPosition;
  inputState.pointerTargetId = targetId;
  inputState.pointerTargetSize = targetSize;
  inputState.pointerTargetUv = targetUv;
}

export function applyRoutedPointerEvent(
  inputState: InputStateComponent,
  pointerEvent: PointerEvent
): void {
  if (pointerEvent.event === 'on-move') {
    applyRoutedPointerPosition(
      inputState,
      pointerEvent.data.positionTarget,
      pointerEvent.data.trace?.targetId,
      pointerEvent.data.trace?.uv,
      resolveTargetSize(pointerEvent.data)
    );
    return;
  }

  if (pointerEvent.event === 'on-button') {
    applyRoutedPointerPosition(
      inputState,
      pointerEvent.data.positionTarget,
      pointerEvent.data.trace?.targetId,
      pointerEvent.data.trace?.uv,
      resolveTargetSize(pointerEvent.data)
    );
    return;
  }

  if (pointerEvent.event === 'on-touch') {
    applyRoutedPointerPosition(
      inputState,
      pointerEvent.data.positionTarget,
      pointerEvent.data.trace?.targetId,
      pointerEvent.data.trace?.uv,
      resolveTargetSize(pointerEvent.data)
    );
    return;
  }

  if (pointerEvent.event === 'on-scroll') {
    inputState.pointerTargetSize = resolveTargetSize(pointerEvent.data);
    return;
  }

  if (pointerEvent.event === 'on-leave') {
    clearRoutedPointerState(inputState);
    return;
  }

  if (pointerEvent.event === 'on-enter') {
    inputState.pointerTargetSize = resolveTargetSize(pointerEvent.data);
  }
}
