这个非常有用，问题解决了，谢谢！

```rust
    let device_picker = DevicePicker::new()?;
    device_picker.DeviceSelected(&TypedEventHandler::<_, DeviceSelectedEventArgs>::new(move |sender, args| {
        let even_args = args.as_ref().unwrap();
        Ok(())
    }))?;
```

这是我初始的代码，不知道为什么编译器无法完成此处的类型推断，编译器只有报错，没有给任何建议，后来参考了#1212，才改成了上面的代码

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