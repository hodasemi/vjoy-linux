use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, Result};
use evdev::{uinput::VirtualDevice, AbsoluteAxisCode, AttributeSet, KeyCode, UinputAbsSetup};

use crate::{input_device::InputDevice, VJoyDescriptor};

pub struct OutputDevice {
    device: VirtualDevice,
}

impl OutputDevice {
    pub fn new(descriptor: &VJoyDescriptor, input_devices: &[InputDevice]) -> Result<Self> {
        let keys: AttributeSet<KeyCode> = descriptor
            .key_mappings
            .values()
            .filter_map(|&b| TryInto::<KeyCode>::try_into(b).ok())
            .collect();

        let mut builder = VirtualDevice::builder()?
            .name(&descriptor.output_device)
            .with_keys(&keys)?;

        for (&(index, src_axis), &dst_axis) in descriptor.axis_mappings.iter() {
            if let Ok(dst_axis) = dst_axis.try_into() {
                let src_axis: AbsoluteAxisCode = src_axis.try_into()?;

                let device = &input_devices[index];
                let abs_info = device
                    .device()
                    .get_absinfo()?
                    .find_map(|(axis, info)| (axis == src_axis).then(|| info))
                    .ok_or(anyhow!(
                        "failed to find described axis ({src_axis:?}) for device {index}"
                    ))?;

                let abs_setup = UinputAbsSetup::new(dst_axis, abs_info);
                builder = builder.with_absolute_axis(&abs_setup)?;
            }
        }

        Ok(Self {
            device: builder.build()?,
        })
    }
}

impl Deref for OutputDevice {
    type Target = VirtualDevice;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DerefMut for OutputDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}
