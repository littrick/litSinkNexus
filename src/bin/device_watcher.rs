use windows::{core::*, Devices::Enumeration::*, Foundation::*};

fn main() -> Result<()> {
    let watcher = DeviceInformation::CreateWatcher()?;

    watcher.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(
        |_, info| {
            println!("{:?}", info.as_ref().expect("info").Name()?);
            Ok(())
        },
    ))?;

    watcher.EnumerationCompleted(&TypedEventHandler::new(|_, _| {
        println!("done!");
        Ok(())
    }))?;

    watcher.Start()?;
    std::thread::sleep(std::time::Duration::new(10, 0));


    let picker = DevicePicker::new()?;
    picker.DeviceSelected(&TypedEventHandler::<DevicePicker, DeviceSelectedEventArgs>::new(
        |sender, args|  {
            let device_name = args.as_ref().unwrap().SelectedDevice().unwrap().Name().unwrap();
            println!("Selected device: {}", device_name);
            Ok(())
        }
    ))?;
    Ok(())
}