use std::time::{Duration, Instant};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::platform::{EventLoop, EventLoopExtPumpEvents, EventLoopProxy};
use crate::core::realm::RealmId;
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
use crate::core::ui::types::{UiImageSource, UiNodeProps};
use crate::core::window::WindowEvent;
use crate::core::window::{CmdResultWindowCreate, CmdWindowCreateArgs};

use super::PlatformProxy;

pub mod handler;

pub struct DesktopProxy {
    event_loop: EventLoop<EngineCustomEvents>,
    proxy: EventLoopProxy<EngineCustomEvents>,
}

impl DesktopProxy {
    pub fn new() -> Self {
        let event_loop = EventLoop::<EngineCustomEvents>::with_user_event()
            .build()
            .unwrap();
        let proxy = event_loop.create_proxy();
        Self { event_loop, proxy }
    }
}

impl PlatformProxy for DesktopProxy {
    fn event_loop_proxy(&self) -> &EventLoopProxy<EngineCustomEvents> {
        &self.proxy
    }

    fn handle_window_create(
        &mut self,
        _state: &mut EngineState,
        cmd_id: u64,
        args: &CmdWindowCreateArgs,
    ) -> Result<(), CmdResultWindowCreate> {
        let _ = self
            .proxy
            .send_event(EngineCustomEvents::CreateWindow(cmd_id, args.clone()));
        Ok(())
    }

    fn process_gamepads(&mut self, state: &mut EngineState) -> u64 {
        let start = Instant::now();
        let has_focused_window = state
            .window
            .states
            .values()
            .any(|window_state| window_state.window.has_focus());
        if !has_focused_window {
            return start.elapsed().as_nanos() as u64;
        }
        let mut gilrs_events = Vec::new();
        if let Some(gilrs) = &mut state.gamepad.gilrs {
            while let Some(event) = gilrs.next_event() {
                gilrs_events.push(event);
            }
        }

        for event in gilrs_events {
            crate::core::gamepad::process_gilrs_event(state, event);
        }

        start.elapsed().as_nanos() as u64
    }

    fn pump_events(&mut self, state: &mut EngineState) -> u64 {
        let pump_start = Instant::now();
        self.event_loop
            .pump_app_events(Some(Duration::from_millis(16)), state);
        let total_pump_time = pump_start.elapsed().as_nanos() as u64;
        total_pump_time.saturating_sub(state.profiling.input.custom_events_ns)
    }

    fn render(&mut self, state: &mut EngineState) -> u64 {
        let start = Instant::now();
        let now_ms = state.time;
        let input_windows = active_windows_from_events(&state.event_queue);
        let pending_ui_image_windows = windows_with_pending_ui_images(state);
        let has_ui_animations = !state.universal_state.ui.animations.is_empty();
        let has_ui_repaint_request = state
            .universal_state
            .ui
            .realms
            .values()
            .any(|realm| realm.needs_repaint);
        let has_unbound_ui_async_loading = state.universal_state.ui.image_async.has_pending()
            && pending_ui_image_windows.is_empty();

        for window_id in &input_windows {
            if let Some(window_state) = state.window.states.get_mut(window_id) {
                window_state.redraw_force_until_ms = now_ms.saturating_add(250);
            }
        }
        for (window_id, window_state) in state.window.states.iter_mut() {
            let has_recent_input = now_ms <= window_state.redraw_force_until_ms;
            let should_redraw = window_state.is_dirty
                || has_recent_input
                || has_ui_animations
                || has_ui_repaint_request
                || state.texture_async.has_pending()
                || pending_ui_image_windows.contains(window_id)
                || has_unbound_ui_async_loading;
            if should_redraw {
                window_state.is_dirty = true;
                window_state.window.request_redraw();
            }
        }
        start.elapsed().as_nanos() as u64
    }
}

fn active_windows_from_events(events: &[EngineEvent]) -> std::collections::HashSet<u32> {
    let mut windows = std::collections::HashSet::new();
    for event in events {
        match event {
            EngineEvent::Pointer(pointer) => {
                let window_id = match pointer {
                    PointerEvent::OnMove { window_id, .. }
                    | PointerEvent::OnEnter { window_id, .. }
                    | PointerEvent::OnLeave { window_id, .. }
                    | PointerEvent::OnButton { window_id, .. }
                    | PointerEvent::OnScroll { window_id, .. }
                    | PointerEvent::OnTouch { window_id, .. }
                    | PointerEvent::OnPinchGesture { window_id, .. }
                    | PointerEvent::OnPanGesture { window_id, .. }
                    | PointerEvent::OnRotationGesture { window_id, .. }
                    | PointerEvent::OnDoubleTapGesture { window_id, .. } => *window_id,
                };
                windows.insert(window_id);
            }
            EngineEvent::Keyboard(KeyboardEvent::OnInput { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnModifiersChange { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImeEnable { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImePreedit { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImeCommit { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImeDisable { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnFocus { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnScaleFactorChange { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnResize { window_id, .. }) => {
                windows.insert(*window_id);
            }
            _ => {}
        }
    }
    windows
}

fn windows_with_pending_ui_images(state: &EngineState) -> std::collections::HashSet<u32> {
    let pending_image_ids = state.universal_state.ui.image_async.pending_image_ids();
    if pending_image_ids.is_empty() {
        return std::collections::HashSet::new();
    }

    let realm_windows: std::collections::HashMap<RealmId, u32> = state
        .universal_state
        .realms
        .entries
        .iter()
        .filter_map(|(realm_id, entry)| {
            entry
                .value
                .host_window_id
                .map(|window_id| (*realm_id, window_id))
        })
        .collect();

    let mut windows = std::collections::HashSet::new();
    for document in state.universal_state.ui.documents.values() {
        let Some(window_id) = realm_windows.get(&document.realm_id).copied() else {
            continue;
        };

        let mut found_pending_in_document = false;
        for node_entry in document.nodes.values() {
            let image_id = match &node_entry.node.props {
                UiNodeProps::Image {
                    source: UiImageSource::UiImage(image_id),
                    ..
                }
                | UiNodeProps::ImageButton {
                    source: UiImageSource::UiImage(image_id),
                    ..
                } => Some(*image_id),
                _ => None,
            };
            if image_id.is_some_and(|id| pending_image_ids.contains(&id)) {
                found_pending_in_document = true;
                break;
            }
        }
        if found_pending_in_document {
            windows.insert(window_id);
        }
    }

    windows
}
