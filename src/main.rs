mod descriptor;
mod input_device;
mod mappings;
mod output_device;

use std::{fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use descriptor::VJoyDescriptor;
use evdev::{AbsoluteAxisCode, EventSummary, EventType, InputEvent, KeyCode};
use input_device::InputDevice;
use output_device::OutputDevice;
use serde_json::from_str;

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

                if let Some(axis) = descriptor.axis_mappings.get(&(index, axis.into())) {
                    output_device.emit(&[InputEvent::new(
                        EventType::ABSOLUTE.0,
                        Into::<AbsoluteAxisCode>::into(*axis).0,
                        value,
                    )])?;
                }
            }

            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn print_devices() {
        println!(
            "{:#?}",
            evdev::enumerate()
                .map(|(p, d)| (p, d.name().map(|s| s.to_string())))
                .collect::<Vec<_>>()
        );
    }
}
