export type RoutedPointerSnapshot = {
  pointerTargetId?: number;
  pointerTargetPosition?: [number, number];
  pointerTargetDelta?: [number, number];
  pointerTargetUv?: [number, number];
  pointerTargetSize?: [number, number];
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
  sameTargetContinuityRequiredForDelta: true,
};
