use std::ffi::CString;
use std::ptr;

use ash::vk;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_TITLE: &str = "Ash Tutorial";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    window: Option<Window>,
}

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new()?;
    let mut app = VulkanApp::new();
    event_loop.run_app(&mut app).map_err(Into::into)
}

impl VulkanApp {
    pub fn new() -> Self {
        let _entry = unsafe { ash::Entry::load().unwrap() };
        let instance = VulkanApp::create_instance(&_entry);
        Self { 
            _entry,
            instance,
            window: None
        }
    }

    pub fn init_window(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId, anyhow::Error> {
        let attributes = winit::window::Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        let window = event_loop.create_window(attributes)?;
        let window_id = window.id();
        self.window = Some(window);

        Ok(window_id)
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        let app_name = CString::new(WINDOW_TITLE).expect("valid app name");
        let engine_name = CString::new(WINDOW_TITLE).expect("valid app name");
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_api_version(0, 0, 1, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 0, 1, 0),
            api_version: vk::API_VERSION_1_3,
            _marker: std::marker::PhantomData,
        };

        let extension_names = vec![vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr()];
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &app_info,
            flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            ..Default::default()
        };
        
        let instance = unsafe {
            entry.create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        instance
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
