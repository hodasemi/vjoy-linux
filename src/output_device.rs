use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, Result};
use evdev::{uinput::VirtualDevice, AbsoluteAxisCode, AttributeSet, KeyCode, UinputAbsSetup};

use crate::{descriptor::OutputType, input_device::InputDevice, VJoyDescriptor};

pub enum Output {
    Combined(OutputDevice),
    Passthrough(Vec<OutputDevice>),
}

pub struct OutputDevice {
    device: VirtualDevice,
}

impl OutputDevice {
    pub fn new(descriptor: &VJoyDescriptor, input_devices: &[InputDevice]) -> Result<Output> {
        match &descriptor.output_device {
            OutputType::Combined(output_device) => {
                let keys: AttributeSet<KeyCode> = descriptor
                    .key_mappings
                    .values()
                    .filter_map(|&b| TryInto::<KeyCode>::try_into(b).ok())
                    .collect();

                let mut builder = VirtualDevice::builder()?
                    .name(&output_device)
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

                Ok(Output::Combined(Self {
                    device: builder.build()?,
                }))
            }
            OutputType::Passthrough(output_devices) => Ok(Output::Passthrough(
                output_devices
                    .into_iter()
                    .enumerate()
                    .map(|(index, output_device)| {
                        let keys: AttributeSet<KeyCode> = descriptor
                            .key_mappings
                            .iter()
                            .filter_map(|(&(input_index, _), &b)| {
                                (index == input_index)
                                    .then_some(TryInto::<KeyCode>::try_into(b).ok())
                                    .flatten()
                            })
                            .collect();

                        let mut builder = VirtualDevice::builder()?
                            .name(&output_device)
                            .with_keys(&keys)?;

                        for (&(input_index, src_axis), &dst_axis) in descriptor.axis_mappings.iter()
                        {
                            if index == input_index {
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
                        }

                        Ok(Self {
                            device: builder.build()?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
            )),
        }
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
