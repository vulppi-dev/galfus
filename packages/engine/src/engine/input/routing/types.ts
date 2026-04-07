import type { vec2 } from 'gl-matrix';
export type RoutedPointerSnapshot = {
  pointerTargetId?: number;
  pointerTargetPosition?: vec2;
  pointerTargetDelta?: vec2;
  pointerTargetUv?: vec2;
  pointerTargetSize?: vec2;
};

export type RoutedPointerReadScope = 'world' | 'target';

export type RoutedPointerFrameSemantics = {
  source: 'mirrored-pointer-event';
  captureStep: 'input';
  captureMoment: 'on-event-application';
  sameTargetContinuityRequiredForDelta: true;
};

export const ROUTED_POINTER_FRAME_SEMANTICS: RoutedPointerFrameSemantics = {
  source: 'mirrored-pointer-event',
  captureStep: 'input',
  captureMoment: 'on-event-application',
  sameTargetContinuityRequiredForDelta: true
};
