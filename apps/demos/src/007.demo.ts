import {
  Mount,
  World3D,
  WorldUI,
  closeWindow,
  createWindow,
  disposeEngine,
  focusWindow,
  initEngine,
  tick,
  updateWindow
} from '@vulfram/engine';
import {
  createFirstPersonController,
  createOrbitController,
  createSpectatorController,
  createThirdPersonController,
  createTopViewController,
  type BaseCameraControllerHandle,
  type MotionCameraControllerHandle,
  type OrbitControllerHandle,
  type ThirdPersonControllerHandle,
  type TopViewControllerHandle
} from '@vulfram/camera-control';
import { transportBunFfi } from '@vulfram/transport-bun';
import { vec3 } from '@vulfram/engine/math';

const FRAME_TARGET_MS = 16;
const POINTER_DEBUG_INTERVAL_MS = 250;

const UI_DOCUMENT_ID = 7001;
const UI_NODE_AREA_ID = 7002;
const UI_NODE_COMBO_ID = 7003;
const UI_NODE_TITLE_ID = 7004;
const UI_NODE_POINTER_DELTA_ID = 7005;
const UI_OPTIONS = ['orbit', 'spectator', 'first-person', 'third-person', 'top-view'] as const;
type ControllerKind = (typeof UI_OPTIONS)[number];
type ActiveController =
  | BaseCameraControllerHandle
  | MotionCameraControllerHandle
  | OrbitControllerHandle
  | ThirdPersonControllerHandle
  | TopViewControllerHandle;
type ZoomControllerHandle =
  | OrbitControllerHandle
  | ThirdPersonControllerHandle
  | TopViewControllerHandle;

function isMotionController(
  controller: ActiveController
): controller is MotionCameraControllerHandle {
  return 'pressForward' in controller;
}
function isThirdPersonController(
  controller: ActiveController
): controller is ThirdPersonControllerHandle {
  return 'setTarget' in controller;
}

function isTopViewController(controller: ActiveController): controller is TopViewControllerHandle {
  return 'setFocus' in controller;
}
function isOrbitController(controller: ActiveController): controller is OrbitControllerHandle {
  return 'setEnabled' in controller;
}

function isZoomController(controller: ActiveController): controller is ZoomControllerHandle {
  return 'toZoom' in controller;
}

function makeUiAreaOp(windowWidth: number) {
  const x = Math.max(12, windowWidth - 300);
  return {
    type: 'area',
    content: {
      label: 'camera-controller-picker',
      x,
      y: 12,
      draggable: false
    }
  } as const;
}

async function main() {
  initEngine({ transport: transportBunFfi });

  const { windowId } = createWindow({
    title: 'Vulfram Demo 007 - Camera Controllers + UI',
    size: [1280, 720],
    position: [100, 100],
    borderless: false,
    resizable: true,
    transparent: false,
    initialState: 'maximized'
  });

  let totalMs = 0;
  let uiVersion = 0;
  let uiReady = false;
  let selectedKind: ControllerKind = 'orbit';
  let topViewFocusLocked = true;
  let confinementEnabled = false;
  let lastAppliedCursorGrab: 'none' | 'locked' = 'none';
  let lastLoggedSystemErrorMessage = '';

  const world3dId = World3D.create3DWorld();
  const world3dMount = Mount.mountWorld(world3dId, {
    target: { kind: 'window', windowId },
    layout: {
      left: { unit: 'percent', value: 0 },
      top: { unit: 'percent', value: 0 },
      width: { unit: 'percent', value: 100 },
      height: { unit: 'percent', value: 100 },
      zIndex: 0,
      blendMode: 0
    }
  });

  const uiWorldId = WorldUI.createUIWorld();
  const uiMount = Mount.mountWorld(uiWorldId, {
    target: { kind: 'window', windowId },
    layout: {
      left: { unit: 'percent', value: 0 },
      top: { unit: 'percent', value: 0 },
      width: { unit: 'percent', value: 100 },
      // Keep UI in a top strip so it doesn't block 3D clicks on the whole viewport.
      height: { unit: 'percent', value: 18 },
      zIndex: 10,
      blendMode: 0
    }
  });
  const worldMountTargetId = world3dMount.targetId as unknown as number;
  const uiMountTargetId = uiMount.targetId as unknown as number;

  World3D.configure3DEnvironment(world3dId, {
    msaa: { enabled: true, sampleCount: 1 },
    skybox: {
      mode: 'none',
      intensity: 0,
      rotation: 0,
      groundColor: [0.02, 0.03, 0.04],
      horizonColor: [0.11, 0.14, 0.2],
      skyColor: [0.2, 0.27, 0.37],
      cubemapTextureId: null
    },
    clearColor: [0.02, 0.03, 0.04, 1],
    post: {
      filterEnabled: false,
      filterExposure: 1,
      filterGamma: 2,
      filterSaturation: 1,
      filterContrast: 1,
      filterVignette: 0,
      filterGrain: 0,
      filterChromaticAberration: 0,
      filterBlur: 0,
      filterSharpen: 0,
      filterTonemapMode: 1,
      outlineEnabled: false,
      outlineStrength: 0,
      outlineThreshold: 0.2,
      outlineWidth: 1,
      outlineQuality: 1,
      filterPosterizeSteps: 0,
      cellShading: false,
      ssaoEnabled: false,
      ssaoStrength: 1,
      ssaoRadius: 0.75,
      ssaoBias: 0.025,
      ssaoPower: 1.5,
      ssaoBlurRadius: 2,
      ssaoBlurDepthThreshold: 0.02,
      bloomEnabled: false,
      bloomThreshold: 1,
      bloomKnee: 0.5,
      bloomIntensity: 0.8,
      bloomScatter: 0.7
    }
  });
  World3D.configure3DShadows(world3dId, {
    tileResolution: 1024,
    atlasTilesW: 16,
    atlasTilesH: 16,
    atlasLayers: 2,
    virtualGridSize: 1,
    smoothing: 2,
    normalBias: 0.02
  });

  const cameraEntityId = World3D.create3DEntity(world3dId);
  World3D.create3DCamera(world3dId, cameraEntityId, {
    kind: 'perspective',
    near: 0.1,
    far: 300,
    order: 0
  });

  const geometryId = World3D.create3DGeometry(world3dId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Demo007Cube'
  });
  const floorTex = World3D.create3DTexture(world3dId, {
    source: { type: 'color', color: [0.25, 0.35, 0.45, 1] },
    srgb: true,
    label: 'Demo007FloorTex'
  });
  const focusTex = World3D.create3DTexture(world3dId, {
    source: { type: 'color', color: [0.95, 0.55, 0.25, 1] },
    srgb: true,
    label: 'Demo007FocusTex'
  });
  const floorMat = World3D.create3DMaterial(world3dId, {
    kind: 'standard',
    label: 'Demo007FloorMat',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: floorTex,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque'
      }
    }
  });
  const focusMat = World3D.create3DMaterial(world3dId, {
    kind: 'standard',
    label: 'Demo007FocusMat',
    options: {
      type: 'standard',
      content: {
        baseColor: [1, 1, 1, 1],
        baseTexId: focusTex,
        baseSampler: 'linear-clamp',
        flags: 0,
        surfaceType: 'opaque'
      }
    }
  });

  const floorEntityId = World3D.create3DEntity(world3dId);
  World3D.update3DTransform(world3dId, floorEntityId, {
    position: [0, -1.6, 0],
    scale: [24, 0.1, 24]
  });
  World3D.create3DModel(world3dId, floorEntityId, {
    geometryId,
    materialId: floorMat,
    castShadow: false,
    receiveShadow: true
  });

  const focusEntityId = World3D.create3DEntity(world3dId);
  World3D.create3DModel(world3dId, focusEntityId, {
    geometryId,
    materialId: focusMat,
    castShadow: true,
    receiveShadow: true
  });

  const keyLightId = World3D.create3DEntity(world3dId);
  World3D.update3DTransform(world3dId, keyLightId, { position: [6, 8, 4] });
  World3D.create3DLight(world3dId, keyLightId, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 12,
    range: 30,
    castShadow: true
  });

  function createController(kind: ControllerKind): ActiveController {
    const speed = 0.05;

    if (kind === 'orbit')
      return createOrbitController(world3dId, cameraEntityId, {
        target: [0, 0, 0],
        radius: 8,
        enabled: false,
        rotateSpeed: speed
      });
    if (kind === 'spectator')
      return createSpectatorController(world3dId, cameraEntityId, {
        alwaysLook: true,
        lookSpeed: speed
      });
    if (kind === 'first-person')
      return createFirstPersonController(world3dId, cameraEntityId, {
        alwaysLook: true,
        lookSpeed: speed
      });
    if (kind === 'third-person')
      return createThirdPersonController(world3dId, cameraEntityId, {
        target: [0, 0, 0],
        distance: 6,
        alwaysLook: true,
        rotateSpeed: speed
      });
    return createTopViewController(world3dId, cameraEntityId, {
      focus: [0, 0, 0],
      height: 12,
      focusLocked: topViewFocusLocked,
      rotateSpeed: speed
    });
  }

  let controller = createController(selectedKind);

  let previousWindowSize = World3D.get3DWindowSize(world3dId);
  let lastPointerDebugText = '';
  let nextPointerDebugAtMs = 0;
  const focus = vec3.fromValues(0, 0, 0);
  World3D.update3DTransform(world3dId, focusEntityId, {
    position: [focus[0], focus[1], focus[2]],
    scale: [0.75, 0.75, 0.75]
  });
  let last = performance.now();
  let shouldExit = false;
  while (!shouldExit) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;
    const dtSeconds = dtMs / 1000;
    const t = totalMs / 1000;

    const animateFocus =
      selectedKind === 'third-person' || (selectedKind === 'top-view' && topViewFocusLocked);
    if (animateFocus) {
      vec3.set(focus, Math.cos(t * 0.65) * 3.5, 0, Math.sin(t * 0.65) * 2.4);
    } else {
      vec3.set(focus, 0, 0, 0);
    }
    World3D.update3DTransform(world3dId, focusEntityId, {
      position: [focus[0], focus[1], focus[2]],
      scale: [0.75, 0.75, 0.75]
    });

    if (isThirdPersonController(controller)) controller.setTarget(focus);
    if (isTopViewController(controller)) {
      controller.setFocus(focus);
      controller.setFocusLocked(topViewFocusLocked);
    }

    const pointerCaptureState = World3D.get3DWindowPointerCaptureState(world3dId);
    const isMouseLocked = pointerCaptureState?.mode === 'locked' && pointerCaptureState.active;
    const requiresLockedControl =
      selectedKind === 'spectator' ||
      selectedKind === 'first-person' ||
      selectedKind === 'third-person';
    const canControlCamera = !requiresLockedControl || isMouseLocked;

    if (isMotionController(controller)) {
      const k = World3D.KeyCode;
      if (
        World3D.is3DWindowCloseRequested(world3dId) ||
        World3D.is3DKeyPressed(world3dId, k.Escape)
      ) {
        shouldExit = true;
      }
      if (canControlCamera) {
        World3D.is3DKeyPressed(world3dId, k.KeyW)
          ? controller.pressForward()
          : controller.releaseForward();
        World3D.is3DKeyPressed(world3dId, k.KeyS)
          ? controller.pressBackward()
          : controller.releaseBackward();
        World3D.is3DKeyPressed(world3dId, k.KeyA)
          ? controller.pressLeft()
          : controller.releaseLeft();
        World3D.is3DKeyPressed(world3dId, k.KeyD)
          ? controller.pressRight()
          : controller.releaseRight();
        World3D.is3DKeyPressed(world3dId, k.KeyE) ? controller.pressUp() : controller.releaseUp();
        World3D.is3DKeyPressed(world3dId, k.KeyQ)
          ? controller.pressDown()
          : controller.releaseDown();
      } else {
        controller.releaseForward();
        controller.releaseBackward();
        controller.releaseLeft();
        controller.releaseRight();
        controller.releaseUp();
        controller.releaseDown();
      }
    } else {
      const k = World3D.KeyCode;
      if (
        World3D.is3DWindowCloseRequested(world3dId) ||
        World3D.is3DKeyPressed(world3dId, k.Escape)
      ) {
        shouldExit = true;
      }
    }

    if (
      selectedKind === 'top-view' &&
      World3D.is3DKeyJustPressed(world3dId, World3D.KeyCode.KeyL)
    ) {
      topViewFocusLocked = !topViewFocusLocked;
      console.info(`[Demo007][TopView] focusLocked=${topViewFocusLocked} (toggle: key L)`);
      if (isTopViewController(controller)) {
        controller.setFocusLocked(topViewFocusLocked);
      }
    }

    if (World3D.is3DKeyJustPressed(world3dId, World3D.KeyCode.Space)) {
      confinementEnabled = !confinementEnabled;
      console.info(`[Demo007][Pointer] confinementEnabled=${confinementEnabled} (toggle: Space)`);
    }

    const confinementAllowedByMode =
      isMotionController(controller) || isThirdPersonController(controller);
    const desiredCursorGrab: 'none' | 'locked' =
      confinementAllowedByMode && confinementEnabled ? 'locked' : 'none';
    if (desiredCursorGrab !== lastAppliedCursorGrab) {
      if (desiredCursorGrab === 'locked') {
        focusWindow(windowId);
      }
      updateWindow(windowId, {
        cursorGrab: desiredCursorGrab,
        cursorVisible: desiredCursorGrab === 'none'
      });
      lastAppliedCursorGrab = desiredCursorGrab;
    }

    if (isOrbitController(controller)) {
      const targetIdUnderPointer = World3D.get3DPointerTargetId(world3dId);
      const clickedThisFrame = World3D.is3DPointerButtonJustPressed(world3dId, 0);
      if (clickedThisFrame) {
        if (targetIdUnderPointer === worldMountTargetId) {
          controller.enable();
          console.info('[Demo007][Orbit] enabled=true (click on 3D realm)');
        } else {
          controller.disable();
          console.info('[Demo007][Orbit] enabled=false (click outside 3D realm)');
        }
      }
    }

    if (isZoomController(controller)) {
      const scrollDelta = World3D.get3DScrollDelta(world3dId);
      if (scrollDelta[1] !== 0) {
        const targetIdUnderPointer = World3D.get3DPointerTargetId(world3dId);
        const canZoomWithScroll = !isThirdPersonController(controller) || isMouseLocked;
        if (targetIdUnderPointer === worldMountTargetId && canZoomWithScroll) {
          controller.toZoom(-scrollDelta[1]);
        }
      }
    }

    if (canControlCamera) {
      controller.update(dtSeconds);
    }

    if (totalMs >= nextPointerDebugAtMs) {
      const pointerPos = World3D.get3DPointerPosition(world3dId);
      const pointerDelta = World3D.get3DPointerDelta(world3dId);
      const pointerTargetPos = World3D.get3DPointerTargetPosition(world3dId);
      const pointerTargetDelta = World3D.get3DPointerTargetDelta(world3dId);
      const pointerTargetUv = World3D.get3DPointerTargetUv(world3dId);
      const pointerTargetId = World3D.get3DPointerTargetId(world3dId);
      const isLeft = World3D.is3DPointerButtonPressed(world3dId, 0);
      const isMiddle = World3D.is3DPointerButtonPressed(world3dId, 1);
      const isRight = World3D.is3DPointerButtonPressed(world3dId, 2);
      const windowPos = World3D.get3DWindowPosition(world3dId);
      const isWindowFocused = World3D.is3DWindowFocused(world3dId);
      const windowScaleFactor = World3D.get3DWindowScaleFactor(world3dId);
      const connectedGamepads = World3D.get3DConnectedGamepads(world3dId);
      const _gamepadEvents = World3D.get3DGamepadEvents(world3dId);
      const firstGamepadId = connectedGamepads[0]?.gamepadId;
      const gamepadAxis0 =
        firstGamepadId !== undefined ? World3D.get3DGamepadAxis(world3dId, firstGamepadId, 0) : 0;
      const gamepadButton0Pressed =
        firstGamepadId !== undefined
          ? World3D.is3DGamepadButtonPressed(world3dId, firstGamepadId, 0)
          : false;

      console.info(
        `[Demo007][Pointer] buttons(LMR)=${Number(isLeft)}${Number(isMiddle)}${Number(isRight)} targetId=${pointerTargetId ?? 'none'} ` +
          `gPos=${pointerPos[0].toFixed(2)},${pointerPos[1].toFixed(2)} gDelta=${pointerDelta[0].toFixed(3)},${pointerDelta[1].toFixed(3)} ` +
          `tPos=${pointerTargetPos ? `${pointerTargetPos[0].toFixed(2)},${pointerTargetPos[1].toFixed(2)}` : 'none'} ` +
          `tDelta=${pointerTargetDelta ? `${pointerTargetDelta[0].toFixed(3)},${pointerTargetDelta[1].toFixed(3)}` : 'none'} ` +
          `tUv=${pointerTargetUv ? `${pointerTargetUv[0].toFixed(3)},${pointerTargetUv[1].toFixed(3)}` : 'none'} ` +
          `winPos=${windowPos[0]},${windowPos[1]} focus=${isWindowFocused ? '1' : '0'} scale=${windowScaleFactor.toFixed(2)} ` +
          `gp=${connectedGamepads.length} gpAxis0=${gamepadAxis0.toFixed(3)} gpB0=${gamepadButton0Pressed ? '1' : '0'}`
      );

      const systemError = World3D.get3DLastSystemError(world3dId);
      if (systemError) {
        const message = `[${systemError.scope}] ${systemError.message}`;
        if (message !== lastLoggedSystemErrorMessage) {
          lastLoggedSystemErrorMessage = message;
          console.warn(`[Demo007][SystemError] ${message}`);
        }
      }
      nextPointerDebugAtMs = totalMs + POINTER_DEBUG_INTERVAL_MS;
    }

    if (!uiReady) {
      const uiRealmId = WorldUI.getUIWorldRealmId(uiWorldId);
      if (uiRealmId !== null) {
        const windowSize = World3D.get3DWindowSize(world3dId);
        previousWindowSize = windowSize;
        WorldUI.createUIDocument(uiWorldId, {
          documentId: UI_DOCUMENT_ID,
          rect: [0, 0, windowSize[0], windowSize[1]]
        });
        WorldUI.applyUIOps(uiWorldId, {
          documentId: UI_DOCUMENT_ID,
          version: ++uiVersion,
          ops: [
            {
              type: 'add',
              content: {
                node: {
                  id: UI_NODE_AREA_ID,
                  kind: 'area',
                  props: makeUiAreaOp(windowSize[0])
                }
              }
            },
            {
              type: 'add',
              content: {
                parent: UI_NODE_AREA_ID,
                node: {
                  id: UI_NODE_TITLE_ID,
                  kind: 'text',
                  props: {
                    type: 'text',
                    content: { text: 'Camera Controller' }
                  }
                }
              }
            },
            {
              type: 'add',
              content: {
                parent: UI_NODE_AREA_ID,
                node: {
                  id: UI_NODE_COMBO_ID,
                  kind: 'combo-box',
                  props: {
                    type: 'combo-box',
                    content: {
                      label: 'Type',
                      selected: selectedKind,
                      options: [...UI_OPTIONS],
                      enabled: true
                    }
                  }
                }
              }
            },
            {
              type: 'add',
              content: {
                parent: UI_NODE_AREA_ID,
                node: {
                  id: UI_NODE_POINTER_DELTA_ID,
                  kind: 'text',
                  props: {
                    type: 'text',
                    content: {
                      text: 'dG: 0.000, 0.000 | dT: none'
                    }
                  }
                }
              }
            }
          ]
        });
        uiReady = true;
      }
    }

    if (uiReady) {
      const sizeNow = World3D.get3DWindowSize(world3dId);
      const pointerDeltaNow = World3D.get3DPointerDelta(world3dId);
      const pointerTargetDeltaNow = World3D.get3DPointerTargetDelta(world3dId);
      const pointerDebugText = `dG: ${pointerDeltaNow[0].toFixed(3)}, ${pointerDeltaNow[1].toFixed(3)} | dT: ${
        pointerTargetDeltaNow
          ? `${pointerTargetDeltaNow[0].toFixed(3)}, ${pointerTargetDeltaNow[1].toFixed(3)}`
          : 'none'
      }`;
      if (World3D.was3DWindowResized(world3dId)) {
        previousWindowSize = sizeNow;
        WorldUI.setUIDocumentRect(uiWorldId, {
          documentId: UI_DOCUMENT_ID,
          rect: [0, 0, sizeNow[0], sizeNow[1]]
        });
        WorldUI.applyUIOps(uiWorldId, {
          documentId: UI_DOCUMENT_ID,
          version: ++uiVersion,
          ops: [
            {
              type: 'set',
              content: {
                node_id: UI_NODE_AREA_ID,
                props: makeUiAreaOp(sizeNow[0])
              }
            }
          ]
        });
      }

      if (pointerDebugText !== lastPointerDebugText) {
        lastPointerDebugText = pointerDebugText;
        WorldUI.applyUIOps(uiWorldId, {
          documentId: UI_DOCUMENT_ID,
          version: ++uiVersion,
          ops: [
            {
              type: 'set',
              content: {
                node_id: UI_NODE_POINTER_DELTA_ID,
                props: {
                  type: 'text',
                  content: { text: pointerDebugText }
                }
              }
            }
          ]
        });
      }

      for (const uiEvent of WorldUI.getUIEvents(uiWorldId)) {
        if (uiEvent.nodeId !== UI_NODE_COMBO_ID || uiEvent.kind !== 'changed') continue;
        const next = uiEvent.label as ControllerKind | undefined;
        if (!next || next === selectedKind || !UI_OPTIONS.includes(next)) continue;
        selectedKind = next;
        controller = createController(selectedKind);
      }
    }

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((resolve) =>
      setTimeout(resolve, Math.max(0, FRAME_TARGET_MS - frameElapsed))
    );
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
