use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, ButtonState, Event, InputEvent, KeyState, KeyboardKeyEvent,
            PointerButtonEvent,
        },
        winit::WinitInput,
    },
    desktop::WindowSurfaceType,
    input::{
        keyboard::{FilterResult, xkb::keysym_get_name},
        pointer::{ButtonEvent, MotionEvent},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
};

use crate::{input::keybindings::{FunctionEnum, KeyAction}, NuonuoState};

pub fn process_input_event(event: InputEvent<WinitInput>, nuonuo_state: &mut NuonuoState) {
    match event {
        InputEvent::Keyboard { event, .. } => {
            let serial = SERIAL_COUNTER.next_serial();
            let time = Event::time_msec(&event);
            let event_state = event.state();
            let conf_priority_map = nuonuo_state.configs.conf_keybinding_manager.conf_priority_map.clone();
            let conf_keybindings = nuonuo_state.configs.conf_keybinding_manager.conf_keybindings.clone();
            
            let keyboard = &mut nuonuo_state.seat.get_keyboard().unwrap();

            // TODO: inhabit shift+word when other modifiers are actived

            keyboard.input::<(), _>(
                nuonuo_state,
                event.key_code(),
                event_state,
                serial,
                time,
                |state, _modifiers, _keysym_handle| {
                    if event_state == KeyState::Pressed {
                        let mut pressed_keys_name: Vec<String> =
                            keyboard.with_pressed_keysyms(|keysym_handles| {
                                keysym_handles
                                    .iter()
                                    .map(|keysym_handle| {
                                        let keysym_value = keysym_handle.modified_sym();
                                        keysym_get_name(keysym_value)
                                    })
                                    .collect()
                            });

                        pressed_keys_name.sort_by_key(|key| {
                            conf_priority_map
                                .get(key)
                                .cloned()
                                .unwrap_or(3)
                        });

                        let keys = pressed_keys_name.join("+");

                        #[cfg(feature = "trace_input")]
                        tracing::info!("Keys: {:?}", keys);

                        if let Some(command) = conf_keybindings.get(&keys) {
                            match command {
                                KeyAction::Command(cmd) => {
                                    tracing::info!("Command: {}", cmd);
                                    std::process::Command::new(cmd).spawn().ok();
                                }
                                KeyAction::Internal(func) => {
                                    let keybinding_manager = &mut state.configs.conf_keybinding_manager;
                                    match func {
                                        FunctionEnum::SwitchWorkspace1 => {
                                            keybinding_manager.switch_workspace1(&mut state.space_manager);
                                        },
                                        FunctionEnum::SwitchWorkspace2 => {
                                            keybinding_manager.switch_workspace2(&mut state.space_manager);
                                        },
                                    }
                                }
                            }
                        }
                    }

                    FilterResult::Forward
                },
            );
        }

        InputEvent::PointerMotion { .. } => {
            // TODO
        }

        InputEvent::PointerMotionAbsolute { event } => {
            let output = nuonuo_state.space_manager.current_space().outputs().next().unwrap();
            let output_geo = nuonuo_state.space_manager.current_space().output_geometry(output).unwrap();
            let position = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();

            let serial = SERIAL_COUNTER.next_serial();
            let pointer = nuonuo_state.seat.get_pointer().unwrap();
            let under =
                {
                    nuonuo_state.space_manager.current_space().element_under(position).and_then(
                        |(window, location)| {
                            window
                                .surface_under(position - location.to_f64(), WindowSurfaceType::ALL)
                                .map(|(s, p)| (s, (p + location).to_f64()))
                        },
                    )
                };

            // TODO: change keyboard focus here when any window under the pointer

            pointer.motion(
                nuonuo_state,
                under,
                &MotionEvent {
                    location: position,
                    serial,
                    time: event.time_msec(),
                },
            );
            pointer.frame(nuonuo_state);
        }

        InputEvent::PointerButton { event, .. } => {
            let pointer = nuonuo_state.seat.get_pointer().unwrap();
            let keyboard = nuonuo_state.seat.get_keyboard().unwrap();

            let serial = SERIAL_COUNTER.next_serial();

            let button = event.button_code();
            let button_state = event.state();

            #[cfg(feature = "trace_input")]
            tracing::info!(
                "The PointerButton event, button: {button}, location: {:?}",
                pointer.current_location()
            );

            if button_state == ButtonState::Pressed && !pointer.is_grabbed() {
                if let Some((window, _loc)) =                     
                    nuonuo_state
                        .space_manager
                        .current_space()
                        .element_under(pointer.current_location())
                        .map(|(w, l)| (w.clone(), l))
                {
                    nuonuo_state.space_manager.raise_element(&window, true);
                    keyboard.set_focus(
                        nuonuo_state,
                        Some(window.toplevel().unwrap().wl_surface().clone()),
                        serial,
                    );
                } else {
                    nuonuo_state.space_manager.current_space().elements().for_each(|window| {
                        window.set_activated(false);
                        window.toplevel().unwrap().send_pending_configure();
                    });
                    keyboard.set_focus(nuonuo_state, Option::<WlSurface>::None, serial);
                }
            }

            // modify pointer state
            pointer.button(
                nuonuo_state,
                &ButtonEvent {
                    button,
                    state: button_state,
                    serial,
                    time: event.time_msec(),
                },
            );
            pointer.frame(nuonuo_state);
        }

        InputEvent::PointerAxis { .. } => {
            // TODO
        }

        InputEvent::DeviceAdded { device } => {
            // TODO
            #[cfg(feature = "trace_input")]
            tracing::info!("DeviceAdded Event, device: {:?} ", device);
        }

        InputEvent::DeviceRemoved { device } => {
            // TODO
            #[cfg(feature = "trace_input")]
            tracing::info!("DeviceRemoved Event, device: {:?} ", device);
        }
        _ => {}
    }
}

// pub fn get_modifiers_name (modifiers: &ModifiersState) -> Vec<String> {
// 	[
//     ("Ctrl", modifiers.ctrl),
//     ("Alt", modifiers.alt),
//     ("Shift", modifiers.shift),
//     ("CapsLock", modifiers.caps_lock),
//     ("Super", modifiers.logo), // Windows/Command 键
//     ("NumLock", modifiers.num_lock),
//     ("AltGr", modifiers.iso_level3_shift),
//     ("ISO_Level5", modifiers.iso_level5_shift),
// 	]
// 	.iter()
// 	.filter_map(|(name, active)| active.then(|| name.to_string()))
// 	.collect()
// }

