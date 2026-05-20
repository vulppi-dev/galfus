import type { LogLevel, NotificationLevel } from '../kinds';

export type ProfilingDetailLevel = 'basic' | 'full';
export type SystemPointerTraceLevel = 'off' | 'errors' | 'basic' | 'full';

/** Command payload for sending a notification. */
export interface CmdNotificationSendArgs {
  id?: string;
  title: string;
  body: string;
  level: NotificationLevel;
  timeout?: number;
}

/** Result payload for notification send. */
export interface CmdResultNotificationSend {
  success: boolean;
}

/** Command payload for runtime diagnostics and tracing controls. */
export interface CmdSystemDiagnosticsSetArgs {
  profilingEnabled?: boolean;
  profilingDetail?: ProfilingDetailLevel;
  profilingSamplingPercent?: number;
  profilingWindowFrames?: number;
  traceLevel?: SystemPointerTraceLevel;
  traceSamplingPercent?: number;
}

/** Result payload for diagnostics update. */
export interface CmdResultSystemDiagnosticsSet {
  success: boolean;
  message: string;
}

/** Command payload for updating core log filter level. */
export interface CmdSystemLogLevelSetArgs {
  level: LogLevel;
}

/** Result payload for log level update. */
export interface CmdResultSystemLogLevelSet {
  success: boolean;
  message: string;
  currentLevel: LogLevel;
}

/** Command payload for retrieving core log filter level. */
export interface CmdSystemLogLevelGetArgs {}

/** Result payload for log level query. */
export interface CmdResultSystemLogLevelGet {
  success: boolean;
  message: string;
  currentLevel: LogLevel;
}

/** Command payload for retrieving core build version. */
export interface CmdSystemBuildVersionGetArgs {}

/** Result payload for build version query. */
export interface CmdResultSystemBuildVersionGet {
  success: boolean;
  message: string;
  buildVersion: string;
}

/** Command payload for discarding all pending upload buffers. */
export interface CmdUploadBufferDiscardAllArgs {}

/** Result payload for upload discard all. */
export interface CmdResultUploadBufferDiscardAll {
  success: boolean;
  discardedCount: number;
  message: string;
}
