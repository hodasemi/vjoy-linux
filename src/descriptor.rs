use std::{collections::HashMap, fmt::Debug, fs, path::PathBuf};

use crate::{
    input_device::InputDevice,
    mappings::{Axis, Button},
};
use anyhow::{bail, Result};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationDescription {
    pub input: Vec<String>,
    pub output: OutputType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputType {
    Combined(String),
    Passthrough(Vec<String>),
}

impl OutputType {
    pub fn is_combined(&self) -> bool {
        match self {
            OutputType::Combined(_) => true,
            OutputType::Passthrough(_) => false,
        }
    }

    pub fn count(&self) -> usize {
        match self {
            OutputType::Combined(_) => 1,
            OutputType::Passthrough(items) => items.len(),
        }
    }
}

impl From<String> for OutputType {
    fn from(value: String) -> Self {
        if value.contains(',') {
            OutputType::Passthrough(value.split(',').map(|s| s.to_string()).collect())
        } else {
            OutputType::Combined(value)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VJoyDescriptor {
    pub input_devices: Vec<String>,
    pub output_device: OutputType,

    pub key_mappings: HashMap<(usize, Button), Button>,
    pub axis_mappings: HashMap<(usize, Axis), Axis>,
}

impl VJoyDescriptor {
    pub fn generate_from_cli(
        input_devices: String,
        output_device: Option<String>,
        output_file: Option<PathBuf>,
    ) -> Result<()> {
        let descriptor = Self::generate_descriptor(GenerationDescription {
            input: input_devices.split(',').map(|s| s.to_string()).collect(),
            output: output_device
                .unwrap_or("Combined Joystick".to_string())
                .into(),
        })?;

        fs::write(
            output_file.unwrap_or("stub_descriptor.ron".into()),
            &to_string_pretty(&descriptor, PrettyConfig::default())?,
        )?;

        Ok(())
    }

    pub fn generate_descriptor(generation: GenerationDescription) -> Result<Self> {
        if !generation.output.is_combined() && generation.output.count() != generation.input.len() {
            bail!("passthrough count must match input count!");
        }

        let mut stub_devices = InputDevice::find_unique_input_devices(&generation.input)?.0;
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
                    key_mappings.insert(
                        (index + 1, key.into()),
                        if generation.output.is_combined() {
                            Button::Stub
                        } else {
                            key.into()
                        },
                    );
                }
            }

            if let Some(axes) = stub_device.device().supported_absolute_axes() {
                for axis in axes.iter() {
                    axis_mappings.insert(
                        (index + 1, axis.into()),
                        if generation.output.is_combined() {
                            Axis::Stub
                        } else {
                            axis.into()
                        },
                    );
                }
            }
        }

        Ok(Self {
            input_devices: generation.input,
            output_device: generation.output,

            key_mappings,
            axis_mappings,
        })
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs};

    use anyhow::Result;
    use ron::ser::{to_string_pretty, PrettyConfig};

    use crate::VJoyDescriptor;

    use super::OutputType;

    #[test]
    fn create_empty_description_file() -> Result<()> {
        let desc = VJoyDescriptor {
            input_devices: vec![
                "Thrustmaster T.16000M".to_string(),
                "Thrustmaster T.16000M".to_string(),
            ],
            output_device: OutputType::Combined("".to_string()),
            key_mappings: HashMap::new(),
            axis_mappings: HashMap::new(),
        };

        fs::write(
            "example_descriptor.ron",
            &to_string_pretty(&desc, PrettyConfig::default())?,
        )?;

        Ok(())
    }

    #[test]
    fn generate_stub_description_file() -> Result<()> {
        let desc = VJoyDescriptor::generate_descriptor(super::GenerationDescription {
            input: vec![
                "Thrustmaster T.16000M".to_string(),
                "Thrustmaster T.16000M".to_string(),
            ],
            output: OutputType::Passthrough(vec![
                "Thrustmaster 1".to_string(),
                "Thrustmaster 2".to_string(),
            ]),
        })?;

        fs::write(
            "stub_descriptor.ron",
            &to_string_pretty(&desc, PrettyConfig::default())?,
        )?;

        Ok(())
    }
}
