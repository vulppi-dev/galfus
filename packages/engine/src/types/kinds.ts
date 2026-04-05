/** Camera projection mode. */
export type CameraKind = 'perspective' | 'orthographic';

/** Audio playback mode. */
export type AudioPlayMode = 'once' | 'loop';

/** Standard cursor icons. */
export type CursorIcon =
  | 'default'
  | 'context-menu'
  | 'help'
  | 'pointer'
  | 'progress'
  | 'wait'
  | 'cell'
  | 'crosshair'
  | 'text'
  | 'vertical-text'
  | 'alias'
  | 'copy'
  | 'move'
  | 'no-drop'
  | 'not-allowed'
  | 'grab'
  | 'grabbing'
  | 'e-resize'
  | 'n-resize'
  | 'ne-resize'
  | 'nw-resize'
  | 's-resize'
  | 'se-resize'
  | 'sw-resize'
  | 'w-resize'
  | 'ew-resize'
  | 'ns-resize'
  | 'nesw-resize'
  | 'nwse-resize'
  | 'col-resize'
  | 'row-resize'
  | 'all-scroll'
  | 'zoom-in'
  | 'zoom-out';

/** Press state for input events. */
export type ElementState = 'released' | 'pressed';

/** Light source type. */
export type LightKind =
  | 'directional'
  | 'point'
  | 'spot'
  | 'ambient'
  | 'hemisphere';

/** Material shading model. */
export type MaterialKind = 'standard' | 'pbr';

/** Notification severity level. */
export type NotificationLevel = 'info' | 'warning' | 'error' | 'success';

/** Built-in primitive geometry shapes. */
export type PrimitiveShape =
  | 'cube'
  | 'plane'
  | 'sphere'
  | 'cylinder'
  | 'torus'
  | 'pyramid'
  | 'pill';

/** Texture sampler presets. */
export type SamplerMode =
  | 'point-clamp'
  | 'linear-clamp'
  | 'point-repeat'
  | 'linear-repeat';

/** Mesh shading model identifier. */
export type ShadeModel = 'standard' | 'pbr';

/** Texture allocation strategy. */
export type TextureCreateMode = 'standalone' | 'forward-atlas';

/** Touch interaction phase. */
export type TouchPhase = 'started' | 'moved' | 'ended' | 'cancelled';

/** Material transparency mode. */
export type TransparencyMode = 'opaque' | 'masked' | 'transparent';

/** Material primitive topology mode. */
export type PrimitiveTopology = 'point-list' | 'line-list' | 'triangle-list';

/** Material polygon rasterization mode. */
export type PolygonMode = 'fill' | 'line' | 'point';

/** Material render side culling mode. */
export type RenderSide = 'front' | 'back' | 'double-side';

/** Upload payload types for buffer transfers. */
export type UploadType =
  | 'raw'
  | 'shader-source'
  | 'geometry-data'
  | 'vertex-data'
  | 'index-data'
  | 'image-data'
  | 'binary-asset';

/** Skybox rendering mode. */
export type SkyboxMode = 'none' | 'procedural' | 'cubemap';

/** Window state transitions. */
export type WindowState =
  | 'minimized'
  | 'maximized'
  | 'windowed'
  | 'fullscreen'
  | 'windowed-fullscreen';

/** Cursor grab behavior. */
export type CursorGrabMode = 'none' | 'confined' | 'locked';

/** User attention request type. */
export type UserAttentionType = 'critical' | 'informational';
