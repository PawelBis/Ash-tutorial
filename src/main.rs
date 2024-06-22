use std::ffi::CString;
use std::ptr;

use ash::vk;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::{Window, WindowId};

const WINDOW_TITLE: &str = "Ash Tutorial";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

const VALIDATION_LAYERS: [&str; 1] = [
    "VK_LAYER_KHRONOS_validation",
];

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

struct VulkanApp {
    entry: ash::Entry,
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
            entry: _entry,
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

    fn check_validation_layers_support(entry: &ash::Entry) -> bool {
        let layers = unsafe { entry.enumerate_instance_layer_properties().unwrap() };
        for expected_layer in VALIDATION_LAYERS {
            let mut layer_found = false;
            for available_layer in &layers {
                if available_layer.layer_name_as_c_str().unwrap().to_str().unwrap() == expected_layer {
                    layer_found = true;
                    break;
                }
            }

            if !layer_found {
                return false;
            }
        }

        true
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

        let extension_properties = unsafe { entry.enumerate_instance_extension_properties(None).unwrap() };
        for ep in extension_properties {
            println!("Extension properties: {ep:?}");
        }

        if ENABLE_VALIDATION_LAYERS && !VulkanApp::check_validation_layers_support(&entry) {
            panic!("validation layers requrested but not available");
        }

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

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
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
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    self.window = None;
                    event_loop.exit();
                }
                _ => (),
            }
            _ => (),
        }
    }
}
