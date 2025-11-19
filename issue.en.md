This is very helpful, the issue is resolved. Thank you!

```rust
    let device_picker = DevicePicker::new()?;
    device_picker.DeviceSelected(&TypedEventHandler::<_, DeviceSelectedEventArgs>::new(move |sender, args| {
        let even_args = args.as_ref().unwrap();
        Ok(())
    }))?;
```

This is my initial code. I'm not sure why the compiler couldn't complete the type inference here. The compiler only reported an error without providing any suggestions. Later, after referring to #1212, I modified it to the code above.

```rust
    let device_picker = DevicePicker::new()?;
    device_picker.DeviceSelected(&TypedEventHandler::new(move |sender, args| {
        let event_args = args.as_ref().unwrap();
        Ok(())
    }))?;
```

console output:
```log
error[E0282]: type annotations needed
 --> src\bin\device_picker.rs:9:30
  |
9 |         let event_args = args.as_ref().unwrap();
  |                              ^^^^^^ cannot infer type

For more information about this error, try `rustc --explain E0282`.
```