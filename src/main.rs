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

#[derive(Default)]
struct QueueFamilyIndices {
    graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

struct VulkanApp {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    window: Option<Window>,
}

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new()?;
    let mut app = VulkanApp::new();
    event_loop.run_app(&mut app).map_err(Into::into)
}

impl VulkanApp {
    pub fn new() -> Self {
        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = VulkanApp::create_instance(&entry);
        let physical_device = VulkanApp::pick_physical_device(&instance);
        let device = VulkanApp::create_logical_device(&instance, physical_device);

        let indices = VulkanApp::find_queue_families(&instance, physical_device);
        let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        Self { 
            entry,
            instance,
            physical_device,
            device,
            graphics_queue,
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

    fn find_queue_families(instance: &ash::Instance, device: vk::PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::default();
        let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (index, queue_properties) in queue_family_properties.iter().enumerate() {
            if queue_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(index as u32);
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn is_device_suitable(instance: &ash::Instance, device: vk::PhysicalDevice) -> bool {
        let indices = VulkanApp::find_queue_families(&instance, device);
        return indices.graphics_family.is_some();
    }

    fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
        let devices = unsafe {
            instance.enumerate_physical_devices().expect("enumerate devices")
        };
        if devices.is_empty() {
            panic!("failed to select physical device");
        };

        for device in devices {
            if VulkanApp::is_device_suitable(&instance, device) {
                return device;
            }
        }

        panic!("failed to find a suitable GPU");
    }

    fn create_logical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> ash::Device {
        let indices = VulkanApp::find_queue_families(&instance, physical_device);
        let queue_priority = 1.0;
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            queue_family_index: indices.graphics_family.unwrap(),
            queue_count: 1,
            p_queue_priorities: &queue_priority,
            ..Default::default()
        };

        let extension_names = vec![vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr()];
        let device_features = vk::PhysicalDeviceFeatures::default();
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_queue_create_infos: &queue_create_info,
            queue_create_info_count: 1,
            p_enabled_features: &device_features,
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            ..Default::default()
        };

        unsafe { instance.create_device(physical_device, &device_create_info, None).unwrap() }
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

        let validation_layers: Vec<CString> = if ENABLE_VALIDATION_LAYERS {
            VALIDATION_LAYERS.clone().into_iter().map(|s| CString::new(s).unwrap()).collect()
        } else {
            Vec::new()
        };

        let validation_layer_names: Vec<*const i8> = validation_layers.iter().map(|n| n.as_ptr())
            .collect();

        let extension_names = vec![vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr()];
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &app_info,
            flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_layer_count: validation_layers.len() as u32,
            pp_enabled_layer_names: validation_layer_names.as_ptr(),
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
            self.device.destroy_device(None);
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
