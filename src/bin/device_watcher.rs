use windows::{Devices::Enumeration::*, Foundation::*, Media::Audio::*, core::*};

fn main() -> Result<()> {
    let filter = AudioPlaybackConnection::GetDeviceSelector().unwrap();

    let watcher = DeviceInformation::CreateWatcherAqsFilter(&filter).unwrap();

    watcher.Added(&TypedEventHandler::<_, DeviceInformation>::new(
        |_, info| {
            println!("{:?}", info.as_ref().expect("info").Name()?);
            Ok(())
        },
    ))?;

    watcher.Updated(&TypedEventHandler::<_, DeviceInformationUpdate>::new(|_, args| {
        println!("updated: {:?}", args.as_ref().unwrap().Id().unwrap());
        Ok(())
    }))?;

    watcher.EnumerationCompleted(&TypedEventHandler::new(|_, _| {
        println!("done!");
        Ok(())
    }))?;

    watcher.Start()?;
    loop {
        std::thread::sleep(std::time::Duration::new(1, 0));
    }
    // std::thread::sleep(std::time::Duration::new(10, 0));
    // Ok(())
}
