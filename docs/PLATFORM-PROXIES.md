# 🦊 Vulfram — Platform Proxies

This document describes how Vulfram separates platform-specific integration
(Desktop vs Browser) without changing the public API. The engine remains a black
box: the host always talks to the same `vulfram_*` functions and MessagePack
payloads.

---

## 1. Why Proxies Exist

Vulfram runs in different environments (native desktop, browser/WASM), but the
external contract must stay identical. Platform proxies isolate:

- Window creation and lifecycle
- Input plumbing (keyboard, pointer, touch, gamepad)
- Event loop pumping
- Surface configuration and redraw requests

This keeps the core stable while allowing new environments (mobile, consoles,
headless, etc.) to be added later.

---

## 2. Proxy Contract (Concept)

The core relies on a small, explicit interface that each platform implements.
At a high level, the proxy:

- Provides an event loop proxy (for system callbacks).
- Handles `CmdWindowCreate` requests.
- Pumps platform events each frame.
- Processes gamepad input.
- Triggers rendering or redraw requests.

In code this is expressed as a Rust trait (`PlatformProxy`) in
`crates/vulfram-runtime/src/core/platforms/mod.rs`, with platform-specific helpers
and policies extracted into the `vulfram-platform` crate.

---

## 3. DesktopProxy (Native)

**Location:** `crates/vulfram-runtime/src/core/platforms/desktop/`

Responsibilities:

- Creates and owns a `winit` event loop.
- Routes window creation through `EngineCustomEvents`.
- Processes keyboard/mouse/touch/gesture events from `winit`.
- Uses `DeviceEvent::MouseMotion` fallback for locked pointer motion.
- Uses `gilrs` for gamepad events.
- Requests redraws for each window during `vulfram_tick`.

This proxy is used by default when **not** compiling with the `wasm` feature.

---

## 4. BrowserProxy (WASM)

**Location:** `crates/vulfram-runtime/src/core/platforms/browser/`

Responsibilities:

- Creates window surfaces from a DOM canvas (`canvasId`).
- Attaches DOM event listeners for keyboard/pointer/scroll/focus.
- Integrates browser pointer lock (`locked`) and logical confined polyfill.
- Tracks explicit `canvas active` state separately from browser window focus.
- Gates action input (keyboard, pointer, touch, gamepad axes/buttons) so it only
  reaches the engine while the canvas is active.
- Emits `WindowEvent::OnCanvasActiveChange` so hosts can react to activation and
  deactivation transitions.
- Blocks browser page scroll/navigation input while the canvas is active,
  including wheel/touch scrolling and navigation keys such as arrows and page keys.
- Tracks canvas resize from CSS bounds × DPR to keep render surface/projection in sync.
- Polls the Web Gamepad API each tick, while still reporting connect/disconnect
  even when the canvas is inactive.
- Renders frames directly during `vulfram_tick` (no native event loop).

This proxy is selected when compiling with the `wasm` feature.

---

## 5. Selection and Build Rules

- **Desktop builds** (`ffi`, `napi`, `lua`, `python`) use `DesktopProxy`.
- **WASM builds** (`--features wasm`) use `BrowserProxy`.

Selection is compile-time, but the external ABI stays the same.

---

## 6. Extending to New Platforms

To add a new environment, implement a new proxy and wire it into
`crates/vulfram-runtime/src/core/platforms/mod.rs` (or a build-time selector).
The public API does not change, and the host logic remains untouched.

Potential future proxies:

- **Mobile** (iOS/Android via winit or native bindings)
- **Consoles** (vendor SDK integration)
- **Headless / Server** (offscreen rendering or compute-only)

---

## 7. Invariants (Black Box Contract)

No matter which proxy is active:

- The host sends commands via `vulfram_send_queue`.
- The host reads responses via `vulfram_receive_queue`.
- The host reads events via `vulfram_receive_events`.
- The host drives the frame via `vulfram_tick`.

The internal platform split is invisible to the host.

## 8. Window State Change Events

Both proxies emit explicit window-state transition events:

- `WindowEvent::OnStateChange` for lifecycle transitions (`windowed`, `fullscreen`, etc.).
- `WindowEvent::OnPointerCaptureChange` for pointer capture mode/activation transitions.
- `WindowEvent::OnCanvasActiveChange` for browser canvas activation state transitions.

This keeps host behavior deterministic across platform differences.
