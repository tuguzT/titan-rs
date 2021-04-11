use std::error::Error;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

use crate::config::Config;
use crate::graphics::debug::DebugUtils;
use crate::graphics::device::PhysicalDevice;
use crate::graphics::utils;
use crate::version::Version;

const VALIDATION_LAYER_NAME: *const c_char = crate::c_str_ptr!("VK_LAYER_KHRONOS_validation");

const ENABLE_VALIDATION: bool = cfg!(debug_assertions);

pub struct Instance {
    version: Version,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    debug_utils: Option<DebugUtils>,
    instance_loader: ash::Instance,
    _entry_loader: ash::Entry,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // Get entry loader and Vulkan API version
        let entry_loader = unsafe { ash::Entry::new()? };
        let version = match entry_loader.try_enumerate_instance_version()? {
            Some(version) => utils::from_vk_version(version),
            None => utils::from_vk_version(vk::API_VERSION_1_0),
        };

        // Get available instance properties
        let available_layer_properties = entry_loader.enumerate_instance_layer_properties()?;
        let available_extension_properties =
            entry_loader.enumerate_instance_extension_properties()?;

        // Setup application info for Vulkan API
        let application_name = CString::new(config.app_name())?;
        let engine_name = CString::new(config.engine_name())?;
        let application_info = vk::ApplicationInfo {
            application_version: utils::to_vk_version(&config.app_version()),
            engine_version: utils::to_vk_version(&config.engine_version()),
            p_application_name: application_name.as_ptr(),
            p_engine_name: engine_name.as_ptr(),
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };

        // Initialize containers for layers' and extensions' names
        let _available_layer_properties_names = available_layer_properties
            .iter()
            .map(|item| unsafe { CStr::from_ptr(item.layer_name.as_ptr()) });
        let mut available_extension_properties_names = available_extension_properties
            .iter()
            .map(|item| unsafe { CStr::from_ptr(item.extension_name.as_ptr()) });
        let mut enabled_layer_properties_names: Vec<&CStr> = Vec::new();
        let mut enabled_extension_properties_names: Vec<&CStr> = Vec::new();

        // Push names' pointers into containers if validation was enabled
        let validation_layer_name = unsafe { CStr::from_ptr(VALIDATION_LAYER_NAME) };
        if ENABLE_VALIDATION {
            enabled_layer_properties_names.push(validation_layer_name);
            if available_extension_properties_names.any(|item| item == DebugUtils::name()) {
                enabled_extension_properties_names.push(DebugUtils::name());
            }
        }

        // Initialize instance create info and get an instance
        let p_enabled_layer_properties_names: Vec<*const c_char> = enabled_layer_properties_names
            .iter()
            .map(|item| item.as_ptr())
            .collect();
        let p_enabled_extension_properties_names: Vec<*const c_char> =
            enabled_extension_properties_names
                .iter()
                .map(|item| item.as_ptr())
                .collect();
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            enabled_layer_count: p_enabled_layer_properties_names.len() as u32,
            pp_enabled_layer_names: p_enabled_layer_properties_names.as_ptr(),
            enabled_extension_count: p_enabled_extension_properties_names.len() as u32,
            pp_enabled_extension_names: p_enabled_extension_properties_names.as_ptr(),
            ..Default::default()
        };
        let instance_loader = unsafe { entry_loader.create_instance(&create_info, None)? };

        // Initialize debug utils extension
        let debug_utils = if ENABLE_VALIDATION
            && enabled_extension_properties_names.contains(&DebugUtils::name())
        {
            let returnable = DebugUtils::new(&entry_loader, &instance_loader)?;
            log::info!("Vulkan validation layer enabled");
            Some(returnable)
        } else {
            None
        };

        // Enumerate enabled layers
        let layer_properties = available_layer_properties
            .into_iter()
            .filter(|item| {
                enabled_layer_properties_names
                    .contains(&unsafe { CStr::from_ptr(item.layer_name.as_ptr()) })
            })
            .collect();

        // Enumerate enabled extensions
        let extension_properties = available_extension_properties
            .into_iter()
            .filter(|item| {
                enabled_extension_properties_names
                    .contains(&unsafe { CStr::from_ptr(item.extension_name.as_ptr()) })
            })
            .collect();

        Ok(Self {
            _entry_loader: entry_loader,
            instance_loader,
            version,
            layer_properties,
            extension_properties,
            debug_utils,
        })
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn layer_properties(&self) -> &Vec<vk::LayerProperties> {
        &self.layer_properties
    }

    pub fn extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.extension_properties
    }

    pub fn loader(&self) -> &ash::Instance {
        &self.instance_loader
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, Box<dyn Error>> {
        unsafe { self.instance_loader.enumerate_physical_devices()? }
            .iter()
            .map(|handle| PhysicalDevice::new(self, *handle))
            .collect()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.debug_utils = None;
        unsafe {
            self.instance_loader.destroy_instance(None);
        }
    }
}
