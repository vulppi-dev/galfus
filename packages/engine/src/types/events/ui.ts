export type UiEventKind =
  | 'click'
  | 'double-click'
  | 'pressed'
  | 'released'
  | 'hover-enter'
  | 'hover-leave'
  | 'changed'
  | 'change-commit'
  | 'focus'
  | 'blur'
  | 'submit'
  | 'anim-complete';

export interface UiEvent {
  realmId: number;
  documentId: number;
  nodeId: number;
  kind: UiEventKind;
  label?: string;
}
