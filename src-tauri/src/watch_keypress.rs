use std::{fmt::Display, process::Stdio};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::{channel, Receiver};

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    UnknownError,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownError => write!(f, "Unknown error occured"),
        }
    }
}

fn parse_code(line: &str) -> Result<bool, ParseError> {
    let code: u32 = line
        .split_whitespace()
        .nth(2)
        .ok_or(ParseError::UnknownError)?
        .trim()
        .parse()
        .map_err(|_| ParseError::UnknownError)?;

    if code == 2 {
        return Ok(true);
    } else if code == 3 {
        return Ok(false);
    } else {
        return Err(ParseError::UnknownError);
    }
}

pub async fn get_key_press(key_code: u32) -> Receiver<bool> {
    let (sender, receiver) = channel(100);
    tokio::spawn(async move {
        let mut is_pressed = false;
        let child = Command::new("xinput")
            .args(["test-xi2", "--root"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process");
        if let Some(stdout) = child.stdout {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Some(line) = lines.next_line().await.unwrap() {
                if let Some((key, value)) = line.split_once(':') {
                    if key.trim() == "detail" && value.trim().parse::<u32>().unwrap() == key_code {
                        sender.send(is_pressed).await.unwrap();
                    }
                } else if let Ok(val) = parse_code(line.trim()) {
                    is_pressed = val;
                } else {
                    continue;
                }
            }
        }
    });
    //we are not returning receiver
    println!("return");
    return receiver;
}
