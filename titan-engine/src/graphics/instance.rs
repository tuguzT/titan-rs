use std::error::Error;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_char;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use ash_window::enumerate_required_extensions;
use raw_window_handle::HasRawWindowHandle;

use crate::config::{Config, Version, ENGINE_NAME, ENGINE_VERSION};

use super::device::PhysicalDevice;
use super::ext::DebugUtils;
use super::utils;

lazy_static::lazy_static! {
    static ref VALIDATION_LAYER_NAME: &'static CStr = crate::c_str!("VK_LAYER_KHRONOS_validation");
}

const ENABLE_VALIDATION: bool = cfg!(debug_assertions);

pub struct Instance {
    version: Version,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    debug_utils: Option<DebugUtils>,
    instance_loader: ash::Instance,
    entry_loader: ash::Entry,
}

impl Instance {
    pub fn new(
        config: &Config,
        window_handle: &dyn HasRawWindowHandle,
    ) -> Result<Self, Box<dyn Error>> {
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
        let application_name = CString::new(config.name())?;
        let engine_name = CString::new(ENGINE_NAME)?;
        let application_version = utils::to_vk_version(&config.version());
        let engine_version = utils::to_vk_version(&ENGINE_VERSION);
        let application_info = vk::ApplicationInfo::builder()
            .application_version(application_version)
            .engine_version(engine_version)
            .application_name(application_name.deref())
            .engine_name(engine_name.deref())
            .api_version(vk::API_VERSION_1_2);

        // Initialize containers for layers' and extensions' names
        let _available_layer_properties_names = available_layer_properties
            .iter()
            .map(|item| unsafe { CStr::from_ptr(item.layer_name.as_ptr()) });
        let mut available_extension_properties_names = available_extension_properties
            .iter()
            .map(|item| unsafe { CStr::from_ptr(item.extension_name.as_ptr()) });
        let mut enabled_layer_names: Vec<&CStr> = Vec::new();
        let mut enabled_extension_names = Vec::new();

        // Push names' pointers into containers if validation was enabled
        if ENABLE_VALIDATION {
            enabled_layer_names.push(*VALIDATION_LAYER_NAME);
            if available_extension_properties_names.any(|item| item == DebugUtils::name()) {
                enabled_extension_names.push(DebugUtils::name());
            }
        }

        // Push extensions' names for surface
        let surface_extensions_names = enumerate_required_extensions(window_handle)?;
        enabled_extension_names.extend(surface_extensions_names.into_iter());

        // Initialize instance create info and get an instance
        let p_enabled_layer_names: Vec<*const c_char> = enabled_layer_names
            .iter()
            .map(|item| item.as_ptr())
            .collect();
        let p_enabled_extension_names: Vec<*const c_char> = enabled_extension_names
            .iter()
            .map(|item| item.as_ptr())
            .collect();
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(application_info.deref())
            .enabled_layer_names(p_enabled_layer_names.as_slice())
            .enabled_extension_names(p_enabled_extension_names.as_slice());
        let instance_loader = unsafe { entry_loader.create_instance(&create_info, None)? };

        // Initialize debug utils extension
        let debug_utils =
            if ENABLE_VALIDATION && enabled_extension_names.contains(&DebugUtils::name()) {
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
                enabled_layer_names.contains(&unsafe { CStr::from_ptr(item.layer_name.as_ptr()) })
            })
            .collect();

        // Enumerate enabled extensions
        let extension_properties = available_extension_properties
            .into_iter()
            .filter(|item| {
                enabled_extension_names
                    .contains(&unsafe { CStr::from_ptr(item.extension_name.as_ptr()) })
            })
            .collect();

        Ok(Self {
            entry_loader,
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

    pub fn entry_loader(&self) -> &ash::Entry {
        &self.entry_loader
    }

    pub fn loader(&self) -> &ash::Instance {
        &self.instance_loader
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, Box<dyn Error>> {
        let handles = unsafe { self.instance_loader.enumerate_physical_devices()? };
        handles
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
