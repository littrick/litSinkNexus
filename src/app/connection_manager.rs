use anyhow::Context;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::pin;
use tracing::log;
use windows::{
    Devices::Enumeration::DevicePicker,
    Foundation::{Rect, TypedEventHandler},
    Media::Audio::{AudioPlaybackConnection, AudioPlaybackConnectionState},
    Win32::Foundation::{GetLastError, HWND},
    core::HSTRING,
};


#[derive(Clone)]
pub struct ConnectionManager {
    device_picker: DevicePicker,
    connections: Arc<Mutex<HashMap<String, AudioPlaybackConnection>>>,
}

impl ConnectionManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            device_picker: DevicePicker::new()?,
            connections: Default::default(),
        })
    }

    pub fn connect(&mut self, device_id: &str) -> anyhow::Result<()> {
        let connection = AudioPlaybackConnection::TryCreateFromId(&HSTRING::from(device_id))
            .context(format!(
                "Failed to create AudioPlaybackConnection for device ID: {device_id}",
            ))?;

        connection.Start().context(format!(
            "Failed to start AudioPlaybackConnection for device ID: {device_id}",
        ))?;

        let connections = self.connections.clone();
        connection.StateChanged(&TypedEventHandler::<AudioPlaybackConnection, _>::new(
            move |sender, arg| {
                let connection = sender.as_ref().unwrap();
                let state = sender.as_ref().unwrap().State().unwrap();
                Self::handle_state(connections.clone(), connection, state);
                Ok(())
            },
        ))?;

        self.connections
            .lock()
            .unwrap()
            .insert(device_id.to_string(), connection);
        Ok(())
    }

    fn handle_state(
        connections: Arc<Mutex<HashMap<String, AudioPlaybackConnection>>>,
        connection: &AudioPlaybackConnection,
        state: AudioPlaybackConnectionState,
    ) {
        match state {
            AudioPlaybackConnectionState::Opened => {
                log::info!("AudioPlaybackConnection opened");
            }
            AudioPlaybackConnectionState::Closed => {
                log::info!("AudioPlaybackConnection closed");
            }
            _ => {
                log::info!("AudioPlaybackConnection state changed: {:?}", state);
            }
        }
    }

    fn show(&self) -> anyhow::Result<()> {
        self.device_picker.Show(Rect {
            X: 100.0,
            Y: 100.0,
            Width: 300.0,
            Height: 400.0,
        })?;
        Ok(())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            device_picker: DevicePicker::new().unwrap(),
            connections: Default::default(),
        }
    }
}
