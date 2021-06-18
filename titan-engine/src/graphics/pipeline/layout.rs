use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use super::{
    super::{
        device::{self, Device},
        slotmap::SlotMappable,
    },
    utils,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct PipelineLayout {
    key: Key,
    handle: vk::PipelineLayout,
    parent_device: device::Key,
}

impl PipelineLayout {
    pub unsafe fn with(
        device_key: device::Key,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<Key, Box<dyn Error>> {
        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;
        let handle = device.loader().create_pipeline_layout(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write()?;
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn new(device_key: device::Key) -> Result<Key, Box<dyn Error>> {
        let create_info = vk::PipelineLayoutCreateInfo::default();
        unsafe { Self::with(device_key, &create_info) }
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let slotmap_device = match Device::slotmap().read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_pipeline_layout(self.handle, None) }
    }
}
