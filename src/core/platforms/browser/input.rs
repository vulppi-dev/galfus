use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    CompositionEvent, Event, EventTarget, HtmlCanvasElement, KeyboardEvent, PointerEvent,
    WheelEvent,
};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{ElementState, ModifiersState, TouchPhase};
use crate::core::input::events::{
    KeyboardEvent as CoreKeyboardEvent, PointerEvent as CorePointerEvent, ScrollDelta,
};
use crate::core::input::keycodes::map_web_key_code;
use crate::core::singleton::with_engine;
use crate::core::state::EngineState;
use crate::core::window::{WebListenerRegistration, WindowEvent};

pub fn attach_canvas_listeners(
    window_id: u32,
    canvas: &HtmlCanvasElement,
) -> Vec<WebListenerRegistration> {
    let mut listeners: Vec<WebListenerRegistration> = Vec::new();

    let window = match web_sys::window() {
        Some(window) => window,
        None => return listeners,
    };
    let window_target: EventTarget = window.clone().unchecked_into();
    let canvas_target: EventTarget = canvas.clone().unchecked_into();
    let modifiers_state = Rc::new(RefCell::new(ModifiersState::default()));

    let canvas_for_resize = canvas.clone();
    let resize_closure = Closure::wrap(Box::new(move |_event: Event| {
        let width = canvas_for_resize.client_width().max(1) as u32;
        let height = canvas_for_resize.client_height().max(1) as u32;

        with_live_window(window_id, |engine| {
            if let Some(window_state) = engine.window.states.get_mut(&window_id) {
                window_state.config.width = width;
                window_state.config.height = height;
                if let Some(device) = engine.device.as_ref() {
                    window_state.surface.configure(device, &window_state.config);
                    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
                    if let Some(render_state) = engine.render.get_mut(&window_id) {
                        render_state.on_resize(device, width, height);
                    }
                    crate::core::resources::ensure_render_target(
                        device,
                        &mut window_state.surface_target,
                        width.max(1),
                        height.max(1),
                        wgpu::TextureFormat::Rgba16Float,
                    );
                }
                window_state.inner_size = glam::UVec2::new(width, height);
                window_state.outer_size = glam::UVec2::new(width, height);
                window_state.is_dirty = true;
            }
            if let Some(surface_id) = engine
                .universal_state
                .presents
                .entries
                .values()
                .find(|present| present.value.window_id == window_id)
                .map(|present| present.value.surface)
            {
                if let Some(surface_entry) =
                    engine.universal_state.surfaces.entries.get_mut(&surface_id)
                {
                    surface_entry.value.size = glam::UVec2::new(width, height);
                }
            }
            engine
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnResize {
                    window_id,
                    width,
                    height,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "resize", resize_closure, &mut listeners);

    let focus_closure = Closure::wrap(Box::new(move |_event: Event| {
        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnFocus {
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
                engine.event_queue.push(EngineEvent::Keyboard(
                    CoreKeyboardEvent::OnModifiersChange {
                        window_id,
                        modifiers: next_modifiers,
                    },
                ));
            }
            engine
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnFocus {
                    window_id,
                    focused: false,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "blur", blur_closure, &mut listeners);

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
        let text = if is_composing {
            None
        } else {
            event
                .key()
                .chars()
                .next()
                .filter(|_| event.key().len() == 1)
                .map(|_| event.key())
        };

        with_live_window(window_id, |engine| {
            let mut current_modifiers = modifiers_state_for_keydown.borrow_mut();
            if *current_modifiers != modifiers {
                *current_modifiers = modifiers;
                engine.event_queue.push(EngineEvent::Keyboard(
                    CoreKeyboardEvent::OnModifiersChange {
                        window_id,
                        modifiers,
                    },
                ));
            }
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnInput {
                    window_id,
                    key_code,
                    state: ElementState::Pressed,
                    location: event.location() as u32,
                    repeat: event.repeat(),
                    text,
                    modifiers,
                }));
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
        let text = if is_composing {
            None
        } else {
            event
                .key()
                .chars()
                .next()
                .filter(|_| event.key().len() == 1)
                .map(|_| event.key())
        };

        with_live_window(window_id, |engine| {
            let mut current_modifiers = modifiers_state_for_keyup.borrow_mut();
            if *current_modifiers != modifiers {
                *current_modifiers = modifiers;
                engine.event_queue.push(EngineEvent::Keyboard(
                    CoreKeyboardEvent::OnModifiersChange {
                        window_id,
                        modifiers,
                    },
                ));
            }
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnInput {
                    window_id,
                    key_code,
                    state: ElementState::Released,
                    location: event.location() as u32,
                    repeat: event.repeat(),
                    text,
                    modifiers,
                }));
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
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnImeEnable {
                    window_id,
                }));
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
        let text = event.data();
        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnImePreedit {
                    window_id,
                    text,
                    cursor_range: None,
                }));
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
        let text = event.data();
        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnImeCommit {
                    window_id,
                    text,
                }));
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnImeDisable {
                    window_id,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&window_target, "compositionend", ime_end, &mut listeners);

    let canvas_for_pointer = canvas.clone();
    let pointer_move = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let position = canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;

        with_live_window(window_id, |engine| {
            engine.window.cursor_positions.insert(window_id, position);
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnMove {
                    window_id,
                    pointer_type,
                    pointer_id,
                    position,
                    position_target: None,
                    trace: None,
                }));
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
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let button = event.button() as u32;

        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnButton {
                    window_id,
                    pointer_type,
                    pointer_id,
                    button,
                    state: ElementState::Pressed,
                    position,
                    position_target: None,
                    trace: None,
                }));
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
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let button = event.button() as u32;

        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnButton {
                    window_id,
                    pointer_type,
                    pointer_id,
                    button,
                    state: ElementState::Released,
                    position,
                    position_target: None,
                    trace: None,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    register_listener(&canvas_target, "pointerup", pointer_up, &mut listeners);

    let pointer_enter = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnEnter {
                    window_id,
                    pointer_type,
                    pointer_id,
                    trace: None,
                }));
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
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        with_live_window(window_id, |engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnLeave {
                    window_id,
                    pointer_type,
                    pointer_id,
                    trace: None,
                }));
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
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnScroll {
                    window_id,
                    delta,
                    phase,
                    trace: None,
                }));
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

fn with_live_window(window_id: u32, apply: impl FnOnce(&mut EngineState)) {
    let _ = with_engine(|engine| {
        if engine.window.states.contains_key(&window_id) {
            apply(engine);
        }
    });
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

fn map_pointer_type(pointer_type: &str) -> u32 {
    match pointer_type {
        "mouse" => 0,
        "touch" => 1,
        "pen" => 2,
        _ => 0,
    }
}
