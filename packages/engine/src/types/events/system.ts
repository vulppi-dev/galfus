export type PointerTraceLevel = 'off' | 'errors' | 'basic' | 'full';

export type SystemEvent =
  | {
      event: 'error';
      data: {
        scope: string;
        message: string;
        commandId?: number;
        commandType?: string;
      };
    }
  | {
      event: 'command-deferred';
      data: {
        commandId: number;
        commandType: string;
        attempts: number;
        reason: string;
      };
    }
  | {
      event: 'command-applied';
      data: {
        commandId: number;
        commandType: string;
        attempts: number;
      };
    }
  | {
      event: 'command-dropped';
      data: {
        commandId: number;
        commandType: string;
        attempts: number;
        reason: string;
      };
    }
  | { event: 'on-resume' }
  | { event: 'on-suspend' }
  | { event: 'on-memory-warning' }
  | { event: 'on-exit' }
  | { event: 'on-notification-clicked'; data: { id: string } }
  | { event: 'on-notification-dismissed'; data: { id: string } }
  | {
      event: 'texture-ready';
      data: { windowId: number; textureId: number; success: boolean; message: string };
    }
  | {
      event: 'texture-processing-started';
      data: { windowId: number; textureId: number; totalBytes: number };
    }
  | {
      event: 'texture-processing-progress';
      data: {
        windowId: number;
        textureId: number;
        processedBytes: number;
        totalBytes: number;
      };
    }
  | {
      event: 'texture-processing-finished';
      data: {
        windowId: number;
        textureId: number;
        success: boolean;
        message: string;
        totalBytes: number;
      };
    }
  | {
      event: 'audio-ready';
      data: { resourceId: number; success: boolean; message: string };
    }
  | {
      event: 'audio-stream-progress';
      data: {
        resourceId: number;
        receivedBytes: number;
        totalBytes: number;
        complete: boolean;
      };
    };
