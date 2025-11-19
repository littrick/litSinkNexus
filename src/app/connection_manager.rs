use anyhow::Context;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::log;
use windows::{
    Devices::Enumeration::*,
    Foundation::{Rect, TypedEventHandler},
    Media::Audio::*,
    Win32::{
        Foundation::*,
        UI::{Shell::IInitializeWithWindow, WindowsAndMessaging::*},
    },
    core::*,
};

use crate::internal::*;

#[derive(Debug, Clone)]
pub struct DeviceStatusStrings {
    connection_list_title: String,
    connecting: String,
    connected: String,
    timeout: String,
    denied_by_system: String,
    unknown_failure: String,
    unknown_reason: String,
    disconnected: String,
}

impl Default for DeviceStatusStrings {
    fn default() -> Self {
        Self {
            connection_list_title: "Connection List".to_string(),
            connecting: "Connecting".to_string(),
            connected: "Connected".to_string(),
            timeout: "Connection Timeout".to_string(),
            denied_by_system: "Connection Denied by System".to_string(),
            unknown_failure: "Unknown Connection Failure".to_string(),
            unknown_reason: "Unknown Reason".to_string(),
            disconnected: "Disconnected".to_string(),
        }
    }
}

#[derive(Debug)]
struct ConnectionContext {
    picker: DevicePicker,
    connections: Mutex<HashMap<HSTRING, (DeviceInformation, AudioPlaybackConnection)>>,
    strings: DeviceStatusStrings,
}

#[derive(Debug)]
pub struct ConnectionManager {
    window: HWND,
    context: Arc<ConnectionContext>,
}

impl ConnectionManager {
    pub fn new(window: HWND, strings: DeviceStatusStrings) -> anyhow::Result<Self> {
        let manager = Self {
            window,

            context: Arc::new(ConnectionContext {
                connections: Default::default(),
                picker: DevicePicker::new().context("Failed to create DevicePicker")?,
                strings,
            }),
        };
        manager
            .init_picker()
            .context("Failed to initialize DevicePicker")?;

        Ok(manager)
    }

    pub fn show(&self, rect: Rect) -> anyhow::Result<()> {
        log::info!("Showing Device Picker");

        log::debug!(
            "connections: {:?}",
            self.context.connections.lock().unwrap()
        );

        unsafe {
            SetWindowPos(
                self.window,
                Some(HWND_TOPMOST),
                0,
                0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
                SWP_HIDEWINDOW,
            )
        }
        .unwrap();

        self.context.picker.Show(rect)?;
        Ok(())
    }

    fn init_picker(&self) -> anyhow::Result<()> {
        unsafe {
            self.context
                .picker
                .cast::<IInitializeWithWindow>()
                .unwrap()
                .Initialize(self.window)
        }?;

        let selector = AudioPlaybackConnection::GetDeviceSelector()?;

        let all_device = DeviceInformation::FindAllAsyncAqsFilter(&selector)?
            .join()?
            .into_iter()
            .collect::<Vec<_>>();

        for device in all_device {
            log::debug!("Clearing device: {}({})", device.Name()?, device.Id()?);

            self.context
                .picker
                .SetDisplayStatus(
                    &device,
                    &HSTRING::from(""),
                    DevicePickerDisplayStatusOptions::None,
                )
                .context("Fail to clear picker display status")?;
        }

        self.context
            .picker
            .Filter()
            .context("Fail to get DevicePickerFilter")?
            .SupportedDeviceSelectors()
            .context("Fail to get Seletors")?
            .Append(&selector)
            .context("Fail to append selector")?;

        self.context
            .picker
            .DeviceSelected(&{
                let context = self.context.clone();
                TypedEventHandler::<_, DeviceSelectedEventArgs>::new(move |_, args| {
                    let device = args.as_ref().unwrap().SelectedDevice()?;

                    log::debug!("Device selected: {}({})", device.Name()?, device.Id()?);

                    Self::connect(context.clone(), &device).to_win_result()
                })
            })
            .context("Fail to set DeviceSeleted callback")?;

        self.context
            .picker
            .DisconnectButtonClicked(&{
                let context = self.context.clone();
                TypedEventHandler::<_, DeviceDisconnectButtonClickedEventArgs>::new(
                    move |_, args| {
                        let device = args.as_ref().unwrap().Device()?;
                        let device_id = device.Id().unwrap();
                        log::info!(
                            "Disconnecting device: {}",
                            device.Name().unwrap_or(HSTRING::from("(Unknown)"))
                        );

                        context.connections.lock().unwrap().remove(&device_id);
                        context
                            .picker
                            .SetDisplayStatus(
                                &device,
                                &HSTRING::from(&context.strings.disconnected),
                                DevicePickerDisplayStatusOptions::None,
                            )
                            .unwrap();

                        Ok(())
                    },
                )
            })
            .context("Fail to set DisconnectButtonClicked callback")?;

        self.context
            .picker
            .DevicePickerDismissed(&TypedEventHandler::new(move |_, _| {
                log::debug!("Device Picker Dismissed");
                Ok(())
            }))
            .context("Fail to set DevicePickerDismissed callback")?;

        self.context
            .picker
            .Appearance()?
            .SetTitle(&HSTRING::from(&self.context.strings.connection_list_title))?;

        Ok(())
    }

    fn connect(context: Arc<ConnectionContext>, device: &DeviceInformation) -> anyhow::Result<()> {
        context.picker.SetDisplayStatus(
            device,
            &HSTRING::from(&context.strings.connecting),
            DevicePickerDisplayStatusOptions::ShowProgress
                | DevicePickerDisplayStatusOptions::ShowDisconnectButton,
        )?;

        let device_id = device.Id().unwrap();

        let connection = AudioPlaybackConnection::TryCreateFromId(&device_id).context(format!(
            "Failed to create AudioPlaybackConnection for device ID: {device_id}",
        ))?;

        connection.StateChanged(&{
            let context = context.clone();
            TypedEventHandler::<AudioPlaybackConnection, _>::new(move |sender, _| {
                let connection = sender.as_ref().unwrap();
                let state = sender.as_ref().unwrap().State().unwrap();
                Self::handle_state(context.clone(), connection, state);
                Ok(())
            })
        })?;

        connection.Start().context(format!(
            "Failed to start AudioPlaybackConnection for device ID: {device_id}",
        ))?;
        match connection
            .Open()
            .context("Fail to open connection")?
            .Status()?
        {
            AudioPlaybackConnectionOpenResultStatus::Success => {
                log::info!("Connected to device: {}", device_id);
                context
                    .connections
                    .lock()
                    .unwrap()
                    .insert(device_id.clone(), (device.clone(), connection));
                context.picker.SetDisplayStatus(
                    device,
                    &HSTRING::from(&context.strings.connected),
                    DevicePickerDisplayStatusOptions::ShowDisconnectButton,
                )?;
            }
            AudioPlaybackConnectionOpenResultStatus::RequestTimedOut => {
                context.picker.SetDisplayStatus(
                    device,
                    &HSTRING::from(&context.strings.timeout),
                    DevicePickerDisplayStatusOptions::ShowRetryButton,
                )?;
            }
            AudioPlaybackConnectionOpenResultStatus::UnknownFailure => {
                context.picker.SetDisplayStatus(
                    device,
                    &HSTRING::from(&context.strings.unknown_failure),
                    DevicePickerDisplayStatusOptions::ShowRetryButton,
                )?;
            }
            AudioPlaybackConnectionOpenResultStatus::DeniedBySystem => {
                context.picker.SetDisplayStatus(
                    device,
                    &HSTRING::from(&context.strings.denied_by_system),
                    DevicePickerDisplayStatusOptions::ShowRetryButton,
                )?;
            }
            res => {
                log::error!("Failed to open connection: {:?}", res);
                context.picker.SetDisplayStatus(
                    device,
                    &HSTRING::from(&format!("{}{}", context.strings.unknown_reason, res.0)),
                    DevicePickerDisplayStatusOptions::ShowRetryButton,
                )?;
            }
        };
        Ok(())
    }

    fn handle_state(
        context: Arc<ConnectionContext>,
        connection: &AudioPlaybackConnection,
        state: AudioPlaybackConnectionState,
    ) {
        match state {
            AudioPlaybackConnectionState::Opened => {
                log::info!(
                    "Device opened: {}",
                    connection.DeviceId().unwrap().to_string_lossy()
                );
            }
            AudioPlaybackConnectionState::Closed => {
                let device_id = connection.DeviceId().unwrap();

                log::info!("AudioPlaybackConnection closed: {}", device_id);

                let connection = context.connections.lock().unwrap().remove(&device_id);

                if let Some((device, _)) = connection {
                    context
                        .picker
                        .SetDisplayStatus(
                            &device,
                            &HSTRING::from(&context.strings.disconnected),
                            DevicePickerDisplayStatusOptions::ShowRetryButton,
                        )
                        .unwrap();
                }
            }
            _ => {
                log::info!("AudioPlaybackConnection state changed: {:?}", state);
            }
        }
    }
}

impl Drop for ConnectionContext {
    fn drop(&mut self) {
        log::debug!("ConnectionContext dropping");
    }
}