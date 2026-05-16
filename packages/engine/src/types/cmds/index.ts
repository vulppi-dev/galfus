import * as Audio from './audio';
import * as Cam from './camera';
import * as Env from './environment';
import * as Geo from './geometry';
import * as Giz from './gizmo';
import * as Lite from './light';
import * as Mat from './material';
import * as Mod from './model';
import * as Realm from './realm';
import * as RenderGraph from './render-graph';
import * as Sys from './system';
import * as Target from './target';
import * as Tex from './texture';
import * as Ui from './ui';
import * as Win from './window';

export * from './audio';
export * from './camera';
export * from './environment';
export * from './geometry';
export * from './gizmo';
export * from './light';
export * from './material';
export * from './model';
export * from './realm';
export * from './render-graph';
export * from './resources';
export * from './shadow';
export * from './system';
export * from './target';
export * from './texture';
export * from './ui';
export * from './window';

/**
 * Discriminated union of all commands accepted by core.
 *
 * Routing convention:
 * - world-scoped commands are dispatched through the world command queue
 * - global commands are dispatched through the global command queue
 */
export type EngineCmd =
  | { type: 'cmd-notification-send'; content: Sys.CmdNotificationSendArgs }
  | {
      type: 'cmd-system-diagnostics-set';
      content: Sys.CmdSystemDiagnosticsSetArgs;
    }
  | {
      type: 'cmd-system-build-version-get';
      content: Sys.CmdSystemBuildVersionGetArgs;
    }
  | { type: 'cmd-window-create'; content: Win.CmdWindowCreateArgs }
  | { type: 'cmd-window-close'; content: Win.CmdWindowCloseArgs }
  | { type: 'cmd-window-measurement'; content: Win.CmdWindowMeasurementArgs }
  | { type: 'cmd-window-cursor'; content: Win.CmdWindowCursorArgs }
  | { type: 'cmd-window-state'; content: Win.CmdWindowStateArgs }
  | {
      type: 'cmd-upload-buffer-discard-all';
      content: Sys.CmdUploadBufferDiscardAllArgs;
    }
  | { type: 'cmd-camera-upsert'; content: Cam.CmdCameraUpsertArgs }
  | { type: 'cmd-camera-dispose'; content: Cam.CmdCameraDisposeArgs }
  | { type: 'cmd-model-upsert'; content: Mod.CmdModelUpsertArgs }
  | { type: 'cmd-pose-update'; content: Mod.CmdPoseUpdateArgs }
  | { type: 'cmd-model-dispose'; content: Mod.CmdModelDisposeArgs }
  | { type: 'cmd-light-upsert'; content: Lite.CmdLightUpsertArgs }
  | { type: 'cmd-light-dispose'; content: Lite.CmdLightDisposeArgs }
  | { type: 'cmd-material-upsert'; content: Mat.CmdMaterialUpsertArgs }
  | { type: 'cmd-material-dispose'; content: Mat.CmdMaterialDisposeArgs }
  | {
      type: 'cmd-texture-create-from-buffer';
      content: Tex.CmdTextureCreateFromBufferArgs;
    }
  | {
      type: 'cmd-texture-create-solid-color';
      content: Tex.CmdTextureCreateSolidColorArgs;
    }
  | { type: 'cmd-texture-dispose'; content: Tex.CmdTextureDisposeArgs }
  | { type: 'cmd-texture-bind-target'; content: Tex.CmdTextureBindTargetArgs }
  | {
      type: 'cmd-audio-listener-upsert';
      content: Audio.CmdAudioListenerUpsertArgs;
    }
  | {
      type: 'cmd-audio-listener-dispose';
      content: Audio.CmdAudioListenerDisposeArgs;
    }
  | {
      type: 'cmd-audio-source-upsert';
      content: Audio.CmdAudioSourceUpsertArgs;
    }
  | {
      type: 'cmd-audio-resource-upsert';
      content: Audio.CmdAudioResourceUpsertArgs;
    }
  | {
      type: 'cmd-audio-source-transport';
      content: Audio.CmdAudioSourceTransportArgs;
    }
  | { type: 'cmd-audio-state-get'; content: Audio.CmdAudioStateGetArgs }
  | {
      type: 'cmd-audio-source-dispose';
      content: Audio.CmdAudioSourceDisposeArgs;
    }
  | {
      type: 'cmd-audio-resource-dispose';
      content: Audio.CmdAudioResourceDisposeArgs;
    }
  | { type: 'cmd-geometry-upsert'; content: Geo.CmdGeometryUpsertArgs }
  | { type: 'cmd-geometry-dispose'; content: Geo.CmdGeometryDisposeArgs }
  | {
      type: 'cmd-primitive-geometry-create';
      content: Geo.CmdPrimitiveGeometryCreateArgs;
    }
  | { type: 'cmd-environment-upsert'; content: Env.CmdEnvironmentUpsertArgs }
  | {
      type: 'cmd-environment-dispose';
      content: Env.CmdEnvironmentDisposeArgs;
    }
  | { type: 'cmd-shadow-configure'; content: import('./shadow').CmdShadowConfigureArgs }
  | { type: 'cmd-realm-create'; content: Realm.CmdRealmCreateArgs }
  | { type: 'cmd-realm-dispose'; content: Realm.CmdRealmDisposeArgs }
  | {
      type: 'cmd-render-graph-upsert';
      content: RenderGraph.CmdRenderGraphUpsertArgs;
    }
  | {
      type: 'cmd-render-graph-dispose';
      content: RenderGraph.CmdRenderGraphDisposeArgs;
    }
  | {
      type: 'cmd-render-graph-list';
      content: RenderGraph.CmdRenderGraphListArgs;
    }
  | {
      type: 'cmd-realm-render-graph-bind';
      content: RenderGraph.CmdRealmRenderGraphBindArgs;
    }
  | { type: 'cmd-target-upsert'; content: Target.CmdTargetUpsertArgs }
  | {
      type: 'cmd-target-measurement';
      content: Target.CmdTargetMeasurementArgs;
    }
  | { type: 'cmd-target-dispose'; content: Target.CmdTargetDisposeArgs }
  | {
      type: 'cmd-target-layer-upsert';
      content: Target.CmdTargetLayerUpsertArgs;
    }
  | {
      type: 'cmd-target-layer-dispose';
      content: Target.CmdTargetLayerDisposeArgs;
    }
  | { type: 'cmd-ui-theme-define'; content: Ui.CmdUiThemeDefineArgs }
  | { type: 'cmd-ui-theme-dispose'; content: Ui.CmdUiThemeDisposeArgs }
  | { type: 'cmd-ui-document-create'; content: Ui.CmdUiDocumentCreateArgs }
  | { type: 'cmd-ui-document-dispose'; content: Ui.CmdUiDocumentDisposeArgs }
  | { type: 'cmd-ui-document-set-rect'; content: Ui.CmdUiDocumentSetRectArgs }
  | { type: 'cmd-ui-document-set-theme'; content: Ui.CmdUiDocumentSetThemeArgs }
  | { type: 'cmd-ui-document-get-tree'; content: Ui.CmdUiDocumentGetTreeArgs }
  | {
      type: 'cmd-ui-document-get-layout-rects';
      content: Ui.CmdUiDocumentGetLayoutRectsArgs;
    }
  | { type: 'cmd-ui-apply-ops'; content: Ui.CmdUiApplyOpsArgs }
  | { type: 'cmd-ui-debug-set'; content: Ui.CmdUiDebugSetArgs }
  | { type: 'cmd-ui-focus-set'; content: Ui.CmdUiFocusSetArgs }
  | { type: 'cmd-ui-focus-get'; content: Ui.CmdUiFocusGetArgs }
  | { type: 'cmd-ui-event-trace-set'; content: Ui.CmdUiEventTraceSetArgs }
  | {
      type: 'cmd-ui-image-create-from-buffer';
      content: Ui.CmdUiImageCreateFromBufferArgs;
    }
  | { type: 'cmd-ui-image-dispose'; content: Ui.CmdUiImageDisposeArgs }
  | { type: 'cmd-ui-clipboard-paste'; content: Ui.CmdUiClipboardPasteArgs }
  | { type: 'cmd-ui-screenshot-reply'; content: Ui.CmdUiScreenshotReplyArgs }
  | {
      type: 'cmd-ui-access-kit-action-request';
      content: Ui.CmdUiAccessKitActionRequestArgs;
    }
  | { type: 'cmd-model-list'; content: Mod.CmdModelListArgs }
  | { type: 'cmd-material-list'; content: Mat.CmdMaterialListArgs }
  | { type: 'cmd-texture-list'; content: Tex.CmdTextureListArgs }
  | { type: 'cmd-geometry-list'; content: Geo.CmdGeometryListArgs }
  | { type: 'cmd-light-list'; content: Lite.CmdLightListArgs }
  | { type: 'cmd-camera-list'; content: Cam.CmdCameraListArgs }
  | { type: 'cmd-gizmo-draw-line'; content: Giz.CmdGizmoDrawLineArgs }
  | { type: 'cmd-gizmo-draw-aabb'; content: Giz.CmdGizmoDrawAabbArgs }
  | { type: 'cmd-gizmo-draw-polyline'; content: Giz.CmdGizmoDrawPolylineArgs };

/** Discriminated union of all command responses returned by core. */
export type CommandResponse =
  | { type: 'notification-send'; content: Sys.CmdResultNotificationSend }
  | {
      type: 'system-diagnostics-set';
      content: Sys.CmdResultSystemDiagnosticsSet;
    }
  | {
      type: 'system-build-version-get';
      content: Sys.CmdResultSystemBuildVersionGet;
    }
  | { type: 'window-create'; content: Win.CmdResultWindowCreate }
  | { type: 'window-close'; content: Win.CmdResultWindowClose }
  | { type: 'window-measurement'; content: Win.CmdResultWindowMeasurement }
  | { type: 'window-cursor'; content: Win.CmdResultWindowCursor }
  | { type: 'window-state'; content: Win.CmdResultWindowState }
  | {
      type: 'upload-buffer-discard-all';
      content: Sys.CmdResultUploadBufferDiscardAll;
    }
  | { type: 'camera-upsert'; content: Cam.CmdResultCameraUpsert }
  | { type: 'camera-dispose'; content: Cam.CmdResultCameraDispose }
  | { type: 'model-upsert'; content: Mod.CmdResultModelUpsert }
  | { type: 'pose-update'; content: Mod.CmdResultPoseUpdate }
  | { type: 'model-dispose'; content: Mod.CmdResultModelDispose }
  | { type: 'light-upsert'; content: Lite.CmdResultLightUpsert }
  | { type: 'light-dispose'; content: Lite.CmdResultLightDispose }
  | { type: 'material-upsert'; content: Mat.CmdResultMaterialUpsert }
  | { type: 'material-dispose'; content: Mat.CmdResultMaterialDispose }
  | {
      type: 'texture-create-from-buffer';
      content: Tex.CmdResultTextureCreateFromBuffer;
    }
  | {
      type: 'texture-create-solid-color';
      content: Tex.CmdResultTextureCreateSolidColor;
    }
  | { type: 'texture-dispose'; content: Tex.CmdResultTextureDispose }
  | { type: 'texture-bind-target'; content: Tex.CmdResultTextureBindTarget }
  | {
      type: 'audio-listener-upsert';
      content: Audio.CmdResultAudioListenerUpsert;
    }
  | {
      type: 'audio-listener-dispose';
      content: Audio.CmdResultAudioListenerDispose;
    }
  | {
      type: 'audio-source-upsert';
      content: Audio.CmdResultAudioSourceUpsert;
    }
  | {
      type: 'audio-resource-upsert';
      content: Audio.CmdResultAudioResourceUpsert;
    }
  | {
      type: 'audio-source-transport';
      content: Audio.CmdResultAudioSourceTransport;
    }
  | { type: 'audio-state-get'; content: Audio.CmdResultAudioStateGet }
  | {
      type: 'audio-source-dispose';
      content: Audio.CmdResultAudioSourceDispose;
    }
  | {
      type: 'audio-resource-dispose';
      content: Audio.CmdResultAudioResourceDispose;
    }
  | { type: 'geometry-upsert'; content: Geo.CmdResultGeometryUpsert }
  | { type: 'geometry-dispose'; content: Geo.CmdResultGeometryDispose }
  | {
      type: 'primitive-geometry-create';
      content: Geo.CmdResultPrimitiveGeometryCreate;
    }
  | { type: 'environment-upsert'; content: Env.CmdResultEnvironment }
  | { type: 'environment-dispose'; content: Env.CmdResultEnvironment }
  | {
      type: 'shadow-configure';
      content: import('./shadow').CmdResultShadowConfigure;
    }
  | { type: 'realm-create'; content: Realm.CmdResultRealmCreate }
  | { type: 'realm-dispose'; content: Realm.CmdResultRealmDispose }
  | {
      type: 'render-graph-upsert';
      content: RenderGraph.CmdResultRenderGraphUpsert;
    }
  | {
      type: 'render-graph-dispose';
      content: RenderGraph.CmdResultRenderGraphDispose;
    }
  | {
      type: 'render-graph-list';
      content: RenderGraph.CmdResultRenderGraphList;
    }
  | {
      type: 'realm-render-graph-bind';
      content: RenderGraph.CmdResultRealmRenderGraphBind;
    }
  | { type: 'target-upsert'; content: Target.CmdResultTargetUpsert }
  | {
      type: 'target-measurement';
      content: Target.CmdResultTargetMeasurement;
    }
  | { type: 'target-dispose'; content: Target.CmdResultTargetDispose }
  | {
      type: 'target-layer-upsert';
      content: Target.CmdResultTargetLayerUpsert;
    }
  | {
      type: 'target-layer-dispose';
      content: Target.CmdResultTargetLayerDispose;
    }
  | { type: 'ui-theme-define'; content: Ui.CmdResultUiThemeDefine }
  | { type: 'ui-theme-dispose'; content: Ui.CmdResultUiThemeDispose }
  | { type: 'ui-document-create'; content: Ui.CmdResultUiDocumentCreate }
  | { type: 'ui-document-dispose'; content: Ui.CmdResultUiDocumentDispose }
  | { type: 'ui-document-set-rect'; content: Ui.CmdResultUiDocumentSetRect }
  | { type: 'ui-document-set-theme'; content: Ui.CmdResultUiDocumentSetTheme }
  | { type: 'ui-document-get-tree'; content: Ui.CmdResultUiDocumentGetTree }
  | {
      type: 'ui-document-get-layout-rects';
      content: Ui.CmdResultUiDocumentGetLayoutRects;
    }
  | { type: 'ui-apply-ops'; content: Ui.CmdResultUiApplyOps }
  | { type: 'ui-debug-set'; content: Ui.CmdResultUiDebugSet }
  | { type: 'ui-focus-set'; content: Ui.CmdResultUiFocusSet }
  | { type: 'ui-focus-get'; content: Ui.CmdResultUiFocusGet }
  | { type: 'ui-event-trace-set'; content: Ui.CmdResultUiEventTraceSet }
  | {
      type: 'ui-image-create-from-buffer';
      content: Ui.CmdResultUiImageCreateFromBuffer;
    }
  | { type: 'ui-image-dispose'; content: Ui.CmdResultUiImageDispose }
  | { type: 'ui-clipboard-paste'; content: Ui.CmdResultUiInputEvent }
  | { type: 'ui-screenshot-reply'; content: Ui.CmdResultUiInputEvent }
  | { type: 'ui-access-kit-action-request'; content: Ui.CmdResultUiInputEvent }
  | { type: 'model-list'; content: Mod.CmdResultModelList }
  | { type: 'material-list'; content: Mat.CmdResultMaterialList }
  | { type: 'texture-list'; content: Tex.CmdResultTextureList }
  | { type: 'geometry-list'; content: Geo.CmdResultGeometryList }
  | { type: 'light-list'; content: Lite.CmdResultLightList }
  | { type: 'camera-list'; content: Cam.CmdResultCameraList }
  | { type: 'gizmo-draw-line'; content: Giz.CmdResultGizmoDraw }
  | { type: 'gizmo-draw-aabb'; content: Giz.CmdResultGizmoDraw }
  | { type: 'gizmo-draw-polyline'; content: Giz.CmdResultGizmoDraw };

/** Command envelope used in batched transport payloads. */
export interface EngineCmdEnvelope {
  id: number;
  type: EngineCmd['type'];
  content: EngineCmd['content'];
}

/** Response envelope returned by queue polling and routed by command id. */
export interface CommandResponseEnvelope {
  id: number;
  type: CommandResponse['type'];
  content: CommandResponse['content'];
}

/** Batched commands payload sent to transport. */
export type EngineBatchCmds = EngineCmdEnvelope[];
/** Batched responses payload decoded from transport. */
export type EngineBatchResponses = CommandResponseEnvelope[];
