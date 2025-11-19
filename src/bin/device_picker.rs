use windows::{Devices::Enumeration::{DevicePicker, DeviceSelectedEventArgs}, Foundation::TypedEventHandler};


fn main() -> anyhow::Result<()> {
    let device_picker = DevicePicker::new()?;
    device_picker.DeviceSelected(&TypedEventHandler::<_, DeviceSelectedEventArgs>::new(move |sender, args| {
        let even_args = args.as_ref().unwrap();
        Ok(())
    }))?;

    todo!()
}