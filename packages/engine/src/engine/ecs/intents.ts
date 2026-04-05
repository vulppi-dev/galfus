import type {
  CmdUiAccessKitActionRequestArgs,
  CmdUiApplyOpsArgs,
  CmdUiClipboardPasteArgs,
  CmdUiDebugSetArgs,
  CmdUiDocumentCreateArgs,
  CmdUiDocumentDisposeArgs,
  CmdUiDocumentGetLayoutRectsArgs,
  CmdUiDocumentGetTreeArgs,
  CmdUiDocumentSetRectArgs,
  CmdUiDocumentSetThemeArgs,
  CmdUiEventTraceSetArgs,
  CmdUiFocusGetArgs,
  CmdUiFocusSetArgs,
  CmdUiImageCreateFromBufferArgs,
  CmdUiImageDisposeArgs,
  CmdUiScreenshotReplyArgs,
  CmdUiThemeDefineArgs,
  CmdUiThemeDisposeArgs,
} from '../../types/cmds/ui';
import type { EnvironmentConfig } from '../../types/cmds/environment';
import type { ShadowConfig } from '../../types/cmds/shadow';
import type { NotificationLevel } from '../../types/kinds';
import type { JsonObject } from '../../types/json';
import type {
  CameraProps,
  ComponentType,
  GeometryProps,
  LightProps,
  MaterialProps,
  ModelProps,
  TagProps,
  TextureProps,
  TransformProps,
  UiFocusCycleMode,
} from './components';

export type Intent =
  | { type: 'configure-environment'; config: EnvironmentConfig }
  | {
      type: 'request-resource-list';
      resourceType:
        | 'model'
        | 'material'
        | 'texture'
        | 'geometry'
        | 'light'
        | 'camera';
    }
  | { type: 'create-entity'; worldId: number; entityId: number }
  | { type: 'remove-entity'; entityId: number }
  | {
      type: 'update-transform';
      entityId: number;
      props: TransformProps;
    }
  | {
      type: 'attach-camera';
      entityId: number;
      props: CameraProps;
    }
  | {
      type: 'attach-model';
      entityId: number;
      props: ModelProps;
    }
  | {
      type: 'attach-light';
      entityId: number;
      props: LightProps;
    }
  | {
      type: 'attach-tag';
      entityId: number;
      props: TagProps;
    }
  | { type: 'create-material'; resourceId: number; props: MaterialProps }
  | { type: 'create-geometry'; resourceId: number; props: GeometryProps }
  | { type: 'create-texture'; resourceId: number; props: TextureProps }
  | { type: 'dispose-material'; resourceId: number }
  | { type: 'dispose-texture'; resourceId: number }
  | { type: 'dispose-geometry'; resourceId: number }
  | { type: 'detach-component'; entityId: number; componentType: ComponentType }
  | { type: 'set-parent'; entityId: number; parentId: number | null }
  | {
      type: 'send-notification';
      level: NotificationLevel;
      title: string;
      message: string;
    }
  | { type: 'configure-shadows'; config: ShadowConfig }
  | {
      type: 'gizmo-draw-line';
      start: [number, number, number];
      end: [number, number, number];
      color: [number, number, number, number];
      thickness?: number;
    }
  | {
      type: 'gizmo-draw-aabb';
      min: [number, number, number];
      max: [number, number, number];
      color: [number, number, number, number];
      thickness?: number;
    }
  | {
      type: 'gizmo-draw-polyline';
      points: [number, number, number][];
      color: [number, number, number, number];
      closed?: boolean;
      thickness?: number;
    }
  | { type: 'ui-theme-define'; args: CmdUiThemeDefineArgs }
  | { type: 'ui-theme-dispose'; args: CmdUiThemeDisposeArgs }
  | { type: 'ui-document-create'; args: CmdUiDocumentCreateArgs }
  | { type: 'ui-document-dispose'; args: CmdUiDocumentDisposeArgs }
  | { type: 'ui-document-set-rect'; args: CmdUiDocumentSetRectArgs }
  | { type: 'ui-document-set-theme'; args: CmdUiDocumentSetThemeArgs }
  | { type: 'ui-document-get-tree'; args: CmdUiDocumentGetTreeArgs }
  | {
      type: 'ui-document-get-layout-rects';
      args: CmdUiDocumentGetLayoutRectsArgs;
    }
  | { type: 'ui-apply-ops'; args: CmdUiApplyOpsArgs }
  | { type: 'ui-debug-set'; args: CmdUiDebugSetArgs }
  | { type: 'ui-focus-set'; args: CmdUiFocusSetArgs }
  | { type: 'ui-focus-get'; args: CmdUiFocusGetArgs }
  | { type: 'ui-event-trace-set'; args: CmdUiEventTraceSetArgs }
  | { type: 'ui-image-create-from-buffer'; args: CmdUiImageCreateFromBufferArgs }
  | { type: 'ui-image-dispose'; args: CmdUiImageDisposeArgs }
  | { type: 'ui-clipboard-paste'; args: CmdUiClipboardPasteArgs }
  | { type: 'ui-screenshot-reply'; args: CmdUiScreenshotReplyArgs }
  | {
      type: 'ui-access-kit-action-request';
      args: CmdUiAccessKitActionRequestArgs;
    }
  | {
      type: 'ui-form-upsert';
      form: {
        formId: string;
        windowId: number;
        realmId: number;
        documentId: number;
        disabled?: boolean;
        cycleMode?: UiFocusCycleMode;
        activeFieldsetId?: string;
      };
    }
  | { type: 'ui-form-dispose'; formId: string }
  | {
      type: 'ui-fieldset-upsert';
      fieldset: {
        formId: string;
        fieldsetId: string;
        disabled?: boolean;
        legendNodeId?: number;
      };
    }
  | { type: 'ui-fieldset-dispose'; formId: string; fieldsetId: string }
  | {
      type: 'ui-focusable-upsert';
      focusable: {
        formId: string;
        nodeId: number;
        tabIndex?: number;
        fieldsetId?: string;
        disabled?: boolean;
        orderHint?: number;
      };
    }
  | { type: 'ui-focusable-dispose'; nodeId: number }
  | {
      type: 'ui-focus-next';
      windowId: number;
      backwards?: boolean;
      formId?: string;
    }
  | {
      type: 'custom';
      name: string;
      data: JsonObject;
    };

/** Internal World Events. */
export type WorldEvent =
  | { type: 'entity-created'; entityId: number }
  | { type: 'entity-destroyed'; entityId: number }
  | { type: 'component-added'; entityId: number; componentType: ComponentType }
  | {
      type: 'component-removed';
      entityId: number;
      componentType: ComponentType;
    };
