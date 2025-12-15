use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowId};

use crate::backend::backend::{BackendError, BackendResult, WindowBackend};
use crate::core::event_handler::EventHandlerApi;
use crate::core::events::{
    AxisMotionEvent, GestureEvent, ImeEvent, ImeKind, Key, KeyEvent, Modifiers, MouseButton,
    MouseButtonEvent, MouseWheelDelta, PanEvent, Position, Size, Theme, Touch, TouchPhase,
    TouchpadPressureEvent,
};
use crate::core::surface_provider::SurfaceProvider;
use log::error;
use std::time::{Duration, Instant};

use winit::event::{
    ElementState, Ime as WinitIme, MouseScrollDelta, TouchPhase as WinitTouchPhase, WindowEvent,
};
use winit::keyboard::{KeyCode, PhysicalKey};

// Full conversion from winit keyboard to our Key enum
fn convert_key(physical_key: PhysicalKey) -> Key {
    /* unchanged mapping */
    match physical_key {
        PhysicalKey::Code(code) => match code {
            // Letters
            KeyCode::KeyA => Key::A,
            KeyCode::KeyB => Key::B,
            KeyCode::KeyC => Key::C,
            KeyCode::KeyD => Key::D,
            KeyCode::KeyE => Key::E,
            KeyCode::KeyF => Key::F,
            KeyCode::KeyG => Key::G,
            KeyCode::KeyH => Key::H,
            KeyCode::KeyI => Key::I,
            KeyCode::KeyJ => Key::J,
            KeyCode::KeyK => Key::K,
            KeyCode::KeyL => Key::L,
            KeyCode::KeyM => Key::M,
            KeyCode::KeyN => Key::N,
            KeyCode::KeyO => Key::O,
            KeyCode::KeyP => Key::P,
            KeyCode::KeyQ => Key::Q,
            KeyCode::KeyR => Key::R,
            KeyCode::KeyS => Key::S,
            KeyCode::KeyT => Key::T,
            KeyCode::KeyU => Key::U,
            KeyCode::KeyV => Key::V,
            KeyCode::KeyW => Key::W,
            KeyCode::KeyX => Key::X,
            KeyCode::KeyY => Key::Y,
            KeyCode::KeyZ => Key::Z,
            // Digits
            KeyCode::Digit0 => Key::Num0,
            KeyCode::Digit1 => Key::Num1,
            KeyCode::Digit2 => Key::Num2,
            KeyCode::Digit3 => Key::Num3,
            KeyCode::Digit4 => Key::Num4,
            KeyCode::Digit5 => Key::Num5,
            KeyCode::Digit6 => Key::Num6,
            KeyCode::Digit7 => Key::Num7,
            KeyCode::Digit8 => Key::Num8,
            KeyCode::Digit9 => Key::Num9,
            // Punctuation and symbols
            KeyCode::Space => Key::Space,
            KeyCode::Backquote => Key::Backquote,
            KeyCode::Minus => Key::Minus,
            KeyCode::Equal => Key::Equal,
            KeyCode::BracketLeft => Key::BracketLeft,
            KeyCode::BracketRight => Key::BracketRight,
            KeyCode::Backslash => Key::Backslash,
            KeyCode::IntlBackslash => Key::IntlBackslash,
            KeyCode::IntlRo => Key::IntlRo,
            KeyCode::IntlYen => Key::IntlYen,
            KeyCode::Semicolon => Key::Semicolon,
            KeyCode::Quote => Key::Quote,
            KeyCode::Comma => Key::Comma,
            KeyCode::Period => Key::Period,
            KeyCode::Slash => Key::Slash,
            // Editing / system
            KeyCode::Enter => Key::Enter,
            KeyCode::Escape => Key::Escape,
            KeyCode::Tab => Key::Tab,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Help => Key::Help,
            KeyCode::ContextMenu => Key::ContextMenu,
            KeyCode::PrintScreen => Key::PrintScreen,
            KeyCode::Pause => Key::Pause,
            // Navigation
            KeyCode::ArrowLeft => Key::Left,
            KeyCode::ArrowRight => Key::Right,
            KeyCode::ArrowUp => Key::Up,
            KeyCode::ArrowDown => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            // Modifiers
            KeyCode::ShiftLeft => Key::LShift,
            KeyCode::ShiftRight => Key::RShift,
            KeyCode::ControlLeft => Key::LCtrl,
            KeyCode::ControlRight => Key::RCtrl,
            KeyCode::AltLeft => Key::LAlt,
            KeyCode::AltRight => Key::RAlt,
            KeyCode::SuperLeft => Key::LSuper,
            KeyCode::SuperRight => Key::RSuper,
            KeyCode::CapsLock => Key::CapsLock,
            KeyCode::NumLock => Key::NumLock,
            KeyCode::ScrollLock => Key::ScrollLock,
            KeyCode::Fn => Key::Fn,
            KeyCode::FnLock => Key::FnLock,
            // Function keys
            KeyCode::F1 => Key::F1,
            KeyCode::F2 => Key::F2,
            KeyCode::F3 => Key::F3,
            KeyCode::F4 => Key::F4,
            KeyCode::F5 => Key::F5,
            KeyCode::F6 => Key::F6,
            KeyCode::F7 => Key::F7,
            KeyCode::F8 => Key::F8,
            KeyCode::F9 => Key::F9,
            KeyCode::F10 => Key::F10,
            KeyCode::F11 => Key::F11,
            KeyCode::F12 => Key::F12,
            KeyCode::F13 => Key::F13,
            KeyCode::F14 => Key::F14,
            KeyCode::F15 => Key::F15,
            KeyCode::F16 => Key::F16,
            KeyCode::F17 => Key::F17,
            KeyCode::F18 => Key::F18,
            KeyCode::F19 => Key::F19,
            KeyCode::F20 => Key::F20,
            KeyCode::F21 => Key::F21,
            KeyCode::F22 => Key::F22,
            KeyCode::F23 => Key::F23,
            KeyCode::F24 => Key::F24,
            KeyCode::F25 => Key::F25,
            KeyCode::F26 => Key::F26,
            KeyCode::F27 => Key::F27,
            KeyCode::F28 => Key::F28,
            KeyCode::F29 => Key::F29,
            KeyCode::F30 => Key::F30,
            KeyCode::F31 => Key::F31,
            KeyCode::F32 => Key::F32,
            KeyCode::F33 => Key::F33,
            KeyCode::F34 => Key::F34,
            KeyCode::F35 => Key::F35,
            // Media
            KeyCode::MediaPlayPause => Key::MediaPlayPause,
            KeyCode::MediaStop => Key::MediaStop,
            KeyCode::MediaTrackNext => Key::MediaNextTrack,
            KeyCode::MediaTrackPrevious => Key::MediaPrevTrack,
            KeyCode::AudioVolumeUp => Key::VolumeUp,
            KeyCode::AudioVolumeDown => Key::VolumeDown,
            KeyCode::AudioVolumeMute => Key::VolumeMute,
            KeyCode::MediaSelect => Key::MediaSelect,
            KeyCode::Eject => Key::Eject,
            KeyCode::Power => Key::Power,
            KeyCode::Sleep => Key::Sleep,
            KeyCode::WakeUp => Key::WakeUp,
            // Browser
            KeyCode::BrowserBack => Key::BrowserBack,
            KeyCode::BrowserForward => Key::BrowserForward,
            KeyCode::BrowserRefresh => Key::BrowserRefresh,
            KeyCode::BrowserStop => Key::BrowserStop,
            KeyCode::BrowserSearch => Key::BrowserSearch,
            KeyCode::BrowserFavorites => Key::BrowserFavorites,
            KeyCode::BrowserHome => Key::BrowserHome,
            KeyCode::LaunchMail => Key::LaunchMail,
            KeyCode::LaunchApp1 => Key::LaunchApp1,
            KeyCode::LaunchApp2 => Key::LaunchApp2,
            // IME / locale-specific
            KeyCode::Convert => Key::Convert,
            KeyCode::NonConvert => Key::NonConvert,
            KeyCode::KanaMode => Key::KanaMode,
            KeyCode::Lang1 => Key::Lang1,
            KeyCode::Lang2 => Key::Lang2,
            KeyCode::Lang3 => Key::Lang3,
            KeyCode::Lang4 => Key::Lang4,
            KeyCode::Lang5 => Key::Lang5,
            // Numpad
            KeyCode::Numpad0 => Key::NumPad0,
            KeyCode::Numpad1 => Key::NumPad1,
            KeyCode::Numpad2 => Key::NumPad2,
            KeyCode::Numpad3 => Key::NumPad3,
            KeyCode::Numpad4 => Key::NumPad4,
            KeyCode::Numpad5 => Key::NumPad5,
            KeyCode::Numpad6 => Key::NumPad6,
            KeyCode::Numpad7 => Key::NumPad7,
            KeyCode::Numpad8 => Key::NumPad8,
            KeyCode::Numpad9 => Key::NumPad9,
            KeyCode::NumpadAdd => Key::NumPadAdd,
            KeyCode::NumpadSubtract => Key::NumPadSubtract,
            KeyCode::NumpadMultiply => Key::NumPadMultiply,
            KeyCode::NumpadDivide => Key::NumPadDivide,
            KeyCode::NumpadDecimal => Key::NumPadDecimal,
            KeyCode::NumpadEnter => Key::NumPadEnter,
            KeyCode::NumpadEqual => Key::NumPadEquals,
            KeyCode::NumpadBackspace => Key::NumPadBackspace,
            KeyCode::NumpadClear => Key::NumPadClear,
            KeyCode::NumpadClearEntry => Key::NumPadClearEntry,
            KeyCode::NumpadComma => Key::NumPadComma,
            KeyCode::NumpadHash => Key::NumPadHash,
            KeyCode::NumpadMemoryAdd => Key::NumPadMemoryAdd,
            KeyCode::NumpadMemoryClear => Key::NumPadMemoryClear,
            KeyCode::NumpadMemoryRecall => Key::NumPadMemoryRecall,
            KeyCode::NumpadMemoryStore => Key::NumPadMemoryStore,
            KeyCode::NumpadMemorySubtract => Key::NumPadMemorySubtract,
            KeyCode::NumpadParenLeft => Key::NumPadParenLeft,
            KeyCode::NumpadParenRight => Key::NumPadParenRight,
            KeyCode::NumpadStar => Key::NumPadStar,
            _ => Key::Unknown,
        },
        _ => Key::Unknown,
    }
}

fn convert_modifiers(mods: winit::keyboard::ModifiersState) -> Modifiers {
    Modifiers {
        shift: mods.shift_key(),
        ctrl: mods.control_key(),
        alt: mods.alt_key(),
        logo: mods.super_key(),
    }
}

fn convert_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(n) => MouseButton::Other(n),
    }
}

fn convert_wheel_delta(delta: MouseScrollDelta) -> MouseWheelDelta {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => MouseWheelDelta::Lines(y),
        MouseScrollDelta::PixelDelta(pos) => MouseWheelDelta::Pixels(pos.y as f32),
    }
}

fn convert_touch_phase(phase: WinitTouchPhase) -> TouchPhase {
    match phase {
        WinitTouchPhase::Started => TouchPhase::Started,
        WinitTouchPhase::Moved => TouchPhase::Moved,
        WinitTouchPhase::Ended => TouchPhase::Ended,
        WinitTouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

fn convert_touch(touch: winit::event::Touch) -> Touch {
    Touch {
        id: touch.id,
        phase: convert_touch_phase(touch.phase),
        position: Position {
            x: touch.location.x as f32,
            y: touch.location.y as f32,
        },
        force: touch.force.map(|f| match f {
            winit::event::Force::Calibrated { force, .. } => force as f32,
            winit::event::Force::Normalized(n) => n as f32,
        }),
    }
}

fn convert_ime(ime: WinitIme) -> ImeEvent {
    let kind = match ime {
        WinitIme::Enabled => ImeKind::Enabled,
        WinitIme::Preedit(text, cursor) => ImeKind::Preedit { text, cursor },
        WinitIme::Commit(text) => ImeKind::Commit(text),
        WinitIme::Disabled => ImeKind::Disabled,
    };
    ImeEvent { kind }
}

fn convert_theme(theme: winit::window::Theme) -> Theme {
    match theme {
        winit::window::Theme::Light => Theme::Light,
        winit::window::Theme::Dark => Theme::Dark,
    }
}

pub struct WinitBackend {
    event_loop: Option<EventLoop<()>>,
    window: Option<Window>,
    pending_config: Option<crate::core::window_config::WindowConfig>,
    last_error: Option<BackendError>,
    continuous: bool,
    current_modifiers: Modifiers,
    mouse_position: Position,
    fixed_frame_duration: Option<Duration>,
    last_frame_instant: Instant,
}

impl WinitBackend {
    pub fn try_new() -> BackendResult<Self> {
        let event_loop =
            EventLoop::new().map_err(|e| BackendError::PlatformError(format!("{:?}", e)))?;
        Ok(Self {
            event_loop: Some(event_loop),
            window: None,
            pending_config: None,
            last_error: None,
            continuous: false,
            current_modifiers: Modifiers::default(),
            mouse_position: Position { x: 0.0, y: 0.0 },
            fixed_frame_duration: None,
            last_frame_instant: Instant::now(),
        })
    }
}

struct WinitApp<'a> {
    backend: &'a mut WinitBackend,
    handler: &'a mut dyn EventHandlerApi,
}

impl<'a> ApplicationHandler for WinitApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application resumes
        let config = self.backend.pending_config.take().unwrap_or_default();

        // Capture redraw policy for use during the loop
        self.backend.continuous = config.continuous.unwrap_or(false);
        if let Some(fps) = config.target_fps {
            if fps > 0 {
                self.backend.fixed_frame_duration = Some(Duration::from_secs_f64(1.0 / fps as f64));
                // Disable continuous when fixed fps requested
                self.backend.continuous = false;
            } else {
                self.backend.fixed_frame_duration = None;
            }
        }

        let mut attrs = Window::default_attributes()
            .with_title(config.title.unwrap_or_else(|| "RustyEngine".to_string()))
            .with_visible(config.visible.unwrap_or(true))
            .with_decorations(config.decorations.unwrap_or(true))
            .with_resizable(config.resizable.unwrap_or(true))
            .with_transparent(config.transparent.unwrap_or(false))
            .with_maximized(config.maximized.unwrap_or(false));

        if let (Some(w), Some(h)) = (config.width, config.height) {
            attrs = attrs.with_inner_size(LogicalSize::new(w as f32, h as f32));
        }

        if config.fullscreen.unwrap_or(false) {
            attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        match event_loop.create_window(attrs) {
            Ok(win) => {
                // Provide surface to engine via handler, then request a redraw
                self.backend.window = Some(win);
                if let Some(w) = self.backend.window.as_ref() {
                    self.handler.on_surface_ready(w as &dyn SurfaceProvider);
                    w.request_redraw();
                }
            }
            Err(e) => {
                self.backend.window = None;
                self.backend.last_error =
                    Some(BackendError::WindowCreationFailed(format!("{:?}", e)));
                error!("WinitBackend: window creation failed: {:?}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            // === WINDOW ===
            WindowEvent::Resized(physical_size) => {
                self.handler.on_resize(&Size {
                    width: physical_size.width,
                    height: physical_size.height,
                });
                if let Some(win) = self.backend.window.as_ref() {
                    win.request_redraw();
                }
            }

            WindowEvent::Moved(position) => {
                self.handler.on_move(&(position.x, position.y));
            }

            WindowEvent::CloseRequested => {
                self.handler.on_close();
                event_loop.exit();
            }

            WindowEvent::Destroyed => {
                self.handler.on_destroy();
            }

            WindowEvent::Focused(focused) => {
                self.handler.on_focus(&focused);
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handler.on_scale_factor_changed(&scale_factor);
                if let Some(win) = self.backend.window.as_ref() {
                    win.request_redraw();
                }
            }

            WindowEvent::ThemeChanged(theme) => {
                self.handler.on_theme_changed(&convert_theme(theme));
            }

            WindowEvent::Occluded(occluded) => {
                self.handler.on_occluded(&occluded);
            }

            // === KEYBOARD ===
            WindowEvent::KeyboardInput { event, .. } => {
                let key = convert_key(event.physical_key);
                let mods = self.backend.current_modifiers;

                match event.state {
                    ElementState::Pressed => {
                        self.handler.on_key_pressed(&KeyEvent {
                            key,
                            modifiers: mods,
                        });
                    }
                    ElementState::Released => {
                        self.handler.on_key_released(&KeyEvent {
                            key,
                            modifiers: mods,
                        });
                    }
                }
            }

            WindowEvent::ModifiersChanged(new_mods) => {
                self.backend.current_modifiers = convert_modifiers(new_mods.state());
                self.handler
                    .on_modifiers_changed(&self.backend.current_modifiers);
            }

            WindowEvent::Ime(ime) => {
                self.handler.on_ime(&convert_ime(ime));
            }

            // === MOUSE ===
            WindowEvent::CursorMoved { position, .. } => {
                let pos = Position {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                self.backend.mouse_position = pos;
                self.handler.on_mouse_move(&pos);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let btn = convert_mouse_button(button);
                let pos = self.backend.mouse_position;
                let ev = MouseButtonEvent {
                    button: btn,
                    position: pos,
                };
                match state {
                    ElementState::Pressed => self.handler.on_mouse_button_pressed(&ev),
                    ElementState::Released => self.handler.on_mouse_button_released(&ev),
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                self.handler.on_mouse_wheel(&convert_wheel_delta(delta));
            }

            WindowEvent::CursorEntered { .. } => {
                self.handler.on_mouse_enter();
            }

            WindowEvent::CursorLeft { .. } => {
                self.handler.on_mouse_leave();
            }

            // === TOUCH ===
            WindowEvent::Touch(touch) => {
                self.handler.on_touch(&convert_touch(touch));
            }

            // === GESTURES ===
            WindowEvent::PinchGesture { delta, phase, .. } => {
                self.handler.on_pinch(&GestureEvent {
                    phase: convert_touch_phase(phase),
                    delta,
                });
            }

            WindowEvent::PanGesture { delta, phase, .. } => {
                self.handler.on_pan(&PanEvent {
                    phase: convert_touch_phase(phase),
                    delta: Position {
                        x: delta.x,
                        y: delta.y,
                    },
                });
            }

            WindowEvent::DoubleTapGesture { .. } => {
                self.handler.on_double_tap();
            }

            WindowEvent::RotationGesture { delta, phase, .. } => {
                self.handler.on_rotate(&GestureEvent {
                    phase: convert_touch_phase(phase),
                    delta: delta as f64,
                });
            }

            WindowEvent::TouchpadPressure {
                pressure, stage, ..
            } => {
                self.handler
                    .on_touchpad_pressure(&TouchpadPressureEvent { pressure, stage });
            }

            // === FILE DROP ===
            WindowEvent::DroppedFile(path) => {
                self.handler.on_file_dropped(&path);
            }

            WindowEvent::HoveredFile(path) => {
                self.handler.on_file_hovered(&path);
            }

            WindowEvent::HoveredFileCancelled => {
                self.handler.on_file_hover_cancelled();
            }

            // === GAMEPAD/JOYSTICK ===
            WindowEvent::AxisMotion { axis, value, .. } => {
                self.handler
                    .on_axis_motion(&AxisMotionEvent { axis, value });
            }

            // === SPECIAL ===
            WindowEvent::ActivationTokenDone { token, .. } => {
                self.handler.on_activation_token(&format!("{:?}", token));
            }

            // === REDRAW ===
            WindowEvent::RedrawRequested => {
                // Frame tick: let engine update its state
                self.handler.on_tick();
                self.backend.last_frame_instant = Instant::now();
                self.handler.on_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Fixed FPS management
        if let Some(frame_dur) = self.backend.fixed_frame_duration {
            let target = self.backend.last_frame_instant + frame_dur;
            let now = Instant::now();
            if now >= target {
                if let Some(win) = self.backend.window.as_ref() {
                    win.request_redraw();
                }
            } else {
                _event_loop.set_control_flow(ControlFlow::WaitUntil(target));
            }
        } else if let Some(win) = self
            .backend
            .window
            .as_ref()
            .filter(|_| self.backend.continuous)
        {
            win.request_redraw();
        }
    }
}

impl WindowBackend for WinitBackend {
    fn create_window(
        &mut self,
        config: crate::core::window_config::WindowConfig,
    ) -> BackendResult<()> {
        // Store the config so resumed() can translate it to winit attributes
        self.pending_config = Some(config);
        Ok(())
    }

    fn run(&mut self, handler: &mut dyn EventHandlerApi) -> BackendResult<()> {
        let event_loop = self
            .event_loop
            .take()
            .ok_or(BackendError::EventLoopConsumed)?;
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut app = WinitApp {
            backend: self,
            handler,
        };

        match event_loop.run_app(&mut app) {
            Ok(()) => {
                if let Some(err) = app.backend.last_error.take() {
                    Err(err)
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(BackendError::PlatformError(format!("{:?}", e))),
        }
    }

    fn surface_provider(&self) -> Option<&dyn SurfaceProvider> {
        // Expose the window as a SurfaceProvider when available
        self.window.as_ref().map(|w| w as &dyn SurfaceProvider)
    }
}

impl SurfaceProvider for Window {
    fn size(&self) -> (u32, u32) {
        let size = self.inner_size();
        (size.width, size.height)
    }
}
