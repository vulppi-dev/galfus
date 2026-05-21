use crate::core::platform::winit;
use crate::core::platform::winit::event::DeviceEvent as WinitDeviceEvent;
use crate::core::platform::winit::event::WindowEvent as WinitWindowEvent;
use crate::core::platform::{ActiveEventLoop, ApplicationHandler, WindowId};
use galfus_input::{
    element_state_from_pressed, keyboard_ime_commit_event, keyboard_ime_disable_event,
    keyboard_ime_enable_event, keyboard_ime_preedit_event, keyboard_input_event,
    keyboard_modifiers_event, pointer_button_event, pointer_double_tap_gesture_event,
    pointer_enter_event, pointer_leave_event, pointer_move_event, pointer_pan_gesture_event,
    pointer_pinch_gesture_event, pointer_rotation_gesture_event, pointer_scroll_event,
    pointer_touch_event,
};
use galfus_platform::{
    PlatformFullscreenMode, PlatformWindowLifecycleState, map_winit_key_location,
    map_winit_mouse_button, map_winit_physical_key_code, map_winit_touch_phase,
    resolve_platform_window_state,
};
use glam::{IVec2, UVec2, Vec2};

use crate::core::input::{ModifiersState, ScrollDelta};
use crate::core::render::render_frames;
use crate::core::system::SystemEvent;
use crate::core::window::engine_cmd_window_create;
use crate::core::window::{
    CursorGrabMode, EngineWindowState, WindowEvent, WindowPointerCaptureState,
};

use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineEvent};
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;

fn read_window_lifecycle_state(
    window_state: &crate::core::window::WindowState,
) -> EngineWindowState {
    let fullscreen = match window_state.window.fullscreen() {
        Some(winit::window::Fullscreen::Exclusive(_)) => Some(PlatformFullscreenMode::Exclusive),
        Some(winit::window::Fullscreen::Borderless(_)) => Some(PlatformFullscreenMode::Borderless),
        None => None,
    };
    match resolve_platform_window_state(
        window_state.window.is_minimized().unwrap_or(false),
        window_state.window.is_maximized(),
        fullscreen,
    ) {
        PlatformWindowLifecycleState::Windowed => EngineWindowState::Windowed,
        PlatformWindowLifecycleState::Fullscreen => EngineWindowState::Fullscreen,
        PlatformWindowLifecycleState::WindowedFullscreen => EngineWindowState::WindowedFullscreen,
        PlatformWindowLifecycleState::Maximized => EngineWindowState::Maximized,
        PlatformWindowLifecycleState::Minimized => EngineWindowState::Minimized,
    }
}

fn emit_window_state_change_if_needed(engine: &mut EngineState, window_id: u32) {
    let Some(window_state) = engine.window.states.get(&window_id) else {
        return;
    };
    let next_state = read_window_lifecycle_state(window_state);
    if engine.window.set_lifecycle_state(window_id, next_state) {
        engine
            .runtime
            .push_event(EngineEvent::Window(WindowEvent::OnStateChange {
                window_id,
                state: next_state,
            }));
    }
}

impl ApplicationHandler<EngineCustomEvents> for EngineState {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime
            .push_event(EngineEvent::System(SystemEvent::OnResume));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime
            .push_event(EngineEvent::System(SystemEvent::OnSuspend));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime
            .push_event(EngineEvent::System(SystemEvent::OnExit));
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime
            .push_event(EngineEvent::System(SystemEvent::OnMemoryWarning));
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        winit_window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        let window_id = match self.window.resolve_window_id(&winit_window_id) {
            Some(id) => id,
            None => return,
        };

        match event {
            WinitWindowEvent::Resized(size) => {
                let new_size = UVec2::new(size.width, size.height);
                let cache = self.window.cache.get_or_create(window_id);

                // Only dispatch event if size actually changed
                if !cache.size_changed(new_size) {
                    self.profiling.input.total_events_cached += 1;
                    return;
                }

                // Update cache
                cache.inner_size = new_size;

                // Update surface configuration with new size
                if let Some(window_state) = self.window.states.get_mut(&window_id) {
                    if size.width > 0 && size.height > 0 {
                        window_state.config.width = size.width;
                        window_state.config.height = size.height;
                        // SAFETY: device is always Some after initialization
                        let device = unsafe { self.device.as_ref().unwrap_unchecked() };
                        window_state.surface.configure(device, &window_state.config);

                        // Update camera targets and projections
                        if let Some(render_state) = self.render.get_mut(&window_id) {
                            render_state.on_resize(device, size.width, size.height);
                        }
                        crate::core::resources::ensure_render_target(
                            device,
                            &mut window_state.surface_target,
                            size.width.max(1),
                            size.height.max(1),
                            wgpu::TextureFormat::Rgba16Float,
                        );

                        // Update size state
                        window_state.inner_size = new_size;
                        let outer_size = window_state.window.outer_size();
                        window_state.outer_size = UVec2::new(outer_size.width, outer_size.height);
                        cache.outer_size = UVec2::new(outer_size.width, outer_size.height);

                        if let Some(surface_id) = self
                            .universal_state
                            .composition
                            .presents
                            .entries
                            .values()
                            .find(|present| present.value.window_id == window_id)
                            .map(|present| present.value.surface)
                        {
                            if let Some(surface_entry) = self
                                .universal_state
                                .composition
                                .surfaces
                                .entries
                                .get_mut(&surface_id)
                            {
                                surface_entry.value.size = new_size;
                            }
                        }

                        // Mark window as dirty to trigger redraw
                        window_state.is_dirty = true;
                    }
                }

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnResize {
                        window_id,
                        width: size.width,
                        height: size.height,
                    }));
                emit_window_state_change_if_needed(self, window_id);
            }

            WinitWindowEvent::Moved(position) => {
                let new_pos = IVec2::new(position.x, position.y);
                let cache = self.window.cache.get_or_create(window_id);

                // Only dispatch event if position actually changed
                if !cache.position_changed(new_pos) {
                    self.profiling.input.total_events_cached += 1;
                    return;
                }

                // Update cache
                cache.inner_position = new_pos;

                // Update window state
                if let Some(window_state) = self.window.states.get_mut(&window_id) {
                    window_state.inner_position = new_pos;
                    if let Ok(outer_pos) = window_state.window.outer_position() {
                        window_state.outer_position = IVec2::new(outer_pos.x, outer_pos.y);
                        cache.outer_position = IVec2::new(outer_pos.x, outer_pos.y);
                    }
                }

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnMove {
                        window_id,
                        position: new_pos,
                    }));
            }

            WinitWindowEvent::CloseRequested => {
                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnCloseRequest {
                        window_id,
                    }));
            }

            WinitWindowEvent::Destroyed => {
                // Cleanup all window resources when destroyed by system
                self.cleanup_window(window_id);

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnDestroy { window_id }));
            }

            WinitWindowEvent::DroppedFile(path) => {
                // Get the last known cursor position for this window
                let position = self
                    .window
                    .cursor_positions
                    .get(&window_id)
                    .copied()
                    .unwrap_or(Vec2::new(0.0, 0.0));

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnFileDrop {
                        window_id,
                        path: path.to_string_lossy().into_owned(),
                        position,
                    }));
            }

            WinitWindowEvent::HoveredFile(path) => {
                // Get the last known cursor position for this window
                let position = self
                    .window
                    .cursor_positions
                    .get(&window_id)
                    .copied()
                    .unwrap_or(Vec2::new(0.0, 0.0));

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnFileHover {
                        window_id,
                        path: path.to_string_lossy().into_owned(),
                        position,
                    }));
            }

            WinitWindowEvent::HoveredFileCancelled => {
                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnFileHoverCancel {
                        window_id,
                    }));
            }

            WinitWindowEvent::Focused(focused) => {
                let cache = self.window.cache.get_or_create(window_id);

                // Only dispatch event if focus state actually changed
                if cache.focused == focused {
                    self.profiling.input.total_events_cached += 1;
                    return;
                }

                // Update cache
                cache.focused = focused;

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnFocus {
                        window_id,
                        focused,
                    }));
                let mode = self.window.cursor_grab_mode(window_id);
                if mode != CursorGrabMode::None {
                    let active = focused;
                    if self.window.set_pointer_capture_active(window_id, active) {
                        self.runtime.push_event(EngineEvent::Window(
                            WindowEvent::OnPointerCaptureChange {
                                window_id,
                                capture: WindowPointerCaptureState {
                                    mode,
                                    active,
                                    reason: Some(if focused {
                                        "focus-gained".into()
                                    } else {
                                        "focus-lost".into()
                                    }),
                                },
                            },
                        ));
                    }
                }
                emit_window_state_change_if_needed(self, window_id);
            }

            WinitWindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if is_synthetic {
                    return;
                }

                let key_code = map_winit_physical_key_code(&event.physical_key);
                let location = map_winit_key_location(event.location);
                let state = element_state_from_pressed(event.state.is_pressed());

                self.runtime
                    .push_event(EngineEvent::Keyboard(keyboard_input_event(
                        window_id,
                        key_code,
                        state,
                        location,
                        event.repeat,
                        event.text.map(|s| s.into()),
                        self.input.modifiers,
                    )));
            }

            WinitWindowEvent::ModifiersChanged(modifiers) => {
                let new_modifiers = ModifiersState {
                    shift: modifiers.state().shift_key(),
                    ctrl: modifiers.state().control_key(),
                    alt: modifiers.state().alt_key(),
                    meta: modifiers.state().super_key(),
                };

                // Only dispatch event if modifiers actually changed
                if self.input.cache.keyboard.modifiers == new_modifiers {
                    self.profiling.input.total_events_cached += 1;
                    return;
                }

                // Update cache and state
                self.input.cache.keyboard.modifiers = new_modifiers;
                self.input.modifiers = new_modifiers;

                self.runtime
                    .push_event(EngineEvent::Keyboard(keyboard_modifiers_event(
                        window_id,
                        new_modifiers,
                    )));
            }

            WinitWindowEvent::Ime(ime) => {
                let ime_event = match ime {
                    winit::event::Ime::Enabled => keyboard_ime_enable_event(window_id),
                    winit::event::Ime::Preedit(text, cursor) => {
                        keyboard_ime_preedit_event(window_id, text, cursor)
                    }
                    winit::event::Ime::Commit(text) => keyboard_ime_commit_event(window_id, text),
                    winit::event::Ime::Disabled => keyboard_ime_disable_event(window_id),
                };
                self.runtime.push_event(EngineEvent::Keyboard(ime_event));
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                let cursor_pos = Vec2::new(position.x as f32, position.y as f32);
                let pointer_cache = self.input.cache.get_or_create_pointer(window_id);

                // Only dispatch event if position changed more than 1px
                if !pointer_cache.position_changed(cursor_pos) {
                    return;
                }

                // Update cache and state
                pointer_cache.position = cursor_pos;
                self.window.cursor_positions.insert(window_id, cursor_pos);

                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_move_event(
                        window_id, 0, 0, cursor_pos, None,
                    )));
            }

            WinitWindowEvent::CursorEntered { .. } => {
                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_enter_event(
                        window_id, 0, 0, None,
                    )));
            }

            WinitWindowEvent::CursorLeft { .. } => {
                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_leave_event(
                        window_id, 0, 0, None,
                    )));
            }

            WinitWindowEvent::MouseWheel { delta, phase, .. } => {
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        ScrollDelta::Line(Vec2::new(x, y))
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        ScrollDelta::Pixel(Vec2::new(pos.x as f32, pos.y as f32))
                    }
                };
                let touch_phase = map_winit_touch_phase(phase);

                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_scroll_event(
                        window_id,
                        scroll_delta,
                        touch_phase,
                        None,
                    )));
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                let btn = map_winit_mouse_button(button);
                let elem_state = element_state_from_pressed(state.is_pressed());

                // Get the last known cursor position for this window
                let position = self
                    .window
                    .cursor_positions
                    .get(&window_id)
                    .copied()
                    .unwrap_or(Vec2::new(0.0, 0.0));

                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_button_event(
                        window_id, 0, 0, btn, elem_state, position, None,
                    )));
            }

            WinitWindowEvent::PinchGesture { delta, phase, .. } => {
                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_pinch_gesture_event(
                        window_id,
                        delta,
                        map_winit_touch_phase(phase),
                        None,
                    )));
            }

            WinitWindowEvent::PanGesture { delta, phase, .. } => {
                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_pan_gesture_event(
                        window_id,
                        Vec2::new(delta.x, delta.y),
                        map_winit_touch_phase(phase),
                        None,
                    )));
            }

            WinitWindowEvent::RotationGesture { delta, phase, .. } => {
                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_rotation_gesture_event(
                        window_id,
                        delta,
                        map_winit_touch_phase(phase),
                        None,
                    )));
            }

            WinitWindowEvent::DoubleTapGesture { .. } => {
                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_double_tap_gesture_event(
                        window_id, None,
                    )));
            }

            WinitWindowEvent::Touch(touch) => {
                let phase = map_winit_touch_phase(touch.phase);
                let pressure = touch.force.map(|f| f.normalized() as f32);

                self.runtime
                    .push_event(EngineEvent::Pointer(pointer_touch_event(
                        window_id,
                        touch.id,
                        phase,
                        Vec2::new(touch.location.x as f32, touch.location.y as f32),
                        pressure,
                        None,
                    )));
            }

            WinitWindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                let cache = self.window.cache.get_or_create(window_id);

                // Only dispatch event if scale factor actually changed
                if !cache.scale_factor_changed(scale_factor) {
                    return;
                }

                // Update cache
                cache.scale_factor = scale_factor;

                // Get the current window inner size for the event
                let (new_width, new_height) = self
                    .window
                    .states
                    .get(&window_id)
                    .map(|ws| {
                        let size = ws.window.inner_size();
                        (size.width, size.height)
                    })
                    .unwrap_or((0, 0));

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnScaleFactorChange {
                        window_id,
                        scale_factor,
                        new_width,
                        new_height,
                    }));
            }

            WinitWindowEvent::ThemeChanged(theme) => {
                let dark_mode = matches!(theme, winit::window::Theme::Dark);
                let cache = self.window.cache.get_or_create(window_id);

                // Only dispatch event if theme actually changed
                if cache.dark_mode == dark_mode {
                    return;
                }

                // Update cache
                cache.dark_mode = dark_mode;

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnThemeChange {
                        window_id,
                        dark_mode,
                    }));
            }

            WinitWindowEvent::Occluded(occluded) => {
                let cache = self.window.cache.get_or_create(window_id);

                // Only dispatch event if occluded state actually changed
                if cache.occluded == occluded {
                    return;
                }

                // Update cache
                cache.occluded = occluded;

                self.runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnOcclude {
                        window_id,
                        occluded,
                    }));
                emit_window_state_change_if_needed(self, window_id);
            }

            WinitWindowEvent::RedrawRequested => {
                // Only dispatch event and render if window is dirty
                if let Some(window_state) = self.window.states.get_mut(&window_id) {
                    if window_state.is_dirty && window_state.window.is_visible().unwrap_or(true) {
                        window_state.is_dirty = false;

                        self.runtime.push_event(EngineEvent::Window(
                            WindowEvent::OnRedrawRequest { window_id },
                        ));

                        render_frames(self);
                    }
                }
            }

            // Events we don't need to handle
            WinitWindowEvent::ActivationTokenDone { .. } => {}
            WinitWindowEvent::AxisMotion { .. } => {}
            WinitWindowEvent::TouchpadPressure { .. } => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: WinitDeviceEvent,
    ) {
        let WinitDeviceEvent::MouseMotion { delta } = event else {
            return;
        };
        let Some(window_id) = self
            .window
            .states
            .iter()
            .find_map(|(window_id, window_state)| {
                if window_state.window.has_focus()
                    && self.window.cursor_grab_mode(*window_id) == CursorGrabMode::Locked
                {
                    Some(*window_id)
                } else {
                    None
                }
            })
        else {
            return;
        };

        let movement = Vec2::new(delta.0 as f32, delta.1 as f32);
        if movement.length_squared() <= f32::EPSILON {
            return;
        }

        let Some(window_state) = self.window.states.get(&window_id) else {
            return;
        };
        let base_position = self
            .window
            .cursor_positions
            .get(&window_id)
            .copied()
            .unwrap_or(Vec2::new(
                (window_state.inner_size.x as f32 * 0.5).max(0.0),
                (window_state.inner_size.y as f32 * 0.5).max(0.0),
            ));
        let next_position = base_position + movement;

        let pointer_cache = self.input.cache.get_or_create_pointer(window_id);
        if !pointer_cache.position_changed(next_position) {
            return;
        }
        pointer_cache.position = next_position;
        self.window
            .cursor_positions
            .insert(window_id, next_position);

        self.runtime
            .push_event(EngineEvent::Pointer(pointer_move_event(
                window_id,
                0,
                0,
                next_position,
                None,
            )));
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: EngineCustomEvents) {
        let start = std::time::Instant::now();

        match event {
            EngineCustomEvents::CreateWindow(id, args) => {
                let result = engine_cmd_window_create(self, event_loop, &args);

                self.runtime.push_response(CommandResponseEnvelope {
                    id,
                    response: CommandResponse::WindowCreate(result),
                });
            }

            EngineCustomEvents::NotificationInteraction(event) => {
                self.runtime.push_event(EngineEvent::System(event));
            }
        }

        // Track time spent in custom events to exclude from profiling
        self.profiling.input.custom_events_ns += start.elapsed().as_nanos() as u64;
    }
}
