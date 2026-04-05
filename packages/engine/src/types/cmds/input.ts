export interface CmdInputTargetListenerUpsertArgs {
  listenerId: number;
  targetId: number;
  enabled?: boolean;
  events?: string[];
  samplePercent?: number;
}

export interface CmdResultInputTargetListenerUpsert {
  success: boolean;
  message: string;
}

export interface CmdInputTargetListenerDisposeArgs {
  listenerId: number;
}

export interface CmdResultInputTargetListenerDispose {
  success: boolean;
  message: string;
}

export interface CmdInputTargetListenerListArgs {
  targetId?: number;
}

export interface InputTargetListenerSnapshot {
  listenerId: number;
  targetId: number;
  enabled: boolean;
  events: string[];
  samplePercent: number;
}

export interface CmdResultInputTargetListenerList {
  success: boolean;
  message: string;
  listeners: InputTargetListenerSnapshot[];
}
