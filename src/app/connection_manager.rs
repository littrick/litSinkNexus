use crate::internal::*;
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

#[derive(Debug, Clone)]
pub struct DeviceStatusStrings {
    pub picker_title: String,
    pub connecting: String,
    pub connected: String,
    pub timeout: String,
    pub denied_by_system: String,
    pub not_found: String,
    pub unknown_reason: String,
    pub disconnected: String,
}

impl Default for DeviceStatusStrings {
    fn default() -> Self {
        Self {
            picker_title: "A2DP Sink: Click to select a source device ".to_string(),
            connecting: "Connecting".to_string(),
            connected: "Connected".to_string(),
            timeout: "Connection Timeout".to_string(),
            denied_by_system: "Connection Denied by System".to_string(),
            not_found: "Device not found".to_string(),
            unknown_reason: "Unknown Reason".to_string(),
            disconnected: "Disconnected".to_string(),
        }
    }
}

#[derive(Debug)]
struct ConnectionContext {
    window: WndHandle,
    picker: DevicePicker,
    connections: Mutex<HashMap<HSTRING, (DeviceInformation, AudioPlaybackConnection)>>,
    strings: DeviceStatusStrings,
}

#[derive(Debug)]
pub struct ConnectionManager {
    context: Arc<ConnectionContext>,
}

impl ConnectionManager {
    pub fn new(window: HWND, strings: DeviceStatusStrings) -> anyhow::Result<Self> {
        let manager = Self {
            context: Arc::new(ConnectionContext {
                window: WndHandle::new(window),
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
                self.context.window.hwnd(),
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
        let context = &self.context;
        let picker = &self.context.picker;

        unsafe {
            picker
                .cast::<IInitializeWithWindow>()
                .unwrap()
                .Initialize(context.window.hwnd())
        }?;

        let selector = AudioPlaybackConnection::GetDeviceSelector()?;

        let all_device = DeviceInformation::FindAllAsyncAqsFilter(&selector)?
            .join()?
            .into_iter()
            .collect::<Vec<_>>();

        for device in all_device {
            log::debug!("Clearing device: {}({})", device.Name()?, device.Id()?);

            picker
                .SetDisplayStatus(
                    &device,
                    &HSTRING::from(""),
                    DevicePickerDisplayStatusOptions::None,
                )
                .context("Fail to clear picker display status")?;
        }

        picker
            .Filter()
            .context("Fail to get DevicePickerFilter")?
            .SupportedDeviceSelectors()
            .context("Fail to get Seletors")?
            .Append(&selector)
            .context("Fail to append selector")?;

        picker
            .DeviceSelected(&{
                let context = context.clone();
                TypedEventHandler::<_, DeviceSelectedEventArgs>::new(move |_, args| {
                    let device = args.as_ref().unwrap().SelectedDevice()?;

                    log::info!("Connecting to: {}({})", device.Name()?, device.Id()?);
                    Self::connect(context.clone(), &device).to_win_result()
                })
            })
            .context("Fail to set DeviceSeleted callback")?;

        picker
            .DisconnectButtonClicked(&{
                let context = context.clone();
                TypedEventHandler::<_, DeviceDisconnectButtonClickedEventArgs>::new(
                    move |_, args| {
                        let device = args.as_ref().unwrap().Device()?;
                        let device_id = device.Id().unwrap();
                        log::info!(
                            "Disconnecting device: {}({})",
                            device.Name().unwrap_or(HSTRING::from("(Unknown)")),
                            device_id
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

        picker
            .DevicePickerDismissed(&{
                let context = context.clone();
                TypedEventHandler::new(move |_, _| {
                    log::debug!("Device Picker Dismissed");

                    unsafe {
                        SetWindowPos(
                            context.window.hwnd(),
                            None,
                            0,
                            0,
                            0,
                            0,
                            SWP_HIDEWINDOW | SWP_NOZORDER,
                        )
                    }?;

                    Ok(())
                })
            })
            .context("Fail to set DevicePickerDismissed callback")?;

        picker
            .Appearance()?
            .SetTitle(&HSTRING::from(&context.strings.picker_title))?;

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
                log::info!(
                    "Device connected: {}({})",
                    device.Name().unwrap_or(HSTRING::from("(Unknown)")),
                    device_id
                );
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
                    &HSTRING::from(&context.strings.not_found), // Error reported here when device cannot be scanned
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
                log::debug!("Device opened: {}", connection.DeviceId().unwrap());
            }
            AudioPlaybackConnectionState::Closed => {
                let device_id = connection.DeviceId().unwrap();
                log::debug!("AudioPlaybackConnection closed: {}", device_id);
                let connection = context.connections.lock().unwrap().remove(&device_id);

                if let Some((device, _)) = connection {
                    log::info!(
                        "Device disconnected: {}({})",
                        device.Name().unwrap_or(HSTRING::from("(Unknown)")),
                        device_id
                    );

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
