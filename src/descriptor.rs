use std::collections::HashMap;

use crate::{
    input_device::InputDevice,
    mappings::{Axis, Button},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationDescription {
    pub input: Vec<String>,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VJoyDescriptor {
    pub input_devices: Vec<String>,
    pub output_device: String,

    pub key_mappings: HashMap<(usize, Button), Button>,
    pub axis_mappings: HashMap<(usize, Axis), Axis>,
}

impl VJoyDescriptor {
    pub fn generate_descriptor(generation: GenerationDescription) -> Self {
        let mut stub_devices = InputDevice::find_unique_input_devices(&generation.input).0;
        let passthrough_device = stub_devices.remove(0);

        let mut key_mappings = HashMap::new();
        let mut axis_mappings = HashMap::new();

        if let Some(keys) = passthrough_device.device().supported_keys() {
            for key in keys.iter() {
                key_mappings.insert((0, key.into()), key.into());
            }
        }

        if let Some(axes) = passthrough_device.device().supported_absolute_axes() {
            for axis in axes.iter() {
                axis_mappings.insert((0, axis.into()), axis.into());
            }
        }

        for (index, stub_device) in stub_devices.iter().enumerate() {
            if let Some(keys) = stub_device.device().supported_keys() {
                for key in keys.iter() {
                    key_mappings.insert((index + 1, key.into()), Button::Stub);
                }
            }

            if let Some(axes) = stub_device.device().supported_absolute_axes() {
                for axis in axes.iter() {
                    axis_mappings.insert((index + 1, axis.into()), Axis::Stub);
                }
            }
        }

        Self {
            input_devices: generation.input,
            output_device: generation.output,

            key_mappings,
            axis_mappings,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs};

    use anyhow::Result;
    use serde_json::to_string_pretty;

    use crate::VJoyDescriptor;

    #[test]
    fn create_file() -> Result<()> {
        let desc = VJoyDescriptor {
            input_devices: vec![
                "Thrustmaster T.16000M".to_string(),
                "Thrustmaster T.16000M".to_string(),
            ],
            output_device: "".to_string(),
            key_mappings: HashMap::new(),
            axis_mappings: HashMap::new(),
        };

        fs::write("example_descriptor.json", &to_string_pretty(&desc)?)?;

        Ok(())
    }
}
