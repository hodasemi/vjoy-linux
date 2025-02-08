use std::path::PathBuf;

use evdev::{enumerate, Device};

pub struct InputDevice {
    path: String,
    device: Device,
}

impl InputDevice {
    pub fn find_unique_input_devices(input_device_names: &[String]) -> Vec<Self> {
        let mut input_devices: Vec<(PathBuf, Device)> = Vec::new();

        for name in input_device_names {
            for (path, device) in enumerate() {
                if let Some(device_name) = device.name() {
                    if name == device_name && !input_devices.iter().any(|(p, _)| path == *p) {
                        input_devices.push((path, device));
                        break;
                    }
                }
            }
        }

        input_devices
            .into_iter()
            .map(|(p, d)| Self {
                path: p.into_os_string().into_string().unwrap(),
                device: d,
            })
            .collect()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}
