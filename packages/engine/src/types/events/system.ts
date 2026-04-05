import type { CursorGrabMode } from '../kinds';

export type PointerTraceLevel = 'off' | 'errors' | 'basic' | 'full';

export type UiViewportClass = 'root' | 'deferred' | 'immediate' | 'embedded';

export type UiViewportCommand =
  | { type: 'close' }
  | { type: 'title'; content: { title: string } }
  | { type: 'inner-size'; content: { width: number; height: number } }
  | { type: 'outer-position'; content: { x: number; y: number } }
  | { type: 'resizable'; content: { value: boolean } }
  | { type: 'decorations'; content: { value: boolean } }
  | { type: 'fullscreen'; content: { value: boolean } }
  | { type: 'minimized'; content: { value: boolean } }
  | { type: 'maximized'; content: { value: boolean } }
  | { type: 'focus' }
  | { type: 'screenshot' }
  | { type: 'cursor-visible'; content: { value: boolean } }
  | { type: 'cursor-grab'; content: { mode: CursorGrabMode | string } }
  | { type: 'ime-allowed'; content: { value: boolean } }
  | {
      type: 'ime-rect';
      content: { minX: number; minY: number; maxX: number; maxY: number };
    };

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
      event: 'ui-image-ready';
      data: { imageId: number; success: boolean; message: string };
    }
  | {
      event: 'ui-image-processing-started';
      data: { imageId: number; totalBytes: number };
    }
  | {
      event: 'ui-image-processing-progress';
      data: { imageId: number; processedBytes: number; totalBytes: number };
    }
  | {
      event: 'ui-image-processing-finished';
      data: {
        imageId: number;
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
    }
  | {
      event: 'ui-open-url';
      data: { windowId: number; realmId: number; url: string; newTab: boolean };
    }
  | {
      event: 'ui-clipboard-set-text';
      data: { windowId: number; realmId: number; text: string };
    }
  | {
      event: 'ui-clipboard-request-copy';
      data: { windowId: number; realmId: number };
    }
  | {
      event: 'ui-clipboard-request-cut';
      data: { windowId: number; realmId: number };
    }
  | {
      event: 'ui-clipboard-request-paste';
      data: { windowId: number; realmId: number };
    }
  | {
      event: 'ui-screenshot-request';
      data: { windowId: number; realmId: number };
    }
  | {
      event: 'ui-viewport-sync';
      data: {
        windowId: number;
        realmId: number;
        viewportId: number;
        parentViewportId?: number;
        class: UiViewportClass;
        title?: string;
      };
    }
  | {
      event: 'ui-viewport-command';
      data: {
        windowId: number;
        realmId: number;
        viewportId: number;
        command: UiViewportCommand;
      };
    }
  | {
      event: 'ui-viewport-fallback-embedded';
      data: {
        windowId: number;
        realmId: number;
        viewportId: number;
        parentViewportId?: number;
      };
    }
  | {
      event: 'input-target-listener-event';
      data: {
        listenerId: number;
        targetId: number;
        eventType: string;
        windowId?: number;
        windowWidth?: number;
        windowHeight?: number;
        pointerId?: number;
        positionGlobal?: [number, number];
        positionTarget?: [number, number];
        targetWidth?: number;
        targetHeight?: number;
        keyCode?: number;
        keyState?: 'pressed' | 'released';
      };
    };
