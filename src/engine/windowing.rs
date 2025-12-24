use std::sync::Arc;
use std::time::Instant;

use crate::engine::{EngineError, EngineResult};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// Minimal winit wrapper (2025 winit style: ApplicationHandler).
pub struct Windowing;

impl Windowing {
    pub fn run_app(
        mut universe: crate::engine::Universe,
        mut renderer: crate::engine::graphics::Renderer,
    ) -> EngineResult<()> {
        let event_loop = EventLoop::new().map_err(|_| EngineError::NotImplemented)?;
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut app = App {
            window: None,
            universe: Some(universe),
            renderer: Some(renderer),
            last_frame: None,
        };

        event_loop
            .run_app(&mut app)
            .map_err(|_| EngineError::NotImplemented)?;

        Ok(())
    }
}

struct App {
    window: Option<Arc<Window>>,
    universe: Option<crate::engine::Universe>,
    renderer: Option<crate::engine::graphics::Renderer>,
    last_frame: Option<Instant>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs: WindowAttributes = Window::default_attributes()
            .with_title("rvx")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

        let window = event_loop
            .create_window(attrs)
            .expect("failed to create window");
        let window = Arc::new(window);

        // Initialize renderer backend for this window (Vulkan surface/swapchain later)
        if let Some(renderer) = self.renderer.as_mut() {
            renderer
                .init_for_window(&window)
                .expect("renderer init failed");
        }

        self.window = Some(window);
        self.last_frame = Some(Instant::now());

        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = self
                    .last_frame
                    .replace(now)
                    .map(|prev| (now - prev).as_secs_f32())
                    .unwrap_or(0.0);

                let universe = self.universe.as_mut().expect("universe missing");
                let renderer = self.renderer.as_mut().expect("renderer missing");

                universe.update(dt);
                universe.sync_visuals();

                renderer.draw_frame(&universe.visuals).expect("draw failed");

                if let Some(w) = &self.window {
                    w.pre_present_notify();
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }
}
