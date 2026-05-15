import type { Quat as quat, Vec2 as vec2, Vec3 as vec3, Vec4 as vec4 } from '../../math/index';
import type { ViewPosition } from '../../types/cmds/camera';
import type { GeometryPrimitiveEntry } from '../../types/cmds/geometry';
import type { MaterialOptions } from '../../types/cmds/material';
import type { ForwardAtlasOptions } from '../../types/cmds/texture';
import type { GamepadEvent, SystemEvent, UiEvent } from '../../types/events';
import type {
  CameraKind,
  CursorGrabMode,
  LightKind,
  MaterialKind,
  TextureCreateMode,
  WindowState
} from '../../types/kinds';
import type { JsonObject, JsonValue } from '../../types/json';

/** Transform component data used to position entities. */
export interface TransformProps {
  position?: vec3;
  rotation?: quat;
  scale?: vec3;
  layerMask?: number;
  visible?: boolean;
}

/** Fully-resolved transform component stored in the ECS. */
export interface TransformComponent extends Required<Omit<TransformProps, never>> {
  type: 'Transform';
}

/** Parent link data used for hierarchical transforms. */
export interface ParentProps {
  parentId: number;
}

/** Parent component stored in the ECS. */
export interface ParentComponent extends ParentProps {
  type: 'Parent';
}

/** Camera component configuration. */
export interface CameraProps {
  kind?: CameraKind;
  near?: number;
  far?: number;
  order?: number;
  viewPosition?: ViewPosition;
  orthoScale?: number;
}

/** Camera component stored in the ECS. */
export interface CameraComponent extends Required<Omit<CameraProps, 'viewPosition'>> {
  type: 'Camera';
  id: number;
  viewPosition?: ViewPosition;
  skipUpdate?: boolean;
}

/** Light component configuration. */
export interface LightProps {
  kind?: LightKind;
  color?: vec3;
  intensity?: number;
  range?: number;
  castShadow?: boolean;
  direction?: vec3;
  spotInnerOuter?: vec2;
}

/** Light component stored in the ECS. */
export interface LightComponent extends Required<Omit<LightProps, never>> {
  type: 'Light';
  id: number;
  skipUpdate?: boolean;
}

/** Model component configuration. */
export interface ModelProps {
  geometryId: number;
  materialId?: number;
  castShadow?: boolean;
  receiveShadow?: boolean;
  castOutline?: boolean;
  outlineColor?: vec4;
}

/** Model component stored in the ECS. */
export interface ModelComponent extends Required<Omit<ModelProps, 'materialId'>> {
  type: 'Model';
  id: number;
  materialId?: number;
  skipUpdate?: boolean;
}

/** Tag component configuration. */
export interface TagProps {
  name?: string;
  labels?: string[];
}

/** Tag component stored in the ECS. */
export interface TagComponent {
  type: 'Tag';
  name: string;
  labels: Set<string>;
}

/** InputState: Stores the current input state for a world. */
export interface InputStateComponent {
  type: 'InputState';
  keysPressed: Set<number>;
  keysJustPressed: Set<number>;
  keysJustReleased: Set<number>;
  pointerButtons: Set<number>;
  pointerPosition: vec2;
  pointerDelta: vec2;
  pointerJustPressed: Set<number>;
  pointerJustReleased: Set<number>;
  pointerWindowId?: number;
  pointerWindowSize?: vec2;
  scrollDelta: vec2;
  imeEnabled: boolean;
  imePreeditText?: string;
  imeCursorRange?: vec2;
  imeCommitText?: string;
}

/** WindowState: Stores window events and state. */
export interface WindowStateComponent {
  type: 'WindowState';
  focused: boolean;
  size: vec2;
  position: vec2;
  scaleFactor: number;
  lifecycleState?: WindowState;
  pointerCapture?: {
    mode: CursorGrabMode;
    active: boolean;
    reason?: string;
  };
  closeRequested: boolean;
  resizedThisFrame: boolean;
  movedThisFrame: boolean;
  focusChangedThisFrame: boolean;
}

/** Gamepad state mirrored from core gamepad events. */
export interface GamepadStateComponent {
  type: 'GamepadState';
  connected: Map<number, { name: string }>;
  buttons: Map<number, Map<number, { pressed: boolean; value: number }>>;
  axes: Map<number, Map<number, number>>;
  eventsThisFrame: GamepadEvent[];
}

/** System events mirrored for host-level integrations. */
export interface SystemEventStateComponent {
  type: 'SystemEventState';
  eventsThisFrame: SystemEvent[];
  lastError?: {
    scope: string;
    message: string;
    commandId?: number;
    commandType?: string;
  };
}

/** UI events mirrored from core UI event channel. */
export interface UiEventStateComponent {
  type: 'UiEventState';
  eventsThisFrame: UiEvent[];
}

/** Focus traversal behavior when reaching first/last input. */
export type UiFocusCycleMode = 'wrap' | 'clamp';

/** Form scope state used to control focus traversal. */
export interface UiFormScope {
  formId: string;
  windowId: number;
  realmId: number;
  documentId: number;
  disabled: boolean;
  cycleMode: UiFocusCycleMode;
  activeFieldsetId?: string;
  activeNodeId?: number;
}

/** Fieldset scope metadata. */
export interface UiFieldsetScope {
  formId: string;
  fieldsetId: string;
  disabled: boolean;
  legendNodeId?: number;
}

/** Focusable node metadata for tab ordering. */
export interface UiFocusableNode {
  formId: string;
  nodeId: number;
  tabIndex: number;
  fieldsetId?: string;
  disabled: boolean;
  orderHint: number;
}

/** UI runtime state mirrored in the ECS world. */
export interface UiStateComponent {
  type: 'UiState';
  forms: Map<string, UiFormScope>;
  fieldsets: Map<string, UiFieldsetScope>;
  nodes: Map<number, UiFocusableNode>;
  focusByWindow: Map<number, { formId: string; nodeId: number }>;
}

/** Base properties shared by resources. */
export interface BaseResourceProps {
  label?: string;
}

/** Material resource configuration. */
export interface MaterialProps extends BaseResourceProps {
  kind: MaterialKind;
  options: MaterialOptions;
}

/** Options for cube primitive geometry. */
export interface CubeOptions {
  size?: vec3;
}

/** Options for plane primitive geometry. */
export interface PlaneOptions {
  size?: vec3;
  subdivisions?: number;
}

/** Options for sphere primitive geometry. */
export interface SphereOptions {
  radius?: number;
  sectors?: number;
  stacks?: number;
}

/** Options for cylinder primitive geometry. */
export interface CylinderOptions {
  radius?: number;
  height?: number;
  sectors?: number;
  segments?: number;
}

/** Options for torus primitive geometry. */
export interface TorusOptions {
  majorRadius?: number;
  minorRadius?: number;
  majorSegments?: number;
  minorSegments?: number;
  radialSegments?: number;
  tubularSegments?: number;
}

/** Options for pyramid primitive geometry. */
export interface PyramidOptions {
  size?: vec3;
  subdivisions?: number;
}

/** Options for pill primitive geometry. */
export interface PillOptions {
  radius?: number;
  height?: number;
  sectors?: number;
  stacks?: number;
}

/** Geometry resource configuration (primitive or custom). */
export type GeometryProps = BaseResourceProps &
  (
    | {
        type: 'custom';
        entries: GeometryPrimitiveEntry[];
      }
    | ({ type: 'primitive' } & (
        | { shape: 'cube'; options?: CubeOptions }
        | { shape: 'plane'; options?: PlaneOptions }
        | { shape: 'sphere'; options?: SphereOptions }
        | { shape: 'cylinder'; options?: CylinderOptions }
        | { shape: 'torus'; options?: TorusOptions }
        | { shape: 'pyramid'; options?: PyramidOptions }
        | { shape: 'pill'; options?: PillOptions }
      ))
  );

/** Texture resource configuration. */
export interface TextureProps extends BaseResourceProps {
  srgb?: boolean;
  mode?: TextureCreateMode;
  atlasOptions?: ForwardAtlasOptions;
  source: { type: 'buffer'; bufferId: number } | { type: 'color'; color: vec4 };
}

/** Custom component payload stored in the ECS. */
export interface CustomComponent {
  type: string;
  data: JsonObject;
}

/** Union of all built-in component types. */
export type Component =
  | TransformComponent
  | ParentComponent
  | CameraComponent
  | LightComponent
  | ModelComponent
  | TagComponent
  | InputStateComponent
  | WindowStateComponent
  | GamepadStateComponent
  | SystemEventStateComponent
  | UiEventStateComponent
  | UiStateComponent
  | CustomComponent;

/** String literal type for component identifiers. */
export type ComponentType = Component['type'];

/** Supported property types for custom schemas. */
export type PropertyType =
  | 'number'
  | 'string'
  | 'boolean'
  | 'vec2'
  | 'vec3'
  | 'vec4'
  | 'quat'
  | 'entity'
  | 'resource'
  | 'array'
  | 'object';

/** Schema entry describing a custom component field. */
export interface SchemaProperty {
  type: PropertyType;
  default?: JsonValue;
  optional?: boolean;
}

/** Schema definition for a custom component. */
export interface ComponentSchema {
  [key: string]: SchemaProperty;
}
