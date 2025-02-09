mod descriptor;
mod input_device;
mod mappings;
mod output_device;

use std::{fs::read_to_string, path::PathBuf, process::Command, sync::mpsc::Receiver, thread};

use anyhow::{anyhow, Result};
use clap::Parser;
use descriptor::VJoyDescriptor;
use evdev::{AbsoluteAxisCode, EventSummary, EventType, InputEvent, KeyCode};
use input_device::InputDevice;
use output_device::{Output, OutputDevice};
use ron::from_str;

/// Linux vjoy cli
#[derive(Debug, Parser)]
#[command(version = "0.1")]
#[command(about, long_about = None)]
struct Args {
    /// VJoyDescriptor file
    #[arg(short = 'f', long = "file")]
    descriptor_file: Option<PathBuf>,

    /// Enable debug output
    #[arg(long = "debug", default_value_t = false)]
    debug: bool,

    /// Generator Input Devices (Comma separated)
    #[arg(short = 'i', long = "input")]
    input_devices: Option<String>,

    /// Generator Output Device
    #[arg(short = 'd', long = "device")]
    output_device: Option<String>,

    /// Generator File
    #[arg(short = 'o', long = "output")]
    generator_file: Option<PathBuf>,

    /// Program that should be started
    #[arg(short = 'p', long = "program")]
    program: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(input_devices) = args.input_devices {
        VJoyDescriptor::generate_from_cli(input_devices, args.output_device, args.generator_file)?;

        return Ok(());
    }

    let descriptor: VJoyDescriptor = from_str(
        &read_to_string(
            args.descriptor_file
                .ok_or(anyhow!("missing descriptor file (-f <path to file>)"))?,
        )
        .map_err(|err| anyhow!("failed to open descriptor file: {err:?}"))?,
    )
    .map_err(|err| anyhow!("failed to parse descriptor file: {err:?}"))?;

    let (input_devices, receiver) =
        InputDevice::find_unique_input_devices(&descriptor.input_devices)?;

    if args.debug {
        println!(
            "input devices: {:#?}",
            input_devices
                .iter()
                .map(|d| (d.path(), d.device().name()))
                .collect::<Vec<_>>()
        );
    }

    let output = OutputDevice::new(&descriptor, &input_devices)?;

    input_devices
        .into_iter()
        .for_each(|device| device.start_event_loop());

    match args.program {
        Some(command) => {
            thread::spawn(move || event_loop(output, receiver, descriptor, args.debug));

            println!("command: {command}");

            let mut split = command.split_whitespace();
            let mut command = Command::new(split.next().unwrap());

            while let Some(s) = split.next() {
                command.arg(s);
            }

            if args.debug {
                println!("command: {command:?}");
            }

            command.output().unwrap();
        }
        None => {
            event_loop(output, receiver, descriptor, args.debug)?;
        }
    }

    Ok(())
}

fn event_loop(
    mut output: Output,
    receiver: Receiver<(usize, EventSummary)>,
    descriptor: VJoyDescriptor,
    debug: bool,
) -> Result<()> {
    loop {
        let (index, input) = receiver.recv()?;

        match input {
            EventSummary::Key(_, key_code, state) => {
                if debug {
                    println!("device {index} sent key event {key_code:?} in state {state}");
                }

                if let Some(button) = descriptor.key_mappings.get(&(index, key_code.into())) {
                    if let Ok(code) = TryInto::<KeyCode>::try_into(*button) {
                        match &mut output {
                            Output::Combined(output_device) => {
                                output_device.emit(&[InputEvent::new(
                                    EventType::KEY.0,
                                    code.0,
                                    state,
                                )])?;
                            }
                            Output::Passthrough(output_devices) => {
                                output_devices[index].emit(&[InputEvent::new(
                                    EventType::KEY.0,
                                    code.0,
                                    state,
                                )])?;
                            }
                        }
                    }
                }
            }
            EventSummary::AbsoluteAxis(_, axis, value) => {
                if debug {
                    println!("device {index} sent axis {axis:?} with {value}");
                }

                if let Some(axis) = descriptor.axis_mappings.get(&(index, axis.into())) {
                    if let Ok(axis) = TryInto::<AbsoluteAxisCode>::try_into(*axis) {
                        match &mut output {
                            Output::Combined(output_device) => {
                                output_device.emit(&[InputEvent::new(
                                    EventType::ABSOLUTE.0,
                                    axis.0,
                                    value,
                                )])?;
                            }
                            Output::Passthrough(output_devices) => {
                                output_devices[index].emit(&[InputEvent::new(
                                    EventType::ABSOLUTE.0,
                                    axis.0,
                                    value,
                                )])?;
                            }
                        }
                    }
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
