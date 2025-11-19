use windows::{
    core::*, Devices::Enumeration::DevicePicker, Foundation::Rect, Media::Audio::AudioPlaybackConnection, Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        System::LibraryLoader::GetModuleHandleA,
        UI::{Shell::IInitializeWithWindow, WindowsAndMessaging::*},
    }
};

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let window_class = w!("pickerwindow");

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        println!("atom = {atom}");
        debug_assert!(atom != 0);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class,
            w!("Fuck window"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let mut message = MSG::default();

        while GetMessageA(&mut message, None, 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                println!("WM_CREATE");
                let device_picker = DevicePicker::new().unwrap();
                let initial: IInitializeWithWindow = device_picker.cast().unwrap();
                initial.Initialize(window).unwrap();

                let selector = AudioPlaybackConnection::GetDeviceSelector().unwrap();
                device_picker
                    .Filter()
                    .unwrap()
                    .SupportedDeviceSelectors()
                    .unwrap()
                    .Append(&selector)
                    .unwrap();

                println!("Picker show");
                device_picker.Show(Rect::default()).unwrap();

                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
