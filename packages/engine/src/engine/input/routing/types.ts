import type { Vec2 as vec2 } from '../../../math/index';
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
