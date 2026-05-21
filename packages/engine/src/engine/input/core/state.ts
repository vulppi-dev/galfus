import { vec2 } from '../../../math/index';
import type {
  GamepadStateComponent,
  InputStateComponent,
  SystemEventStateComponent,
  WindowStateComponent
} from '../../ecs/components';
import type { WorldState } from '../../state';

const WORLD_ENTITY_ID = 0;

function createInputState(): InputStateComponent {
  return {
    type: 'InputState',
    keysPressed: new Set(),
    keysJustPressed: new Set(),
    keysJustReleased: new Set(),
    pointerButtons: new Set(),
    pointerPosition: vec2.create(),
    pointerDelta: vec2.create(),
    pointerJustPressed: new Set(),
    pointerJustReleased: new Set(),
    pointerWindowSize: undefined,
    scrollDelta: vec2.create(),
    imeEnabled: false
  };
}

function createWindowState(): WindowStateComponent {
  return {
    type: 'WindowState',
    focused: true,
    size: vec2.fromValues(800, 600),
    position: vec2.create(),
    scaleFactor: 1.0,
    lifecycleState: undefined,
    pointerCapture: undefined,
    closeRequested: false,
    resizedThisFrame: false,
    movedThisFrame: false,
    focusChangedThisFrame: false
  };
}

function createGamepadState(): GamepadStateComponent {
  return {
    type: 'GamepadState',
    connected: new Map(),
    buttons: new Map(),
    axes: new Map(),
    eventsThisFrame: []
  };
}

function createSystemEventState(): SystemEventStateComponent {
  return {
    type: 'SystemEventState',
    eventsThisFrame: []
  };
}

export type InputMirrorStateStore = {
  inputState: InputStateComponent;
  windowState: WindowStateComponent;
  gamepadState: GamepadStateComponent;
  systemEventState: SystemEventStateComponent;
};

export function ensureInputMirrorState(world: WorldState): InputMirrorStateStore {
  let worldStore = world.components.get(WORLD_ENTITY_ID);
  if (!worldStore) {
    worldStore = new Map();
    world.components.set(WORLD_ENTITY_ID, worldStore);
    world.entities.add(WORLD_ENTITY_ID);
  }

  let inputState = worldStore.get('InputState') as InputStateComponent | undefined;
  if (!inputState) {
    inputState = createInputState();
    worldStore.set('InputState', inputState);
  }

  let windowState = worldStore.get('WindowState') as WindowStateComponent | undefined;
  if (!windowState) {
    windowState = createWindowState();
    worldStore.set('WindowState', windowState);
  }

  let gamepadState = worldStore.get('GamepadState') as GamepadStateComponent | undefined;
  if (!gamepadState) {
    gamepadState = createGamepadState();
    worldStore.set('GamepadState', gamepadState);
  }

  let systemEventState = worldStore.get('SystemEventState') as
    | SystemEventStateComponent
    | undefined;
  if (!systemEventState) {
    systemEventState = createSystemEventState();
    worldStore.set('SystemEventState', systemEventState);
  }

  return {
    inputState,
    windowState,
    gamepadState,
    systemEventState
  };
}

export function resetInputMirrorFrame(state: InputMirrorStateStore): void {
  state.inputState.keysJustPressed.clear();
  state.inputState.keysJustReleased.clear();
  state.inputState.pointerJustPressed.clear();
  state.inputState.pointerJustReleased.clear();
  state.inputState.pointerDelta = vec2.create();
  state.inputState.scrollDelta = vec2.create();
  state.inputState.imeCommitText = undefined;

  state.windowState.resizedThisFrame = false;
  state.windowState.movedThisFrame = false;
  state.windowState.focusChangedThisFrame = false;
  state.windowState.closeRequested = false;

  state.gamepadState.eventsThisFrame.length = 0;
  state.systemEventState.eventsThisFrame.length = 0;
}
