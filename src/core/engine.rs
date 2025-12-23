use crate::audio::{AudioError, AudioSystem, RodioBackend};
use crate::backend::window_backend::{BackendError, BackendResult, WindowBackend};
use crate::core::assets::AssetManager;
use crate::core::engine_state::EngineState;
use crate::core::events::EventHandler;
use crate::core::events::EventHandlerApi;
use crate::core::events::{
    AxisMotionEvent, GestureEvent, ImeEvent, KeyEvent, Modifiers, MouseButtonEvent,
    MouseWheelDelta, PanEvent, Position, Size, Theme, Touch, TouchpadPressureEvent,
};
use crate::render::context::RenderContext;
use crate::backend::surface_provider::SurfaceProvider;
use crate::backend::window::WindowConfig;
use crate::render::Renderer;
use std::path::Path;

pub struct Engine {
    pub events: EventHandler,
    pub state: EngineState,
    pub audio: AudioSystem,
    pub assets: AssetManager,
    backend: Box<dyn WindowBackend>,
    renderer: Box<dyn Renderer>,

    window_size: (u32, u32),
    window_config: Option<WindowConfig>,
}

impl Engine {
    pub fn new(
        backend: Box<dyn WindowBackend>,
        renderer: Box<dyn Renderer>,
    ) -> Result<Self, AudioError> {
        let audio_backend = RodioBackend::new()?;
        let audio = AudioSystem::new(Box::new(audio_backend));
        Ok(Self {
            events: EventHandler::new(),
            state: EngineState::new(),
            audio,
            assets: AssetManager::new(),
            backend,
            renderer,
            window_size: (1, 1),
            window_config: None,
        })
    }

    /// Create a window via the backend. Returns an error if the backend fails.
    pub fn create_window(&mut self, config: WindowConfig) -> BackendResult<()> {
        // Validate window configuration before passing to backend
        config.validate().map_err(BackendError::InvalidConfig)?;
        self.window_config = Some(config.clone());
        self.backend.create_window(config)
    }

    /// Run the backend event loop. Returns an error if the backend fails.
    pub fn run(&mut self) -> BackendResult<()> {
        // Forward backend events and hook renderer calls in the engine layer
        struct Forwarder<'a> {
            events: &'a mut EventHandler,
            renderer: &'a mut dyn Renderer,
            initialized: bool,
            state: &'a mut EngineState,
            window_size: &'a mut (u32, u32),
            window_config: Option<&'a WindowConfig>,
            assets: &'a AssetManager,
        }

        impl<'a> EventHandlerApi for Forwarder<'a> {
            fn on_surface_ready(&mut self, surface: &dyn SurfaceProvider) {
                if !self.initialized {
                    let _ = self.renderer.init(surface, self.window_config);
                    // Upload any images that were loaded before the surface was ready.
                    for (id, image) in self.assets.iter_images() {
                        let _ =
                            self.renderer
                                .upload_image(id, image.width, image.height, &image.data);
                    }
                    self.initialized = true;
                }
            }

            fn on_resize(&mut self, size: &Size) {
                if self.initialized {
                    self.renderer.resize((size.width, size.height));
                }
                *self.window_size = (size.width, size.height);
                EventHandlerApi::on_resize(self.events, size);
            }

            fn on_move(&mut self, pos: &(i32, i32)) {
                EventHandlerApi::on_move(self.events, pos);
            }

            fn on_close(&mut self) {
                EventHandlerApi::on_close(self.events);
            }

            fn on_destroy(&mut self) {
                EventHandlerApi::on_destroy(self.events);
            }

            fn on_focus(&mut self, focused: &bool) {
                EventHandlerApi::on_focus(self.events, focused);
            }

            fn on_scale_factor_changed(&mut self, scale: &f64) {
                EventHandlerApi::on_scale_factor_changed(self.events, scale);
            }

            fn on_theme_changed(&mut self, theme: &Theme) {
                EventHandlerApi::on_theme_changed(self.events, theme);
            }

            fn on_occluded(&mut self, occluded: &bool) {
                EventHandlerApi::on_occluded(self.events, occluded);
            }

            fn on_key_pressed(&mut self, ev: &KeyEvent) {
                EventHandlerApi::on_key_pressed(self.events, ev);
            }

            fn on_key_released(&mut self, ev: &KeyEvent) {
                EventHandlerApi::on_key_released(self.events, ev);
            }

            fn on_modifiers_changed(&mut self, mods: &Modifiers) {
                EventHandlerApi::on_modifiers_changed(self.events, mods);
            }

            fn on_ime(&mut self, ime: &ImeEvent) {
                EventHandlerApi::on_ime(self.events, ime);
            }

            fn on_mouse_button_pressed(&mut self, ev: &MouseButtonEvent) {
                EventHandlerApi::on_mouse_button_pressed(self.events, ev);
            }

            fn on_mouse_button_released(&mut self, ev: &MouseButtonEvent) {
                EventHandlerApi::on_mouse_button_released(self.events, ev);
            }

            fn on_mouse_move(&mut self, pos: &Position) {
                EventHandlerApi::on_mouse_move(self.events, pos);
            }

            fn on_mouse_wheel(&mut self, delta: &MouseWheelDelta) {
                EventHandlerApi::on_mouse_wheel(self.events, delta);
            }

            fn on_mouse_enter(&mut self) {
                EventHandlerApi::on_mouse_enter(self.events);
            }

            fn on_mouse_leave(&mut self) {
                EventHandlerApi::on_mouse_leave(self.events);
            }

            fn on_touch(&mut self, touch: &Touch) {
                EventHandlerApi::on_touch(self.events, touch);
            }

            fn on_pinch(&mut self, gesture: &GestureEvent) {
                EventHandlerApi::on_pinch(self.events, gesture);
            }

            fn on_pan(&mut self, pan: &PanEvent) {
                EventHandlerApi::on_pan(self.events, pan);
            }

            fn on_rotate(&mut self, gesture: &GestureEvent) {
                EventHandlerApi::on_rotate(self.events, gesture);
            }

            fn on_double_tap(&mut self) {
                EventHandlerApi::on_double_tap(self.events);
            }

            fn on_touchpad_pressure(&mut self, ev: &TouchpadPressureEvent) {
                EventHandlerApi::on_touchpad_pressure(self.events, ev);
            }

            fn on_file_dropped(&mut self, path: &Path) {
                EventHandlerApi::on_file_dropped(self.events, path);
            }

            fn on_file_hovered(&mut self, path: &Path) {
                EventHandlerApi::on_file_hovered(self.events, path);
            }

            fn on_file_hover_cancelled(&mut self) {
                EventHandlerApi::on_file_hover_cancelled(self.events);
            }

            fn on_axis_motion(&mut self, ev: &AxisMotionEvent) {
                EventHandlerApi::on_axis_motion(self.events, ev);
            }

            fn on_activation_token(&mut self, token: &str) {
                EventHandlerApi::on_activation_token(self.events, token);
            }

            fn on_tick(&mut self) {
                // Update engine state, then forward to EventHandler
                self.state.update();
                EventHandlerApi::on_update(self.events, self.state);
            }

            fn on_redraw(&mut self) {
                // Let user redraw callbacks run, then render
                EventHandlerApi::on_redraw(self.events);
                // RenderContext callbacks (immediate-mode drawing)
                let mut ctx = RenderContext::new(*self.window_size);
                self.events.on_render.invoke(&mut ctx);
                if let Some(color) = ctx.clear_color {
                    let [r, g, b, a] = color.to_linear_rgba();
                    self.renderer.set_clear_color([r, g, b, a]);
                }
                if !ctx.vertices.is_empty() {
                    self.renderer.submit(&ctx.vertices);
                }
                if !ctx.sprites.is_empty() {
                    self.renderer.draw_sprites(&ctx.sprites, *self.window_size);
                }
                if self.initialized {
                    let _ = self.renderer.present();
                }
            }
        }

        let mut forwarder = Forwarder {
            events: &mut self.events,
            renderer: self.renderer.as_mut(),
            initialized: false,
            state: &mut self.state,
            window_size: &mut self.window_size,
            window_config: self.window_config.as_ref(),
            assets: &self.assets,
        };

        self.backend.run(&mut forwarder)
    }
}
