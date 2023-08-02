use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, Key};
use std::time::Duration;
use tokio::sync::mpsc::channel;

pub async fn emit_keyboard_event() -> Result<tokio::sync::mpsc::Sender<evdev::Key>, std::io::Error>
{
    let (sender, mut receiver) = channel::<Key>(100);
    let mut keys = AttributeSet::<evdev::Key>::new();
    keys.insert(evdev::Key::KEY_SPACE);
    keys.insert(evdev::Key::KEY_F1);

    let mut device = VirtualDeviceBuilder::new()?
        .name("Fake Keyboard")
        .with_keys(&keys)?
        .build()
        .unwrap();

    for path in device.enumerate_dev_nodes_blocking()? {
        let path = path?;
        println!("Available as {}", path.display());
    }

    // Note this will ACTUALLY PRESS the button on your computer.
    // Hopefully you don't have BTN_DPAD_UP bound to anything important.
    tokio::spawn(async move {
        while let Some(code) = receiver.recv().await {
            // this guarantees a key event
            let down_event = InputEvent::new(EventType::KEY, code.code(), 1);
            device.emit(&[down_event]).unwrap();
            println!("Pressed.");
            tokio::time::sleep(Duration::from_millis(10)).await;

            // alternativeley we can create a InputEvent, which will be any variant of InputEvent
            // depending on the type_ value
            let up_event = InputEvent::new(EventType::KEY, code.code(), 0);
            device.emit(&[up_event]).unwrap();
            println!("Released.");
        }
    });
    return Ok(sender);
}
