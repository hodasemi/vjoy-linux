use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use anyhow::Result;
use evdev::{enumerate, Device, EventSummary};

pub struct InputDevice {
    index: usize,
    path: String,
    device: Device,
    sender: Sender<(usize, EventSummary)>,
}

impl InputDevice {
    pub fn find_unique_input_devices(
        input_device_names: &[String],
    ) -> (Vec<Self>, Receiver<(usize, EventSummary)>) {
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

        let (sender, receiver) = channel();

        (
            input_devices
                .into_iter()
                .enumerate()
                .map(|(index, (p, d))| Self {
                    index,
                    path: p.into_os_string().into_string().unwrap(),
                    device: d,
                    sender: sender.clone(),
                })
                .collect(),
            receiver,
        )
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn start_event_loop(mut self) {
        thread::spawn(move || -> Result<()> {
            loop {
                for event in self.device.fetch_events()? {
                    self.sender.send((self.index, event.destructure()))?;
                }
            }
        });
    }
}
