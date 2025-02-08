mod input_device;
mod output_device;

use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use evdev::{AbsoluteAxisCode, EventSummary, EventType, InputEvent, KeyCode};
use input_device::InputDevice;
use output_device::OutputDevice;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

macro_rules! create_mapping {
    ( $name:ident, $mapper:ident, [ $( $btn:ident $(,)? )+ ] ) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum $name {
            $(
                $btn,
            )+
        }

        impl From<$mapper> for $name {
            fn from(key_code: $mapper) -> Self {
                match key_code {
                    $(
                        $mapper::$btn => Self::$btn,
                    )+
                    _ => panic!()
                }
            }
        }

        impl Into<$mapper> for $name {
            fn into(self) -> $mapper {
                match self {
                    $(
                        Self::$btn => $mapper::$btn,
                    )+
                }
            }
        }
    };
}

create_mapping!(
    Button,
    KeyCode,
    [
        BTN_0, BTN_1, BTN_2, BTN_3, BTN_4, BTN_5, BTN_6, BTN_7, BTN_8, BTN_9, BTN_LEFT, BTN_RIGHT,
        BTN_MIDDLE, BTN_SIDE
    ]
);

create_mapping!(
    Axis,
    AbsoluteAxisCode,
    [
        ABS_X,
        ABS_Y,
        ABS_Z,
        ABS_RX,
        ABS_RY,
        ABS_RZ,
        ABS_THROTTLE,
        ABS_RUDDER,
        ABS_WHEEL,
        ABS_BRAKE
    ]
);

#[derive(Debug, Serialize, Deserialize)]
pub struct VJoyDescriptor {
    pub input_devices: Vec<String>,
    pub output_device: String,

    pub key_mappings: HashMap<(usize, Button), Button>,
    pub axis_mappings: HashMap<(usize, Axis), Axis>,
}

/// Linux vjoy cli
#[derive(Debug, Parser)]
#[command(version = "0.1")]
#[command(about, long_about = None)]
struct Args {
    /// VJoyDescriptor file
    #[arg(short = 'f', long = "file")]
    descriptor_file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let descriptor: VJoyDescriptor = from_str(
        &read_to_string(args.descriptor_file)
            .map_err(|err| anyhow!("failed to open descriptor file: {err:?}"))?,
    )
    .map_err(|err| anyhow!("failed to parse descriptor file: {err:?}"))?;

    let (input_devices, receiver) =
        InputDevice::find_unique_input_devices(&descriptor.input_devices);

    println!(
        "input devices: {:#?}",
        input_devices
            .iter()
            .map(|d| (d.path(), d.device().name()))
            .collect::<Vec<_>>()
    );

    let mut output_device = OutputDevice::new(&descriptor, &input_devices)?;

    input_devices
        .into_iter()
        .for_each(|device| device.start_event_loop());

    loop {
        let (index, input) = receiver.recv()?;

        match input {
            EventSummary::Key(_, key_code, state) => {
                println!("device {index} sent key event {key_code:?} in state {state}");

                if let Some(button) = descriptor.key_mappings.get(&(index, key_code.into())) {
                    output_device.emit(&[InputEvent::new(
                        EventType::KEY.0,
                        Into::<KeyCode>::into(*button).0,
                        state,
                    )])?;
                }
            }
            EventSummary::AbsoluteAxis(_, axis, value) => {
                println!("device {index} sent axis {axis:?} with {value}");

                if let Some(axis) = descriptor.axis_mappings
            }

            _ => (),
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
    fn print_devices() {
        println!(
            "{:#?}",
            evdev::enumerate()
                .map(|(p, d)| (p, d.name().map(|s| s.to_string())))
                .collect::<Vec<_>>()
        );
    }

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
