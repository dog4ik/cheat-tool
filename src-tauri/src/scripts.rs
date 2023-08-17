use std::sync::Arc;

use crate::{emit_keypress::emit_keyboard_event, process::Variable, watch_keypress, Process};

pub async fn bhop(process: Process, size: usize, offset: usize) -> Result<(), String> {
    println!("started bhop");
    let (mut receiver, abort) = process
        .watch_value(
            &Variable {
                position: offset,
                size: size.try_into().unwrap(),
            },
            10,
        )
        .await
        .unwrap();
    let key_event_sender = emit_keyboard_event().await.expect("to work");
    let is_space_pressed = Arc::new(tokio::sync::Mutex::new(false));
    let is_on_ground = Arc::new(tokio::sync::Mutex::new(true));
    let key_event_sender_copy = key_event_sender.clone();
    let is_space_pressed_copy = is_space_pressed.clone();
    let is_on_ground_copy = is_on_ground.clone();

    // 65 is space
    let handle = tokio::spawn(async move {
        let mut key_press_receiver = watch_keypress::get_key_press(65).await;
        let mut is_pressed = false;
        while let Some(press) = key_press_receiver.recv().await {
            if !press && is_pressed {
                println!("spacebar released variable is: {}", press);
                let mut is_space_pressed = is_space_pressed_copy.lock().await;
                *is_space_pressed = press;
            }
            if press && !is_pressed {
                println!("spacebar pressed variable is: {}", press);
                let mut is_space_pressed = is_space_pressed_copy.lock().await;
                let is_on_ground = *is_on_ground_copy.lock().await;
                try_jump(press, is_on_ground, key_event_sender_copy.clone()).await;
                *is_space_pressed = press;
            }
            is_pressed = press;
        }
    });
    while let Some(val) = receiver.recv().await {
        if val == 256 {
            *is_on_ground.lock().await = false;
            // in the air
        };
        if val == 257 {
            // on the ground
            let is_space_pressed = *is_space_pressed.lock().await;
            if is_space_pressed {
                println!("just jumped");
                try_jump(is_space_pressed, true, key_event_sender.clone()).await;
            }
            *is_on_ground.lock().await = true;
        }
    }
    println!("aborting handles");
    abort.abort();
    handle.abort();
    Ok(())
}

async fn try_jump(space: bool, ground: bool, emitter: tokio::sync::mpsc::Sender<evdev::Key>) {
    if space && ground {
        emitter.send(evdev::Key::KEY_F1).await.unwrap();
    }
}
