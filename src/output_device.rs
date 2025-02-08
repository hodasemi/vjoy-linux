use anyhow::{anyhow, Result};
use evdev::{uinput::VirtualDevice, AttributeSet, KeyCode, UinputAbsSetup};

use crate::{input_device::InputDevice, VJoyDescriptor};

pub struct OutputDevice {
    device: VirtualDevice,
}

impl OutputDevice {
    pub fn new(descriptor: &VJoyDescriptor, input_devices: &[InputDevice]) -> Result<Self> {
        let keys: AttributeSet<KeyCode> = descriptor
            .key_mappings
            .values()
            .map(|&b| -> KeyCode { b.into() })
            .collect();

        let mut builder = VirtualDevice::builder()?
            .name(&descriptor.output_device)
            .with_keys(&keys)?;

        for (&(index, src_axis), &dst_axis) in descriptor.axis_mappings.iter() {
            let device = &input_devices[index];
            let abs_info = device
                .device()
                .get_absinfo()?
                .find_map(|(axis, info)| (axis == src_axis.into()).then(|| info))
                .ok_or(anyhow!(
                    "failed to find described axis ({src_axis:?}) for device {index}"
                ))?;

            let abs_setup = UinputAbsSetup::new(dst_axis.into(), abs_info);
            builder = builder.with_absolute_axis(&abs_setup)?;
        }

        Ok(Self {
            device: builder.build()?,
        })
    }
}
