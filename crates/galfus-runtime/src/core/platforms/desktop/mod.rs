use std::time::{Duration, Instant};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::platform::{EventLoop, EventLoopExtPumpEvents, EventLoopProxy};
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
use crate::core::window::WindowEvent;
use crate::core::window::{CmdResultWindowCreate, CmdWindowCreateArgs};
use galfus_input::{
    connect_gamepad, disconnect_gamepad, update_gamepad_axis, update_gamepad_button,
};
use galfus_platform::{
    PlatformActivityEvent, RedrawContext, WindowRedrawInput, drain_gilrs_events, map_gilrs_axis,
    map_gilrs_button, plan_window_redraws, resolve_gilrs_gamepad_name,
};

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
        for event in drain_gilrs_events(&mut state.gamepad_backend) {
            let gamepad_id: u32 = usize::from(event.id) as u32;
            match event.event {
                galfus_platform::PlatformGamepadEventType::Connected => {
                    let name = resolve_gilrs_gamepad_name(&state.gamepad_backend, event.id)
                        .unwrap_or_else(|| "Unknown".into());
                    if let Some(gamepad_event) =
                        connect_gamepad(&mut state.gamepad.cache, gamepad_id, name)
                    {
                        state
                            .runtime
                            .push_event(EngineEvent::Gamepad(gamepad_event));
                    }
                }
                galfus_platform::PlatformGamepadEventType::Disconnected => {
                    if let Some(gamepad_event) =
                        disconnect_gamepad(&mut state.gamepad.cache, gamepad_id)
                    {
                        state
                            .runtime
                            .push_event(EngineEvent::Gamepad(gamepad_event));
                    }
                }
                galfus_platform::PlatformGamepadEventType::ButtonPressed(button, _code) => {
                    if let Some(gamepad_event) = update_gamepad_button(
                        &mut state.gamepad.cache,
                        gamepad_id,
                        map_gilrs_button(button),
                        1.0,
                    ) {
                        state
                            .runtime
                            .push_event(EngineEvent::Gamepad(gamepad_event));
                    }
                }
                galfus_platform::PlatformGamepadEventType::ButtonReleased(button, _code) => {
                    if let Some(gamepad_event) = update_gamepad_button(
                        &mut state.gamepad.cache,
                        gamepad_id,
                        map_gilrs_button(button),
                        0.0,
                    ) {
                        state
                            .runtime
                            .push_event(EngineEvent::Gamepad(gamepad_event));
                    }
                }
                galfus_platform::PlatformGamepadEventType::ButtonChanged(button, value, _code) => {
                    if let Some(gamepad_event) = update_gamepad_button(
                        &mut state.gamepad.cache,
                        gamepad_id,
                        map_gilrs_button(button),
                        value,
                    ) {
                        state
                            .runtime
                            .push_event(EngineEvent::Gamepad(gamepad_event));
                    }
                }
                galfus_platform::PlatformGamepadEventType::AxisChanged(axis, value, _code) => {
                    if let Some(gamepad_event) = update_gamepad_axis(
                        &mut state.gamepad.cache,
                        gamepad_id,
                        map_gilrs_axis(axis),
                        value,
                    ) {
                        state
                            .runtime
                            .push_event(EngineEvent::Gamepad(gamepad_event));
                    }
                }
                _ => {}
            }
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
        let now_ms = state.runtime.time_ms();
        let activity_events = collect_platform_activity_events(state.runtime.events());
        let pending_ui_image_windows = windows_with_pending_ui_images(state);
        let has_ui_animations = false;
        let has_ui_repaint_request = false;
        let has_unbound_ui_async_loading = false && pending_ui_image_windows.is_empty();
        let redraw_inputs: Vec<_> = state
            .window
            .states
            .iter()
            .map(|(&window_id, window_state)| WindowRedrawInput {
                window_id,
                redraw_force_until_ms: window_state.redraw_force_until_ms,
                is_dirty: window_state.is_dirty,
            })
            .collect();
        let redraw_plans = plan_window_redraws(
            &activity_events,
            &pending_ui_image_windows,
            &redraw_inputs,
            RedrawContext {
                now_ms,
                had_commands_this_frame: state.runtime.had_commands_this_frame(),
                has_ui_animations,
                has_ui_repaint_request,
                has_pending_texture_work: state.texture_async.has_pending(),
                has_unbound_ui_async_loading,
            },
        );
        for plan in redraw_plans {
            let Some(window_state) = state.window.states.get_mut(&plan.window_id) else {
                continue;
            };
            window_state.redraw_force_until_ms = plan.redraw_force_until_ms;
            if plan.should_request_redraw {
                window_state.is_dirty = true;
                window_state.window.request_redraw();
            }
        }
        start.elapsed().as_nanos() as u64
    }
}

fn collect_platform_activity_events(events: &[EngineEvent]) -> Vec<PlatformActivityEvent> {
    events
        .iter()
        .filter_map(|event| match event {
            EngineEvent::Pointer(pointer) => Some(PlatformActivityEvent::Pointer {
                window_id: match pointer {
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
                },
            }),
            EngineEvent::Keyboard(KeyboardEvent::OnInput { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnModifiersChange { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImeEnable { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImePreedit { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImeCommit { window_id, .. })
            | EngineEvent::Keyboard(KeyboardEvent::OnImeDisable { window_id, .. }) => {
                Some(PlatformActivityEvent::Keyboard {
                    window_id: *window_id,
                })
            }
            EngineEvent::Window(WindowEvent::OnFocus { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnScaleFactorChange { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnResize { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnStateChange { window_id, .. })
            | EngineEvent::Window(WindowEvent::OnPointerCaptureChange { window_id, .. }) => {
                Some(PlatformActivityEvent::Window {
                    window_id: *window_id,
                })
            }
            _ => None,
        })
        .collect()
}

fn windows_with_pending_ui_images(state: &EngineState) -> std::collections::HashSet<u32> {
    let _state = state;
    std::collections::HashSet::new()
}
