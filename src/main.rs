use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_TITLE: &str = "Ash Tutorial";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct VulkanApp {
    window: Option<Window>,
}

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new()?;
    let mut app = VulkanApp::new();
    event_loop.run_app(&mut app).map_err(Into::into)
}

impl VulkanApp {
    pub fn init_window(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId, anyhow::Error> {
        let attributes = winit::window::Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        let window = event_loop.create_window(attributes)?;
        let window_id = window.id();
        self.window = Some(window);

        Ok(window_id)
    }

    pub fn new() -> Self {
        Self { window: None }
    }
}

impl ApplicationHandler for VulkanApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_window(event_loop)
            .expect("failed to create initial window");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.window = None;
                event_loop.exit();
            }
            _ => (),
        }
    }
}
