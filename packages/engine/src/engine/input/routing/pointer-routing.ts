import type { PointerEvent } from '../../../types/events/pointer';
import type { InputStateComponent } from '../../ecs/components';

function resolveTargetSize(data: {
  targetWidth?: number;
  targetHeight?: number;
}): [number, number] | undefined {
  if (
    typeof data.targetWidth === 'number' &&
    typeof data.targetHeight === 'number'
  ) {
    return [data.targetWidth, data.targetHeight];
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
  inputState.pointerTargetDelta = inputState.pointerPositionTarget
    ? [0, 0]
    : undefined;
}

function applyRoutedPointerPosition(
  inputState: InputStateComponent,
  targetPosition?: [number, number],
  targetId?: number,
  targetUv?: [number, number],
  targetSize?: [number, number],
): void {
  if (!targetPosition) {
    clearRoutedPointerState(inputState);
    return;
  }

  const previousTargetId = inputState.pointerTargetId;
  const previousTargetPosition = inputState.pointerPositionTarget;
  if (previousTargetId === targetId && previousTargetPosition !== undefined) {
    inputState.pointerTargetDelta = [
      targetPosition[0] - previousTargetPosition[0],
      targetPosition[1] - previousTargetPosition[1],
    ];
  } else {
    inputState.pointerTargetDelta = [0, 0];
  }

  inputState.pointerPositionTarget = targetPosition;
  inputState.pointerTargetId = targetId;
  inputState.pointerTargetSize = targetSize;
  inputState.pointerTargetUv = targetUv;
}

export function applyRoutedPointerEvent(
  inputState: InputStateComponent,
  pointerEvent: PointerEvent,
): void {
  if (pointerEvent.event === 'on-move') {
    applyRoutedPointerPosition(
      inputState,
      pointerEvent.data.positionTarget,
      pointerEvent.data.trace?.targetId,
      pointerEvent.data.trace?.uv,
      resolveTargetSize(pointerEvent.data),
    );
    return;
  }

  if (pointerEvent.event === 'on-button') {
    applyRoutedPointerPosition(
      inputState,
      pointerEvent.data.positionTarget,
      pointerEvent.data.trace?.targetId,
      pointerEvent.data.trace?.uv,
      resolveTargetSize(pointerEvent.data),
    );
    return;
  }

  if (pointerEvent.event === 'on-touch') {
    applyRoutedPointerPosition(
      inputState,
      pointerEvent.data.positionTarget,
      pointerEvent.data.trace?.targetId,
      pointerEvent.data.trace?.uv,
      resolveTargetSize(pointerEvent.data),
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
