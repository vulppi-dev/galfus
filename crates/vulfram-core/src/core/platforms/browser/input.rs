use std::cell::RefCell;
use std::rc::Rc;

use vulfram_input::{
    keyboard_ime_commit_event, keyboard_ime_disable_event, keyboard_ime_enable_event,
    keyboard_ime_preedit_event, keyboard_input_event, keyboard_modifiers_event, map_web_key_code,
    pointer_button_event, pointer_enter_event, pointer_leave_event, pointer_move_event,
    pointer_scroll_event,
};
use vulfram_platform::{
    BrowserPointerMotionInput, PlatformCursorGrabMode, PlatformWindowState,
    map_browser_pointer_type, normalize_browser_key_text, plan_browser_surface_resize,
    resolve_browser_pointer_position, resolve_browser_window_state, resolve_canvas_surface_size,
    resolve_pointer_lock_change, resolve_pointer_lock_error,
};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    CompositionEvent, Event, EventTarget, HtmlCanvasElement, KeyboardEvent, PointerEvent,
    WheelEvent,
};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::ScrollDelta;
use crate::core::input::events::{ElementState, ModifiersState, TouchPhase};
use crate::core::singleton::with_engine;
use crate::core::state::EngineState;
use crate::core::window::{
    CursorGrabMode, EngineWindowState, WebListenerRegistration, WindowEvent,
    WindowPointerCaptureState,
};

pub fn attach_canvas_listeners(
    window_id: u32,
    canvas: &HtmlCanvasElement,
) -> Vec<WebListenerRegistration> {
    let mut listeners: Vec<WebListenerRegistration> = Vec::new();

    let window = match web_sys::window() {
        Some(window) => window,
        None => return listeners,
    };
    let document = match window.document() {
        Some(document) => document,
        None => return listeners,
    };
    let window_target: EventTarget = window.clone().unchecked_into();
    let document_target: EventTarget = document.clone().unchecked_into();
    let canvas_target: EventTarget = canvas.clone().unchecked_into();
    let modifiers_state = Rc::new(RefCell::new(ModifiersState::default()));

    let canvas_for_resize = canvas.clone();
    let resize_closure = Closure::wrap(Box::new(move |_event: Event| {
        let (width, height) = canvas_surface_size_from_rect(&canvas_for_resize);
        let _ = sync_canvas_surface_size(window_id, width, height);
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "resize", resize_closure, &mut listeners);

    // Polling via RAF captures real canvas surface changes even without `window.resize`.
    let raf_slot: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let raf_slot_loop = raf_slot.clone();
    let canvas_for_raf = canvas.clone();
    let window_for_raf = window.clone();
    *raf_slot.borrow_mut() = Some(Closure::wrap(Box::new(move |_ts: f64| {
        if !with_live_window(window_id, |_| {}) {
            let _ = raf_slot_loop.borrow_mut().take();
            return;
        }
        let (width, height) = canvas_surface_size_from_rect(&canvas_for_raf);
        let _ = sync_canvas_surface_size(window_id, width, height);
        if let Some(callback) = raf_slot_loop.borrow().as_ref() {
            let _ = window_for_raf.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>));
    if let Some(callback) = raf_slot.borrow().as_ref() {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    }

    let focus_closure = Closure::wrap(Box::new(move |_event: Event| {
        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Window(WindowEvent::OnFocus {
                    window_id,
                    focused: true,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "focus", focus_closure, &mut listeners);

    let modifiers_state_for_blur = modifiers_state.clone();
    let blur_closure = Closure::wrap(Box::new(move |_event: Event| {
        with_live_window(window_id, |engine| {
            let next_modifiers = ModifiersState::default();
            let mut current_modifiers = modifiers_state_for_blur.borrow_mut();
            if *current_modifiers != next_modifiers {
                *current_modifiers = next_modifiers;
                engine
                    .runtime
                    .push_event(EngineEvent::Keyboard(keyboard_modifiers_event(
                        window_id,
                        next_modifiers,
                    )));
            }
            engine
                .runtime
                .push_event(EngineEvent::Window(WindowEvent::OnFocus {
                    window_id,
                    focused: false,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "blur", blur_closure, &mut listeners);

    let fullscreen_change = Closure::wrap(Box::new(move |_event: Event| {
        with_live_window(window_id, |engine| {
            let state = web_sys::window()
                .and_then(|window| window.document())
                .map(|document| {
                    match resolve_browser_window_state(document.fullscreen_element().is_some()) {
                        PlatformWindowState::Windowed => EngineWindowState::Windowed,
                        PlatformWindowState::Fullscreen => EngineWindowState::Fullscreen,
                    }
                })
                .unwrap_or(EngineWindowState::Windowed);
            if engine.window.set_lifecycle_state(window_id, state) {
                engine
                    .runtime
                    .push_event(EngineEvent::Window(WindowEvent::OnStateChange {
                        window_id,
                        state,
                    }));
            }
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &document_target,
        "fullscreenchange",
        fullscreen_change,
        &mut listeners,
    );

    let canvas_lock_id = canvas.id();
    let pointer_lock_change = Closure::wrap(Box::new(move |_event: Event| {
        with_live_window(window_id, |engine| {
            let mode = map_platform_cursor_grab_mode(engine.window.cursor_grab_mode(window_id));
            let active = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.pointer_lock_element())
                .map(|element| {
                    if canvas_lock_id.is_empty() {
                        true
                    } else {
                        element.id() == canvas_lock_id
                    }
                })
                .unwrap_or(false);
            let Some(capture_update) = resolve_pointer_lock_change(mode, active) else {
                return;
            };
            if engine
                .window
                .set_pointer_capture_active(window_id, capture_update.active)
            {
                engine.runtime.push_event(EngineEvent::Window(
                    WindowEvent::OnPointerCaptureChange {
                        window_id,
                        capture: WindowPointerCaptureState {
                            mode: CursorGrabMode::Locked,
                            active: capture_update.active,
                            reason: Some(capture_update.reason.into()),
                        },
                    },
                ));
            }
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &document_target,
        "pointerlockchange",
        pointer_lock_change,
        &mut listeners,
    );

    let pointer_lock_error = Closure::wrap(Box::new(move |_event: Event| {
        with_live_window(window_id, |engine| {
            let mode = map_platform_cursor_grab_mode(engine.window.cursor_grab_mode(window_id));
            let Some(capture_update) = resolve_pointer_lock_error(mode) else {
                return;
            };
            engine
                .window
                .set_pointer_capture_active(window_id, capture_update.active);
            engine
                .runtime
                .push_event(EngineEvent::Window(WindowEvent::OnPointerCaptureChange {
                    window_id,
                    capture: WindowPointerCaptureState {
                        mode: CursorGrabMode::Locked,
                        active: capture_update.active,
                        reason: Some(capture_update.reason.into()),
                    },
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &document_target,
        "pointerlockerror",
        pointer_lock_error,
        &mut listeners,
    );

    let modifiers_state_for_keydown = modifiers_state.clone();
    let keydown_closure = Closure::wrap(Box::new(move |event: Event| {
        let event: KeyboardEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let modifiers = ModifiersState {
            shift: event.shift_key(),
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        };
        let key_code = map_web_key_code(&event.code());
        let is_composing = event.is_composing();
        let text = normalize_browser_key_text(&event.key(), is_composing);

        with_live_window(window_id, |engine| {
            let mut current_modifiers = modifiers_state_for_keydown.borrow_mut();
            if *current_modifiers != modifiers {
                *current_modifiers = modifiers;
                engine
                    .runtime
                    .push_event(EngineEvent::Keyboard(keyboard_modifiers_event(
                        window_id, modifiers,
                    )));
            }
            engine
                .runtime
                .push_event(EngineEvent::Keyboard(keyboard_input_event(
                    window_id,
                    key_code,
                    ElementState::Pressed,
                    event.location() as u32,
                    event.repeat(),
                    text,
                    modifiers,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "keydown", keydown_closure, &mut listeners);

    let modifiers_state_for_keyup = modifiers_state.clone();
    let keyup_closure = Closure::wrap(Box::new(move |event: Event| {
        let event: KeyboardEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let modifiers = ModifiersState {
            shift: event.shift_key(),
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        };
        let key_code = map_web_key_code(&event.code());
        let is_composing = event.is_composing();
        let text = normalize_browser_key_text(&event.key(), is_composing);

        with_live_window(window_id, |engine| {
            let mut current_modifiers = modifiers_state_for_keyup.borrow_mut();
            if *current_modifiers != modifiers {
                *current_modifiers = modifiers;
                engine
                    .runtime
                    .push_event(EngineEvent::Keyboard(keyboard_modifiers_event(
                        window_id, modifiers,
                    )));
            }
            engine
                .runtime
                .push_event(EngineEvent::Keyboard(keyboard_input_event(
                    window_id,
                    key_code,
                    ElementState::Released,
                    event.location() as u32,
                    event.repeat(),
                    text,
                    modifiers,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "keyup", keyup_closure, &mut listeners);

    let ime_start = Closure::wrap(Box::new(move |event: Event| {
        let _event: CompositionEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Keyboard(keyboard_ime_enable_event(window_id)));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &window_target,
        "compositionstart",
        ime_start,
        &mut listeners,
    );

    let ime_update = Closure::wrap(Box::new(move |event: Event| {
        let event: CompositionEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let text = event.data().unwrap_or_default();
        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Keyboard(keyboard_ime_preedit_event(
                    window_id, text, None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &window_target,
        "compositionupdate",
        ime_update,
        &mut listeners,
    );

    let ime_end = Closure::wrap(Box::new(move |event: Event| {
        let event: CompositionEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let text = event.data().unwrap_or_default();
        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Keyboard(keyboard_ime_commit_event(
                    window_id, text,
                )));
            engine
                .runtime
                .push_event(EngineEvent::Keyboard(keyboard_ime_disable_event(window_id)));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "compositionend", ime_end, &mut listeners);

    let canvas_for_pointer = canvas.clone();
    let pointer_move = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let pointer_type = map_browser_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let absolute_position =
            canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let movement = glam::Vec2::new(event.movement_x() as f32, event.movement_y() as f32);

        with_live_window(window_id, |engine| {
            let mode = map_platform_cursor_grab_mode(engine.window.cursor_grab_mode(window_id));
            let window_size = engine.window.states.get(&window_id).map(|window_state| {
                glam::Vec2::new(
                    window_state.inner_size.x as f32,
                    window_state.inner_size.y as f32,
                )
            });
            let position = resolve_browser_pointer_position(BrowserPointerMotionInput {
                cursor_grab_mode: mode,
                pointer_capture_active: engine.window.pointer_capture_active(window_id),
                absolute_position,
                movement,
                last_position: engine.window.cursor_positions.get(&window_id).copied(),
                window_size,
            });
            engine.window.cursor_positions.insert(window_id, position);
            engine
                .runtime
                .push_event(EngineEvent::Pointer(pointer_move_event(
                    window_id,
                    pointer_type,
                    pointer_id,
                    position,
                    None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&canvas_target, "pointermove", pointer_move, &mut listeners);

    let canvas_for_pointer = canvas.clone();
    let pointer_down = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let position = canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let pointer_type = map_browser_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let button = event.button() as u32;

        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Pointer(pointer_button_event(
                    window_id,
                    pointer_type,
                    pointer_id,
                    button,
                    ElementState::Pressed,
                    position,
                    None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&canvas_target, "pointerdown", pointer_down, &mut listeners);

    let canvas_for_pointer = canvas.clone();
    let pointer_up = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let position = canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let pointer_type = map_browser_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let button = event.button() as u32;

        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Pointer(pointer_button_event(
                    window_id,
                    pointer_type,
                    pointer_id,
                    button,
                    ElementState::Released,
                    position,
                    None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&canvas_target, "pointerup", pointer_up, &mut listeners);

    let pointer_enter = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let pointer_type = map_browser_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Pointer(pointer_enter_event(
                    window_id,
                    pointer_type,
                    pointer_id,
                    None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &canvas_target,
        "pointerenter",
        pointer_enter,
        &mut listeners,
    );

    let pointer_leave = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let pointer_type = map_browser_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        with_live_window(window_id, |engine| {
            engine
                .runtime
                .push_event(EngineEvent::Pointer(pointer_leave_event(
                    window_id,
                    pointer_type,
                    pointer_id,
                    None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(
        &canvas_target,
        "pointerleave",
        pointer_leave,
        &mut listeners,
    );

    let wheel_canvas = canvas.clone();
    let wheel_closure = Closure::wrap(Box::new(move |event: Event| {
        let event: WheelEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let delta = glam::Vec2::new(event.delta_x() as f32, event.delta_y() as f32);
        let phase = TouchPhase::Moved;
        let delta = if event.delta_mode() == WheelEvent::DOM_DELTA_PIXEL {
            ScrollDelta::Pixel(delta)
        } else {
            ScrollDelta::Line(delta)
        };

        with_live_window(window_id, |engine| {
            let _ = &wheel_canvas;
            engine
                .runtime
                .push_event(EngineEvent::Pointer(pointer_scroll_event(
                    window_id, delta, phase, None,
                )));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&canvas_target, "wheel", wheel_closure, &mut listeners);

    listeners
}

fn canvas_relative_pos(canvas: &HtmlCanvasElement, x: i32, y: i32) -> glam::Vec2 {
    let rect = canvas.get_bounding_client_rect();
    glam::Vec2::new(
        (x as f64 - rect.left()) as f32,
        (y as f64 - rect.top()) as f32,
    )
}

fn canvas_surface_size_from_rect(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let rect = canvas.get_bounding_client_rect();
    let dpr = web_sys::window()
        .map(|window| window.device_pixel_ratio())
        .unwrap_or(1.0);
    resolve_canvas_surface_size(rect.width(), rect.height(), dpr)
}

fn with_live_window(window_id: u32, apply: impl FnOnce(&mut EngineState)) -> bool {
    let mut is_live = false;
    let _ = with_engine(|engine| {
        if engine.window.states.contains_key(&window_id) {
            is_live = true;
            apply(engine);
        }
    });
    is_live
}

fn sync_canvas_surface_size(window_id: u32, width: u32, height: u32) -> bool {
    let mut changed = false;
    let _ = with_live_window(window_id, |engine| {
        let mut window_changed = false;
        if let Some(window_state) = engine.window.states.get_mut(&window_id) {
            let current_width = window_state.config.width.max(1);
            let current_height = window_state.config.height.max(1);
            let resize_plan =
                plan_browser_surface_resize(current_width, current_height, width, height);
            window_changed = resize_plan.is_some();
            if let Some(resize_plan) = resize_plan {
                window_state.config.width = resize_plan.width;
                window_state.config.height = resize_plan.height;
                if let Some(device) = engine.device.as_ref() {
                    window_state.surface.configure(device, &window_state.config);
                    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
                    if let Some(render_state) = engine.render.get_mut(&window_id) {
                        render_state.on_resize(device, resize_plan.width, resize_plan.height);
                    }
                    crate::core::resources::ensure_render_target(
                        device,
                        &mut window_state.surface_target,
                        resize_plan.width,
                        resize_plan.height,
                        wgpu::TextureFormat::Rgba16Float,
                    );
                }
                window_state.inner_size = glam::UVec2::new(resize_plan.width, resize_plan.height);
                window_state.outer_size = glam::UVec2::new(resize_plan.width, resize_plan.height);
                window_state.is_dirty = true;
            }
        }

        if !window_changed {
            return;
        }
        changed = true;

        if let Some(surface_id) = engine
            .universal_state
            .presents
            .entries
            .values()
            .find(|present| present.value.window_id == window_id)
            .map(|present| present.value.surface)
            && let Some(surface_entry) =
                engine.universal_state.surfaces.entries.get_mut(&surface_id)
        {
            surface_entry.value.size = glam::UVec2::new(width, height);
        }
        engine
            .runtime
            .push_event(EngineEvent::Window(WindowEvent::OnResize {
                window_id,
                width,
                height,
            }));
    });
    changed
}

fn register_listener(
    target: &EventTarget,
    event_type: &'static str,
    callback: Closure<dyn FnMut(Event)>,
    listeners: &mut Vec<WebListenerRegistration>,
) {
    let _ = target.add_event_listener_with_callback(event_type, callback.as_ref().unchecked_ref());
    listeners.push(WebListenerRegistration {
        target: target.clone(),
        event_type,
        callback,
    });
}

fn map_platform_cursor_grab_mode(mode: CursorGrabMode) -> PlatformCursorGrabMode {
    match mode {
        CursorGrabMode::None => PlatformCursorGrabMode::None,
        CursorGrabMode::Confined => PlatformCursorGrabMode::Confined,
        CursorGrabMode::Locked => PlatformCursorGrabMode::Locked,
    }
}
