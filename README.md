<div align="center">
  <img src="./assets/brand.svg" alt="Vulfram" width="400" />
  
  # Vulfram
  
  **High-Performance Rendering & Systems Core powered by WebGPU**
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)
  [![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
</div>

---

## 📋 Table of Contents

- [About Vulfram](#-about-vulfram)
- [Design Philosophy](#-design-philosophy)
- [Architecture](#️-architecture)
- [Key Concepts](#-key-concepts)
- [Quick Start](#-quick-start)
- [Main Features](#-main-features)
- [Development](#️-development)
- [Project Structure](#-project-structure)
- [Documentation](#-documentation)
- [Contributing](#-contributing)
- [License](#-license)
- [About Vulppi](#-about-vulppi)

---

## 🦊 About Vulfram

**Vulfram** is a **rendering and systems core** written in Rust and exposed as a dynamic library. The name combines "Vulppi" (derived from _Vulpes_, the scientific name for fox) and "Frame", representing our mission to create perfect frames for incredible interactive experiences.

Vulfram is designed to be **host-agnostic** and driven by external runtimes via FFI or WASM:

- 🟢 **Node.js** (N-API)
- 🌙 **Lua** (via `mlua`)
- 🐍 **Python** (via `PyO3`)
- 🔧 Any environment capable of calling C-ABI functions
- 🌐 Browser runtimes via WASM (WebGPU + DOM canvas)

### Core Features

- 🚀 **High Performance**: GPU-accelerated rendering with WGPU (WebGPU)
- 🔄 **Cross-Platform**: Native support for Windows, macOS, and Linux
- 🎮 **Complete Input System**: Keyboard, mouse, touch, and gamepads (via Gilrs)
- 🪟 **Advanced Window Management**: Full control over multiple windows (via Winit) or a DOM canvas (WASM)
- 💡 **Lighting & Shadows**: Support for various light types and shadow mapping
- 🎨 **Materials & Textures**: Flexible resource management for rendering
- 🔌 **Language Bindings**: N-API (Node.js), Lua, Python, and more. With C-ABI, `bun:ffi` is also possible.
- ⚡ **MessagePack Communication**: Fast binary serialization for commands and events
- 🎯 **Host-Agnostic Design**: No assumptions about ECS, OOP, or game framework

---

## 💡 Design Philosophy

Vulfram follows a **black box** approach where:

> The host controls the engine **only** through:
>
> - A small set of C functions (`vulfram_*`)
> - Binary buffers serialized with **MessagePack**

**Design Goals:**

- **Host-agnostic**: Works with any game framework or architecture
- **Minimal public surface**: Only essential C-ABI functions exposed
- **Binary and fast**: MessagePack for structured data, raw bytes for heavy assets
- **Clear separation**: Host manages logic and IDs, Core manages GPU and rendering
- **Async linking**: Components/resources can be created in any order, using fallbacks until data is available

---

## 🏗️ Architecture

Vulfram uses a queue-based architecture that enables efficient communication between the host and the Rust core:

```

## Workspace Layout

The monorepo is split by ecosystem:

- `crates/`
  Rust workspace crates for runtime, render, realms, bindings and demos
- `packages/`
  Bun/TypeScript workspace packages for transports and host-facing JS runtimes
- `scripts/`
  repository automation executed through Bun

### Repository Scripts

- `bun run check`
  runs the standard Rust/WGSL validation pipeline
- `bun run artifacts -- --skip-download`
  skips artifact download explicitly via CLI flag
- `bun run artifacts -- --base-url <url>`
  overrides the artifact source URL without environment variables
- `bun run version -- <semver>`
  updates Cargo workspace version and transport package versions, except
  `transport-types`
- `bun run release-meta -- --ref <channel>/vX.Y.Z`
  resolves the package version, artifact version, npm dist-tag, and GitHub release tag
┌─────────────────────────────────────┐
│          Host Layer                 │
│  (JS/TS, Lua, Python, etc.)         │
│  • Game Logic                       │
│  • Entity Management                │
│  • ID Generation                    │
└────────┬────────────┬───────────────┘
         │            │
         │ Commands   │ Events
         │ (MsgPack)  │ (MsgPack)
         ▼            ▲
    ┌────────────────────────────┐
    │  vulfram_send_queue()      │
    │  vulfram_receive_queue()   │
    │  vulfram_receive_events()  │
    │  vulfram_upload_buffer()   │
    │  vulfram_tick()            │
    └──────────┬─────────────────┘
               │
┌──────────────▼───────────────────────┐
│        Vulfram Core (Rust)           │
│  • Resource Management               │
│  • Component Instances               │
│  • Platform Proxy (Desktop/Browser)  │
│  • GPU Rendering (WGPU)              │
│  • Input Processing (Gilrs/Web)      │
└──────────────┬───────────────────────┘
               │
┌──────────────▼───────────────────────┐
│           GPU Layer                  │
│  Vulkan / Metal / DirectX 12         │
└──────────────────────────────────────┘
```

### Responsibilities

**Host (Your Game/App):**

- Manage game logic and world state
- Generate logical IDs (entities, resources, buffers)
- Build MessagePack command batches
- Drive the main loop with `vulfram_tick()`
- Process events and responses

**Vulfram Core:**

- Abstract window, input, and rendering systems via platform proxies
- Manage GPU resources and pipelines using WGPU
- Track component instances (cameras, models, etc.)
- Translate commands into internal state changes
- Render frames efficiently

---

## 🔑 Key Concepts

### Components vs Resources

Vulfram distinguishes between two fundamental types:

**Components** - High-level structures describing scene participation:

- Always attached to a host-chosen ID (e.g. `camera_id`, `model_id`, `light_id`)
- Examples: `Camera`, `Model`, `Light`
- Can contain static data (local colors, matrices)
- Reference sharable resources by logical ID
- Created/updated via MessagePack commands

**Resources** - Reusable assets used by components:

- Identified by logical IDs: `GeometryId`, `MaterialId`, `TextureId`
- Sharable across multiple components/entities
- Have internal GPU handles (buffers, textures, pipelines)
- Examples: geometries, textures, materials

### Logical IDs

The host manages all logical identifiers:

- `WindowId` - Window instances
- `RealmId` - Realm instances
- `TargetId` - Logical composition targets
- `CameraId` - Camera instances
- `ModelId` - Model instances
- `LightId` - Light instances
- `GeometryId` - Mesh/geometry assets
- `MaterialId` - Material configurations
- `TextureId` - Texture assets
- `BufferId` - Upload blob identifiers

These are opaque integers to the core. The host ensures uniqueness and consistency.

### Internal Composition Tables

The core derives and owns these runtime tables internally:

- `Surface`
- `Present`
- `Connector`

The host composes output through `Realm`, `Target`, and `TargetLayer` commands
instead of managing these internal tables directly.

### Asynchronous Resource Linking (Fallback-Driven)

Vulfram allows resources to be created out of order:

- Models can reference geometry or material IDs that do not exist yet.
- Materials can reference texture IDs that do not exist yet.

In these cases, Vulfram renders using fallback resources until the real
resources are created later with the same IDs. When a referenced resource
appears, it is picked up automatically on the next frame without recreating
the model or material.

This enables async streaming and decouples creation order.

### Layer Masking

Vulfram uses `u32` bitmasks for visibility control:

- `layerMaskCamera` - Specifies which layers a camera can see
- `layerMaskComponent` - Specifies which layers a model belongs to
- `layerMaskLight` - (Future) Which layers a light affects

**Visibility Rule:**

```text
Visible if: (layerMaskCamera & layerMaskComponent) > 0
```

This enables:

- World-only or UI-only cameras
- Team-based rendering
- Debug geometry filtering
- Multi-pass rendering strategies

### Reuse Semantics

- A single geometry can be referenced by many models.
- A single material can be referenced by many models.
- A single texture can be referenced by many materials.

There is no ownership tracking. If a resource is disposed while still referenced,
rendering falls back gracefully.

### Render Ordering & Batching

Per camera:

- Opaque/masked objects are sorted by `material_id` then `geometry_id` to reduce
  state changes and batch draw calls.
- Transparent objects are sorted by depth for correct blending.

Draw calls are batched by runs of `(material_id, geometry_id)` after sorting.

### Upload System

Heavy data uses one-shot uploads:

1. Host calls `vulframUploadBuffer(bufferId, type, data)`
2. Core stores blob in internal upload table
3. `Create*` commands reference `bufferId` to consume data
4. Entry is marked as used and can be removed
5. `CmdUploadBufferDiscardAll` command clears all pending uploads

Uploads are independent of model/material creation, so you can create
components first and upload data later.

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **Vulkan**, **Metal**, or **DirectX 12** updated drivers

### Quick Start (Demo Harness)

```bash
# Clone the repository
git clone https://github.com/vulppi-dev/vulfram.git
cd vulfram

# Build and run the first demo
cargo run -p vulfram-demo -- 1
```

The demo harness lives in `crates/vulfram-demo` and exercises:

- window creation
- primitive geometry creation
- camera + model setup
- basic rendering loop

---

## 📚 Main Features

### Window Management

- Multiple window creation and management
- Window state control (normal, minimized, maximized, fullscreen)
- Position and size configuration
- Borderless and resizable options
- Custom window icons and cursors
- Drag-and-drop file support
- Window events (resize, move, focus, close)

### Input System

- **Keyboard**: Physical key events with modifiers and IME support
- **Mouse**: Movement, buttons, scroll wheel
- **Touch**: Multi-touch support with gestures (pinch, pan, rotate) on native
- **Pointer**: Unified API for mouse/touch/pen via `PointerEvent`
- **Gamepad**: Automatic detection, buttons, analog sticks, triggers (Gilrs on native, Gamepad API on web)
  - Standard mapping with dead zones
  - Change threshold filtering for efficient event generation

### Rendering (WGPU)

- GPU-accelerated rendering via WebGPU
- Cross-platform support (Vulkan, Metal, DirectX 12)
- Buffer upload for textures and meshes
- Efficient CPU-GPU synchronization
- Component-based rendering system
- Layer-based visibility control

### Communication

- **MessagePack** serialization for fast binary communication
- Separate queues for:
  - Commands (host → core)
  - Responses (core → host)
  - Events (core → host)
- Profiling data export for performance analysis

### Performance Optimizations

- **Event caching**: Filters duplicate events to reduce overhead
- **Redraw optimization**: Dirty flag system for selective rendering
- **Dead zone filtering**: Reduces gamepad noise
- **Batch processing**: Commands processed in bulk for efficiency

---

## 🛠️ Development

```bash
# Build the workspace
cargo build --workspace --release

# Run tests
cargo test

# Check code with Clippy
cargo clippy

# Format code
cargo fmt
```

### Recommended Development Loop

```text
1. Update host-side logic (game state, entities)
2. Upload heavy data (optional) via `vulfram_upload_buffer()`
3. Send command batch via `vulfram_send_queue()`
4. Advance the core via `vulfram_tick(time, delta_time)`
5. Receive responses via `vulfram_receive_queue()`
6. Receive events via `vulfram_receive_events()`
7. Read profiling data (optional) via `vulfram_get_profiling()`
```

`vulfram_receive_queue()` consumes and clears the internal response queue.

---

## 📦 Project Structure

```text
vulfram/
├── crates/
│   ├── vulfram-runtime/         # Integration root, ABI re-exports and frame orchestration
│   ├── vulfram-types/           # Shared logical/base types
│   ├── vulfram-protocol/        # Host/runtime contracts + MsgPack codec
│   ├── vulfram-realm-core/      # Realm composition semantics and graph/report DTOs
│   ├── vulfram-input/           # Normalized input semantics
│   ├── vulfram-realm-ui/        # UI semantic layer
│   ├── vulfram-render/          # WGPU-facing render policy, graphs and planning helpers
│   ├── vulfram-audio/           # Audio domain + backends
│   ├── vulfram-platform/        # Desktop/browser integration
│   ├── vulfram-realm-3d/        # 3D realm semantics + sync plans
│   ├── vulfram-realm-2d/        # 2D realm placeholder/contracts
│   ├── vulfram-bindings-ffi/    # C ABI host binding
│   ├── vulfram-bindings-wasm/   # wasm-bindgen host binding
│   ├── vulfram-bindings-napi/   # Node.js binding
│   ├── vulfram-bindings-python/ # Python binding
│   ├── vulfram-bindings-lua/    # Lua binding
│   └── vulfram-demo/            # Visual/manual validation demos
├── docs/
├── assets/
├── scripts/
├── Cargo.toml
└── README.md
```

---

## 📖 Documentation

Comprehensive documentation is available in the `docs/` folder.

### 📑 [Documentation Index](docs/INDEX.md)

**Not sure where to start?** Check our [complete documentation index](docs/INDEX.md) for guided navigation based on your role and needs.

### Quick Links

**For Engine Users (Binding Authors):**

1. **[OVERVIEW.md](docs/OVERVIEW.md)** - Start here! High-level concepts and design philosophy
2. **[ABI.md](docs/ABI.md)** - C-ABI functions, usage contract, and calling conventions
3. **[ARCH.md](docs/ARCH.md)** - Architecture, lifecycle, and main loop patterns
4. **[REALM-ARCH.md](docs/REALM-ARCH.md)** - Realm composition architecture and refactor direction
5. **[GLOSSARY.md](docs/GLOSSARY.md)** - Terminology and naming conventions

**For Core Contributors (Rust Developers):**

1. **[OVERVIEW.md](docs/OVERVIEW.md)** - Core concepts and design
2. **[ARCH.md](docs/ARCH.md)** - System architecture
3. **[API.md](docs/API.md)** - Internal Rust API, crates, and data structures
4. **[GLOSSARY.md](docs/GLOSSARY.md)** - Internal terminology

**Additional Resources:**

- **[UI.md](docs/UI.md)** - UI runtime technical index
- **[ui/README.md](docs/ui/README.md)** - RealmUI overview and subsystem docs
- **[PLATFORM-PROXIES.md](docs/PLATFORM-PROXIES.md)** - Platform proxy architecture
- **[VALIDATION.md](docs/VALIDATION.md)** - Automated tests and manual demo validation scope
- **[Copilot Instructions](.github/copilot-instructions.md)** - Development patterns and memory

Automated tests do not fully cover perceptual/platform-dependent flows (window lifecycle details, audio audibility, visual quality). Those are validated manually by running demos as defined in `docs/VALIDATION.md`.

---

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

### Code Style

- **Rust code**: Minimal comments, self-descriptive names
- **All code**: English for variables, functions, types, and comments
- **Documentation**: English for all docs (README, API docs)
- **Communication**: Brazilian Portuguese for discussions and issues

### Development Process

1. Fork the project
2. Create a feature branch (`git checkout -b feature/MyFeature`)
3. Follow the project's coding conventions (see `.github/copilot-instructions.md`)
4. Write tests for your changes
5. Ensure all tests pass (`cargo test`)
6. Format your code (`cargo fmt`)
7. Check for issues (`cargo clippy`)
8. Commit your changes (`git commit -m 'Add MyFeature'`)
9. Push to your branch (`git push origin feature/MyFeature`)
10. Open a Pull Request

### Architecture Guidelines

- Keep the C-ABI surface minimal
- Use MessagePack for all structured data crossing the ABI
- Maintain clear separation between host and core responsibilities
- Follow the component/resource model
- Ensure thread safety (all functions are main-thread only)

---

## 🏛️ Core Technology Stack

### Rust Dependencies

- **`wgpu`** - GPU abstraction layer (WebGPU implementation)
- **`winit`** - Cross-platform windowing
- **`gilrs`** - Gamepad input
- **`napi`** - Node.js N-API bindings (optional)
- **`serde`** - Serialization framework
- **`rmp-serde`** - MessagePack serialization
- **`glam`** - Vector and matrix math
- **`bytemuck`** - Safe type conversions for GPU data
- **`image`** - Image loading and decoding

Bindings now live in dedicated workspace crates:
`vulfram-bindings-ffi`, `vulfram-bindings-wasm`, `vulfram-bindings-napi`,
`vulfram-bindings-lua`, and `vulfram-bindings-python`.

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE.md](LICENSE.md) file for details.

## 🦊 About Vulppi

**Vulppi** is a company focused on creating cutting-edge technologies for game development and interactive applications. Our name comes from _Vulpes_, the scientific name for fox, symbolizing agility, intelligence, and adaptability.

---

<div align="center">
  Made with ❤️ by the Vulppi team
</div>
