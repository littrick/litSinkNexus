use std::{collections::HashMap, mem::size_of, sync::Mutex};
use tracing::log::info;

use windows::{
    Devices::Enumeration::*,
    Foundation::*,
    Media::Audio::*,
    System::Launcher,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

const WM_NOTIFYICON: u32 = WM_APP + 1;

// 定义菜单项 ID
const IDM_EXIT: u32 = 1001;
const IDM_SHOW: u32 = 1002;
const IDM_ABOUT: u32 = 1003;
const IDM_SHOW_SETTINGS: u32 = 1004;

lazy_static::lazy_static! {
    static ref DEVICES:Mutex<HashMap<String, AudioPlaybackConnection>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let h_instance = unsafe { GetModuleHandleW(None) }?;

    let window_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        hInstance: h_instance.into(),
        lpszClassName: w!("awindow"),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }?,
        lpfnWndProc: Some(wnproc),
        hbrBackground: HBRUSH(unsafe { GetStockObject(WHITE_BRUSH) }.0),
        ..Default::default()
    };

    let class_atom = unsafe { RegisterClassExW(&window_class) };
    anyhow::ensure!(
        class_atom != 0,
        "Register class fail, Error: {:?}",
        unsafe { GetLastError() }
    );

    let _window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class.lpszClassName,
            w!("windows name"),
            WS_POPUP,
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            None,
            None,
            Some(h_instance.into()),
            None,
        )
    }?;

    let mut msg = MSG::default();

    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        // info!("{}:{} got message {msg:?}", file!(), line!());

        let _ = unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }

    Ok(())
}

extern "system" fn wnproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // info!("{}:{} message: 0x{message:X} wparam: 0x{:X} lparam: 0x{:X}", file!(), line!(), wparam.0, lparam.0);

    let notify_icon_data = NOTIFYICONDATAW {
        hWnd: window,
        uFlags: NIF_ICON | NIF_MESSAGE,
        uCallbackMessage: WM_NOTIFYICON,
        Anonymous: NOTIFYICONDATAW_0 { uVersion: 4 },
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap(),
        ..Default::default()
    };

    match message {
        WM_CREATE => {
            info!("WM_CREATE");
            unsafe { Shell_NotifyIconW(NIM_ADD, &notify_icon_data) }.unwrap();
            unsafe { Shell_NotifyIconW(NIM_SETVERSION, &notify_icon_data) }.unwrap();

            // show_device_picker(window);
            // show_device_picker2(window);
        }
        WM_DESTROY => {
            info!("WM_DESTROY");
            unsafe { PostQuitMessage(0) };
            unsafe { Shell_NotifyIconW(NIM_DELETE, &notify_icon_data) }.unwrap();
        }
        WM_NOTIFYICON => {
            let lp = lparam.0 as u32;
            // info!("WM_NOTIFYICON event: lp=0x{lp:X} wp=0x{:X}", wparam.0);
            match lp {
                WM_LBUTTONUP => {
                    info!("Notify Icon Left Button Up");
                }
                WM_RBUTTONUP => {
                    info!("Notify Icon Right Button Up");
                    show_context_menu(window);
                }
                NIN_SELECT => {
                    info!("Notify Icon Select");

                    show_device_picker(window);
                }
                _ => {}
            }
        }
        WM_COMMAND => {
            // 处理菜单项点击
            let menu_id = wparam.0 as u32;
            info!("WM_COMMAND menu_id: {}", menu_id);
            match menu_id {
                IDM_SHOW_SETTINGS => {
                    info!("打开蓝牙设备列表");
                    let uri =
                        Uri::CreateUri(&windows::core::HSTRING::from("ms-settings:bluetooth"))
                            .unwrap();
                    Launcher::LaunchUriAsync(&uri).unwrap();
                }
                IDM_SHOW => {
                    info!("显示窗口");
                }
                IDM_ABOUT => {
                    unsafe {
                        MessageBoxW(
                            Some(window),
                            w!("系统托盘示例程序"),
                            w!("关于"),
                            MB_OK | MB_ICONINFORMATION,
                        )
                    };
                }
                IDM_EXIT => {
                    unsafe { PostQuitMessage(0) };
                }
                _ => {}
            }
        }
        _ => {}
    };

    unsafe { DefWindowProcW(window, message, wparam, lparam) }

    // LRESULT(0)
}

fn show_context_menu(hwnd: HWND) {
    // 创建弹出菜单
    let hmenu = unsafe { CreatePopupMenu() }.unwrap();

    // 添加菜单项
    unsafe {
        AppendMenuW(
            hmenu,
            MF_STRING,
            IDM_SHOW_SETTINGS as usize,
            w!("打开蓝牙设备列表"),
        )
    }
    .unwrap();
    unsafe { AppendMenuW(hmenu, MF_STRING, IDM_SHOW as usize, w!("显示(&S)")) }.unwrap();
    unsafe { AppendMenuW(hmenu, MF_STRING, IDM_ABOUT as usize, w!("关于(&A)")) }.unwrap();
    unsafe { AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()) }.unwrap();
    unsafe { AppendMenuW(hmenu, MF_STRING, IDM_EXIT as usize, w!("退出(&X)")) }.unwrap();

    // 设置默认菜单项（加粗显示）
    unsafe { SetMenuDefaultItem(hmenu, IDM_SHOW, 0) }.unwrap();

    // 获取鼠标位置
    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt) }.unwrap();

    // 需要先将窗口设为前台窗口，菜单才能正常工作
    unsafe { SetForegroundWindow(hwnd) }.unwrap();

    // 显示菜单
    unsafe { TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, Some(0), hwnd, None) }.unwrap();

    // 点击菜单外区域时关闭菜单
    // unsafe { PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0)) }.unwrap();

    // 销毁菜单
    // unsafe { DestroyMenu(hmenu) }.unwrap();
}

fn show_device_picker(hwnd: HWND) {
    info!("显示设备选择器");
    let selector = AudioPlaybackConnection::GetDeviceSelector().unwrap();
    let device_picker = DevicePicker::new().unwrap();
    // 选择音频播放设备

    device_picker
        .Filter()
        .unwrap()
        .SupportedDeviceSelectors()
        .unwrap()
        .Append(&selector)
        .unwrap();

    let all_devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)
        .unwrap()
        .join()
        .unwrap();
    info!(
        "Found {} audio playback devices",
        all_devices.Size().unwrap()
    );
    for device in all_devices {
        device_picker
            .SetDisplayStatus(
                &device,
                &HSTRING::new(),
                DevicePickerDisplayStatusOptions::None,
            )
            .unwrap();
    }

    device_picker
        .DeviceSelected(
            &TypedEventHandler::<DevicePicker, DeviceSelectedEventArgs>::new({
                move |picker, args| {
                    let device_picker = picker.as_ref().unwrap();
                    let device_info = args.as_ref().unwrap().SelectedDevice().unwrap();

                    info!(
                        "Selected device: {} - {}",
                        device_info.Name().unwrap(),
                        device_info.Id().unwrap()
                    );

                    device_picker
                        .SetDisplayStatus(
                            &device_info,
                            &HSTRING::from("连接中"),
                            DevicePickerDisplayStatusOptions::ShowProgress
                                | DevicePickerDisplayStatusOptions::ShowDisconnectButton,
                        )
                        .unwrap();

                    let connection = AudioPlaybackConnection::TryCreateFromId(
                        &args
                            .as_ref()
                            .unwrap()
                            .SelectedDevice()
                            .unwrap()
                            .Id()
                            .unwrap(),
                    )
                    .unwrap();
                    connection
                        .StateChanged(&TypedEventHandler::<AudioPlaybackConnection, _>::new(
                            |sender, _| {
                                let _connection = sender.as_ref().unwrap();
                                let state = sender.as_ref().unwrap().State().unwrap();
                                match state {
                                    AudioPlaybackConnectionState::Opened => {
                                        info!("设备状态改变：连接已打开");
                                    }
                                    AudioPlaybackConnectionState::Closed => {
                                        info!("设备状态改变：连接已关闭");
                                        todo!();
                                    }
                                    _ => {
                                        info!("设备状态改变：其他状态: {:?}", state);
                                    }
                                }

                                Ok(())
                            },
                        ))
                        .unwrap();

                    connection.Start().unwrap();
                    let res = connection.Open().unwrap();
                    info!("Connection result: {:?}", res.Status().unwrap());

                    match res.Status().unwrap() {
                        AudioPlaybackConnectionOpenResultStatus::Success => {
                            DEVICES
                                .lock()
                                .unwrap()
                                .insert(device_info.Id().unwrap().to_string_lossy(), connection);
                            device_picker
                                .SetDisplayStatus(
                                    &device_info,
                                    &HSTRING::from("已连接"),
                                    DevicePickerDisplayStatusOptions::ShowDisconnectButton,
                                )
                                .unwrap();
                            info!("已连接到设备： {}", device_info.Name().unwrap());
                        }
                        AudioPlaybackConnectionOpenResultStatus::UnknownFailure => {
                            device_picker
                                .SetDisplayStatus(
                                    &device_info,
                                    &HSTRING::from("设备不可用"),
                                    DevicePickerDisplayStatusOptions::None,
                                )
                                .unwrap();
                            info!("设备不可用");
                        }
                        _ => {
                            device_picker
                                .SetDisplayStatus(
                                    &device_info,
                                    &HSTRING::from("连接失败"),
                                    DevicePickerDisplayStatusOptions::None,
                                )
                                .unwrap();
                            info!("连接失败, 状态: {:?}", res.Status().unwrap());
                        }
                    }

                    Ok(())
                }
            }),
        )
        .unwrap();

    device_picker
        .DisconnectButtonClicked(&TypedEventHandler::<
            DevicePicker,
            DeviceDisconnectButtonClickedEventArgs,
        >::new(|sender, args| {
            let device_picker = sender.as_ref().unwrap();
            let device_info = args.as_ref().unwrap().Device().unwrap();
            info!(
                "Disconnect button clicked for device: {} - {}",
                device_info.Name().unwrap(),
                device_info.Id().unwrap()
            );

            DEVICES
                .lock()
                .unwrap()
                .remove(&device_info.Id().unwrap().to_string_lossy())
                .and_then(|connection| {
                    connection.Close().unwrap();
                    device_picker
                        .SetDisplayStatus(
                            &device_info,
                            &HSTRING::default(),
                            DevicePickerDisplayStatusOptions::None,
                        )
                        .ok()
                });

            info!(
                "All Devices:{:?}",
                DEVICES.lock().unwrap().keys().collect::<Vec<_>>()
            );

            Ok(())
        }))
        .unwrap();

    device_picker
        .Appearance()
        .unwrap()
        .SetTitle(&HSTRING::from("选择音频源设备"))
        .unwrap();

    let notify_icon_id = NOTIFYICONIDENTIFIER {
        cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
        hWnd: hwnd,
        ..Default::default()
    };
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    info!("DPI: {}", dpi);

    let rect = unsafe { Shell_NotifyIconGetRect(&notify_icon_id) }.unwrap();

    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt) }.unwrap();

    let scale = dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32;
    let selection = Rect {
        X: rect.left as f32 / scale,
        Y: rect.top as f32 / scale,
        Width: (rect.right - rect.left) as f32 / scale,
        Height: (rect.bottom - rect.top) as f32 / scale,
        // X: pt.x as f32 / scale,
        // Y: pt.y as f32 / scale,
        // Width: 1.0,
        // Height: 1.0,
    };

    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt) }.unwrap();

    info!("Notify Icon Rect: {:?}", rect);
    info!("Selection rect: {:?}", selection);
    info!("Scale factor: {}", scale);
    info!("Cursor position: {:?}", pt);

    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            SWP_HIDEWINDOW,
        )
    }
    .unwrap();
    // let _ = unsafe { SetForegroundWindow(hwnd) };
    device_picker.Show(selection).unwrap();
}
