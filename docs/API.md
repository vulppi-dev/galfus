# 🦊 Vulfram — Internal API (Crates, Structs, Internal Flow)

> This is an **internal engineering document**.
> It describes how the Vulfram core is structured on the Rust side: crates,
> data structures, and internal flows. It is **not** intended for engine users.

---

## 1. Crates and Runtime Stack

The Vulfram core is built as a Rust library with the following key crates:

- **Platform proxies**
  - `winit` (desktop)
    - Window creation
    - Event loop integration
    - Input events (keyboard, mouse, touch, gestures)
  - `web-sys` (browser/WASM)
    - DOM canvas, input events, gamepad polling
  - `console_error_panic_hook` (browser/WASM)
    - Captures Rust panics and forwards them to browser console traces
    - Hook is installed at WASM module start and re-ensured on exported entry points
  - `web-time` (browser/WASM)
    - Provides `Instant::now()` compatibility where `std::time` is unsupported

- **Rendering**
  - `wgpu`
    - Cross-platform GPU abstraction
    - Device and queue management
    - Render pipelines, buffers, textures

- **Gamepad input**
  - `gilrs` (desktop)
  - Web Gamepad API (browser/WASM)

- **Images**
  - `image`
    - Texture file decoding (PNG, JPEG, etc.)
    - Conversion to raw pixel data for WGPU

- **Math / binary packing**
  - `glam`
    - Vector, matrix, quaternion types (`Vec2`, `Vec3`, `Mat4`, etc.)
  - `bytemuck`
    - Safe casting between typed structs and `&[u8]` / `&mut [u8]`
    - Marks types as `Pod` / `Zeroable` for GPU upload

- **Serialization**
  - `serde`
  - `rmp-serde`
    - MessagePack serialization/deserialization for commands, events, profiling

---

## 2. Engine State Overview

At a high level, the core is organized around a central `EngineState`:

```rust
pub struct EngineState {
    pub window: WindowManager,

    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub wgpu: wgpu::Instance,
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub caps: Option<wgpu::SurfaceCapabilities>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    pub buffers: BufferStorage,
    pub texture_async: TextureAsyncManager,
    pub audio: Box<dyn AudioProxy>,
    pub universal_state: UniversalState,

    pub cmd_queue: EngineBatchCmds,
    pub event_queue: EngineBatchEvents,
    pub response_queue: EngineBatchResponses,

    pub(crate) time: u64,
    pub(crate) delta_time: u32,
    pub(crate) frame_index: u64,

    #[cfg(not(feature = "wasm"))]
    pub input: InputState,
    pub(crate) gamepad: GamepadState,

    pub(crate) profiling: TickProfiling,
    pub(crate) gpu_profiler: Option<GpuProfiler>,
}
```

`EngineSingleton` owns the `EngineState` plus a platform proxy
(`DesktopProxy` or `BrowserProxy`) that handles window/input integration.

`UniversalState` is the realm-centric runtime table set:

- `realms`, `surfaces`, `connectors`, `presents`
- `render_graphs` (global catalog of render graph resources referenced by `render_graph_id`)
- `surface_cache` for cycle-breaking (`LastGoodSurface`/`FallbackSurface`)
- `frame_report` for RealmGraph diagnostics

---

## 3. Resources (Current)

The core manages several first-class resources:

- **Geometry**: Managed by the vertex allocator (pooled or dedicated buffers).
- **Textures**: Loaded from buffers or created as solid colors.
- **Materials**: Define the appearance of meshes.
- **Shadows**: Global shadow mapping configuration per window.

State ownership split (current):

- `camera`, `model`, and `light` are realm-owned entities (`realmId` scoped).
- `material`, `texture`, and `geometry` are shared under `UniversalState.universal_resources`.
- Window lifecycle affects presentation/surfaces, not universal resource ownership.

Gizmos (current):

- `line`, `aabb`, and `polyline` commands render in the forward color path.
- `thickness` is interpreted in screen-space pixels.
- Thick gizmos are emitted as solid triangulated ribbons (triangle list), not hardware line width.

---

## 4. Components and Instances (Current)

The render state consumes realm/global registries to build the active per-realm scene each frame:

- `CameraRecord`
  - `label: Option<String>` (semantic name)
  - `data: CameraComponent` (projection/view matrices)
  - `layer_mask`, `order`
  - `view_position` (optional, relative/absolute)
  - `render_target` (per-camera texture)
  - `forward_depth_target` (per-camera depth intermediate for forward/SSAO)
  - `forward_msaa_target` (per-camera MSAA color intermediate for forward/skybox)
  - `forward_emissive_msaa_target` (per-camera MSAA emissive intermediate)
  - Projection is computed from the effective per-camera target/surface size (window size only when rendering directly to a window-sized surface).

- `ModelRecord`
  - `label: Option<String>` (semantic name)
  - `data: ModelComponent` (transform + derived TRS)
  - `geometry_id` (required)
  - `cast_outline` and `outline_color` (outline mask + color for post)

---

## 5. Environment & Post-Processing (Current)

The environment config now includes a post-processing block used by the `post` pass.

`EnvironmentConfig` (core/resources/environment/spec.rs):

- `msaa`
- `skybox`
- `clearColor` (`Vec4` RGBA clear used when skybox is disabled; default/fallback is `[0,0,0,0]`)
- `post`

Environment profiles are host-addressable by `environmentId` via
`CmdEnvironmentUpsert` and can be assigned per `TargetLayer`
using `CmdTargetLayerUpsert.environmentId`.
Selection priority during rendering:
1. camera layer `environmentId` (when set and profile exists), applied per camera;
2. current default profile;
3. core fallback defaults.

MSAA is resolved per camera from the selected environment for that camera-target bind.
If a layer references a missing `environmentId`, that bind falls back to default/fallback profiles.

`SkyboxConfig` highlights:

- `mode`: `none`, `procedural`, `cubemap`
- `intensity`: overall multiplier
- `rotation`: radians, applied around Y
- `ground_color`: ground/low hemisphere color
- `horizon_color`: horizon blend color
- `sky_color`: upper sky color
- `horizon_ground_threshold`: ground influence over horizon on lower hemisphere (`0.0` = horizon-only, `1.0` = ground-dominant with minimal blend)
- `horizon_sky_threshold`: sky influence over horizon on upper hemisphere (`0.0` = horizon-only, `1.0` = sky-dominant with minimal blend)
- `directional_lights`: list of sun descriptors with:
  - `light_id`: directional light id
  - `solid_size`: normalized solid size (`0.0` no coverage, `1.0` covers hemisphere, `2.0` covers full sky)
  - `gradient_size`: normalized halo size using the same scale as `solid_size` (should be >= solid size)
- `cubemap_texture_id`: 2D equirect sky texture ID (lat/long); sampled only when `mode = cubemap`

Texture loading notes:

- EXR/HDR inputs decode to `rgba16f` textures (not supported in forward atlas).

Async texture decode:

- `CmdTextureCreateFromBuffer` returns `{ pending: true }` when decode is queued.
- The engine later emits `SystemEvent::TextureReady { windowId, textureId, success, message }`.
- Diagnostic failures also emit `SystemEvent::Error { scope, message, commandId?, commandType? }`.
- Order-independent command reconciliation emits:
  - `SystemEvent::CommandDeferred { commandId, commandType, attempts, reason }`
  - `SystemEvent::CommandApplied { commandId, commandType, attempts }`
  - `SystemEvent::CommandDropped { commandId, commandType, attempts, reason }`
- Deferred retries use exponential backoff (up to 64 frames between attempts) and
  are dropped after retry/age limits to avoid infinite loops.

`PostProcessConfig` highlights:

- `filter_enabled`: master enable for filters
- `filter_exposure`: HDR exposure multiplier
- `filter_gamma`: gamma correction
- `filter_saturation`: color saturation
- `filter_contrast`: color contrast
- `filter_vignette`: vignette strength
- `filter_grain`: film grain
- `filter_chromatic_aberration`: chromatic aberration strength
- `filter_blur`: blur amount
- `filter_sharpen`: sharpen amount
- `filter_tonemap_mode`: 0 = none, 1 = Reinhard, 2 = ACES
- `filter_posterize_steps`: number of posterize steps (0 disables)
- `outline_enabled`: enables outline composition in post
- `outline_strength`: mix amount for outline color
- `outline_threshold`: edge threshold (clamped to `[0, 1)`)
- `outline_width`: pixel width used by edge kernel
- `outline_quality`: 0 = 3×3 kernel, 1 = 5×5 kernel
- `ssao_enabled`: enable SSAO composition in post
- `ssao_strength`: SSAO mix strength in post
- `ssao_radius`: sampling radius for SSAO
- `ssao_bias`: depth bias to reduce self-occlusion
- `ssao_power`: contrast curve for SSAO output
- `ssao_blur_radius`: bilateral blur radius (pixels)
- `ssao_blur_depth_threshold`: depth threshold for blur weights
- SSAO suporta depth MSAA (amostra média por pixel quando MSAA está ativo)
- Em realms com múltiplas câmeras, intermediários forward/depth são isolados por câmera (MSAA não é desativado por contagem de câmera)
- `bloom_enabled`: enable bloom/glow composition in post
- `bloom_threshold`: threshold for bright pass
- `bloom_knee`: soft knee for thresholding
- `bloom_intensity`: bloom mix intensity in post
- `bloom_scatter`: scatter factor during upsample

The outline mask is rendered in a dedicated `outline` pass into `outline_color`
(now `rgba8`), and sampled by the `post` pass for final composition.
`post` and `compose` now use separate uniform buffers (`PostProcessUniform` vs
cover `vec4`) to avoid cross-pass parameter overwrite.

---

## 6. Audio (Core, WIP)

The audio system is proxy-based (desktop = Kira, browser = WebAudio). The API is shared across backends.

Command definitions live in `docs/cmds`.

Notes:

- `intensity` is a 0..1 scalar applied on top of `gain` when playing.
- `mode` supports `once`, `loop`.
- When a source is bound to a model, the core updates its position every tick.
- If the bound source model is the same as the bound listener model, spatialization is bypassed.
- Streaming resources emit `SystemEvent::AudioStreamProgress`.
- If audio initialization fails (for example, missing device/backend), the runtime marks audio as unavailable, emits one `SystemEvent::Error` with scope `audio-init`, and short-circuits audio commands/binding updates with deterministic command errors.

Events:

- `SystemEvent::AudioReady { resourceId, success, message }` (async decode)
  - Emitted when the audio buffer finishes decoding (desktop and web).
  - Use this to decide when `CmdAudioSourceTransport` with `action = "play"` is safe to call.
- `SystemEvent::AudioStreamProgress { resourceId, receivedBytes, totalBytes, complete }` (streaming)
  - Emitted as stream chunks are received.

The visibility rule uses `layer_mask`:

```text
(model.layer_mask & camera.layer_mask) != 0
```

---

## 5. Internal Command Flow

Commands are sent from the host via MessagePack to `vulfram_send_queue`,
then decoded into internal Rust enums.

### 5.1 ABI Layer (C → Rust)

- `vulfram_send_queue(buffer, length)`
  1. Copies the buffer into a `Vec<u8>`.
  2. Deserializes with `rmp-serde` into `EngineBatchCmds` (`Vec<EngineCmdEnvelope>`).
  3. Pushes the commands into `EngineState::cmd_queue`.
  4. On deserialize/type mismatch error, returns `CmdInvalidMessagePackError` and emits
     `SystemEvent::Error` with scope `serialization`, command type `send-queue`, and
     the decode path of the invalid field when available.

### 5.2 Command Representation

`EngineCmd` is the internal command enum; see `docs/cmds` for the command surface.
Unified upsert commands are available for resource create/update pairs:
`camera`, `model`, `light`, `material`, `geometry`, `environment`,
`audio-listener`, and `audio-source`.
For these families, use only `*Upsert` command variants.
Diagnostics and trace/profiling runtime policy is configured by `CmdSystemDiagnosticsSet`.

UI command surface is split by domain:

- Theme/document ops:
  - `CmdUiThemeDefine`, `CmdUiThemeDispose`
  - `CmdUiDocumentCreate`, `CmdUiDocumentDispose`, `CmdUiDocumentSetRect`, `CmdUiDocumentSetTheme`
  - `CmdUiApplyOps`
- Introspection/focus/trace:
  - `CmdUiDocumentGetTree`, `CmdUiDocumentGetLayoutRects`
  - `CmdUiFocusSet`, `CmdUiFocusGet`
  - `CmdUiEventTraceSet`
- UI resources and input bridge:
  - `CmdUiImageCreateFromBuffer`, `CmdUiImageDispose`
  - `CmdUiClipboardPaste`, `CmdUiScreenshotReply`, `CmdUiAccessKitActionRequest`

Technical UI runtime details are in `docs/ui/*`.
Per-command contracts remain in `docs/cmds/*`.

### 5.3 Command Execution

During `vulfram_tick`:

1. Drain `cmd_queue`.
2. For each `EngineCommand`, call into appropriate systems:
   - Resource creation/update
   - Component creation/update
   - Maintenance (e.g. cleaning uploads)

3. Update any derived data required for rendering (culling, visibility, etc.).

Diagnostic rule:
- Any diagnosable failure must also emit `SystemEvent::Error` into the host event pool.
- Core-side code should use the shared helper `push_error_event(...)` for consistency.

---

## 5.4 Asynchronous Resource Linking (Fallback-Driven)

The render path tolerates missing references:

- Models may reference geometry or material IDs that do not exist yet.
- Materials may reference texture IDs that do not exist yet.

Missing resources fall back to safe defaults. When the real resource is created
later with the same ID, the core picks it up automatically on the next frame.

## 6. Upload Handling (`UploadBuffer`)

Upload buffers manage blobs uploaded via `vulfram_upload_buffer`:

```rust
pub struct UploadBuffer {
    pub upload_type: UploadType,
    pub data: Vec<u8>,
}
```

Stored in `EngineState`:

```rust
buffers: HashMap<u64, UploadBuffer>  // BufferId -> UploadBuffer
```

- `vulfram_upload_buffer`:
  - Inserts an `UploadBuffer` for a given `BufferId` (u64).
  - Returns error if buffer ID already exists (one-shot semantics).

- `Create*` commands:
  - Look up the `UploadBuffer` by `BufferId`.
  - Use/consume its data to create WGPU resources.
  - Remove entry after consumption.

- `CmdUploadBufferDiscardAll` command:
  - Iterates and removes any unconsumed upload buffers.

---

## 7. Rendering System Overview

The `RenderState` is responsible for managing WGPU objects and executing
the draw passes.

Rendering is Realm-based:

- Each `Realm` owns a `RenderGraphState` (3D or 2D).
- The `RealmGraphPlanner` orders realms and determines cut edges for cycles.
- Composition uses connector overlays after per-realm rendering.

### 7.0 Auto-Graph Maps (Experimental)

The host provides logical maps only:

- `RealmMap` (realmId -> kind)
- `TargetMap` (targetId -> kind)
- `TargetLayerMap` (realmId -> targetId + layout)

The core resolves `TargetGraph` + `RealmGraph` automatically and creates
`Surface`, `Present`, and `Connector` entries internally.
These entries are internal-only and are not exposed as host commands.

Host bindings can update these maps with:

- `CmdTargetUpsert` / `CmdTargetMeasurement` / `CmdTargetDispose`
- `CmdTargetLayerUpsert` / `CmdTargetLayerDispose`

Example:

```text
CmdTargetUpsert(targetId=9000, kind=window, windowId=1)
CmdTargetUpsert(targetId=9002, kind=window, windowId=1)
CmdTargetUpsert(targetId=9003, kind=texture, size=640x360)
CmdTargetLayerUpsert(realmId=10, targetId=9000, layout=..., cameraId=1)
CmdTargetLayerUpsert(realmId=11, targetId=9002, layout=left/top/width/height/zIndex/clip)
```

Rules:
- `windowId` is mandatory only for `window`.
  `widget-realm-viewport` is intended to be consumed by `UiNodeProps::WidgetRealmViewport`.
  Its internal auto-link is used for dependency/routing and UI sampling, not for visible connector composition.
- `size` is accepted only for `texture`.
- `cameraId` on `CmdTargetLayerUpsert` is optional:
  - when set, that camera is used for `Realm3D` sampling on the layer;
  - when omitted, core uses the first available camera;
  - when no camera exists, output remains `clearColor`.
- `environmentId` on `CmdTargetLayerUpsert` is optional:
  - when set, that environment profile is used for the layer;
  - when omitted, current default environment profile is used;
  - when no profile exists, core fallback defaults are used.
  - effective MSAA for that camera bind is derived from this selected environment.
- For `window`/`widget-realm-viewport` connector layers and `realm-plane`,
  surface size is derived from
  `TargetLayerLayout.width` and `TargetLayerLayout.height` when the layer is
  resolved.
- `TargetLayerLayout.left/top/width/height` accept `DimensionValue`:
  - `px`
  - `percent`
  - `character` (`ch`)
  - `display` (`dp`, 4px grid)

Frame reporting includes composition diagnostics for host tooling.

### 7.1 Buffers

Current GPU buffers:

- `FrameUniformBuffer`
  - Time, delta time, frame index.

- `CameraUniformBuffer`
  - Camera matrices and parameters.

- `ModelUniformBuffer`
  - Model transforms and derived TRS.

- Vertex / index buffers for geometries (managed by `VertexAllocatorSystem`).

### 7.2 Render Pass Flow (per Frame)

Conceptual flow:

1. Update uniform buffers (if dirty):
   - Write updated frame/camera/model data into pools.

2. For each camera:
   - Set its render target (texture view) as the color attachment.
   - Configure viewport and scissor according to `CameraInstance`.
   - Clear or load the attachment as needed.
   - Iterate over `MeshInstance`s:
     - Filter by `layerMask`:

     ```rust
     if camera.layer_mask & mesh.layer_mask != 0 {
         // visible
     }
     ```

     - Select pipeline:

     - Fetch from the pipeline cache or create if needed.

     - Bind vertex/index buffers and resource bind groups.

     - Issue draw calls via `RenderPass::draw` / `draw_indexed`.

### 7.3 Render Ordering & Batching (Per Camera)

- Opaque/masked objects are sorted by `(material_id, geometry_id)` to reduce
  state changes and batch draw calls.
- Transparent objects are sorted by depth for correct blending.

Draw calls are batched by runs of `(material_id, geometry_id)` after sorting.

3. Submit the frame to the surface swapchain.

---

## 8. Event System

The input layer aggregates events via the active platform proxy:

- Keyboard/pointer/touch from `winit` (desktop) or DOM (browser)
- Gamepad from `gilrs` (desktop) or the Web Gamepad API (browser)
- Window events (resize, close, focus, etc.) from the platform

These are translated into internal `EngineEvent` enums and pushed into
`event_queue`.

Window state transitions are emitted explicitly as:
- `WindowEvent::OnStateChange` for lifecycle changes (`windowed`, `fullscreen`, etc.);
- `WindowEvent::OnPointerCaptureChange` for pointer capture mode/activation changes.

Pointer events now include optional routing metadata via `trace`, which
provides the resolved `windowId`, `realmId`, `targetId`, `connectorId`,
`sourceRealmId`, and UV coordinates when a connector hit-test succeeds. This
metadata is optional and omitted when routing is not available.

For `window`/`widget-realm-viewport` connector layers sourced from `Realm3D`,
connector input routing uses raycast mode automatically. Other layer kinds use
rect/clip hit-test routing.

For `RealmPlane`-style usage, when a 3D model material samples a texture bound to a
`texture` target that is also bound by a `TwoD` realm, pointer routing performs
raycast + hitbox test against the model plane and forwards events to that UI realm.

Routing is multi-hop across realms/targets in a single pointer event. Cycles are
supported using bounded propagation (`MAX_ROUTE_STEPS`) to avoid blocking.

Command failures (`success=false` responses) emit `SystemEvent::Error` with:
- `scope = "command"`
- `commandId` and `commandType` when available.

Auto-graph diagnostics may also emit `SystemEvent::Error` with:
- `scope = "target-auto-link"`
- reason payload encoded in `message`.
  - current reasons: `realm-not-found`, `target-not-found`, `host-realm-not-found`.

On `vulfram_receive_events`, the core:

1. Serializes `event_queue` into MessagePack (using `rmp-serde`).
2. Allocates a buffer, copies bytes, and exposes pointer & length.
3. Clears `event_queue` for the next frame.

---

## 9. Profiling Data

`ProfilingData` tracks these metrics:

- Timings (microseconds):
  - `commandProcessingUs`
  - `gamepadProcessingUs`
  - `eventLoopPumpUs`
  - `requestRedrawUs`
  - `serializationUs`
  - `renderTotalUs`
  - `renderShadowUs`
  - `renderWindowsUs`
  - `frameDeltaUs`
- Derived:
  - `fpsInstant`
- Per-window:
  - `windowFps[]` with `windowId`, `fpsInstant`, `frameDeltaUs`
- Counters:
  - `totalEventsDispatched`
  - `totalEventsCached`
- `frameReport` with the RealmGraph execution order, cut edges, cached surface
  entries, any throttled/no-progress realms, plus target auto-link diagnostics
  (`targetAutolinkFailures`).
  - `targetAutolinkFailures[]` entries: `{ realmId, targetId, reason }`.

On `vulfram_get_profiling`, the core:

1. Takes a snapshot of `ProfilingData`.
2. Serializes it into MessagePack.
3. Allocates and exposes the buffer via pointer & length.
