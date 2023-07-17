use std::{fmt::Display, io::IoSliceMut, thread};

use bytes::{Bytes, BytesMut};
use nix::{
    sys::uio::{process_vm_readv, process_vm_writev, RemoteIoVec},
    unistd::Pid,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{channel, Receiver},
    time,
};

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Heap,
    Stack,
    Misc,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Heap => write!(f, "HEAP"),
            Location::Stack => write!(f, "STACK"),
            Location::Misc => write!(f, "MISC"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueSize {
    U8,
    U16,
    U32,
}

impl TryFrom<usize> for ValueSize {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::U8),
            2 => Ok(Self::U16),
            4 => Ok(Self::U32),
            _ => Err("size not supported".into()),
        }
    }
}

impl From<ValueSize> for usize {
    fn from(value: ValueSize) -> Self {
        match value {
            ValueSize::U8 => 1,
            ValueSize::U16 => 2,
            ValueSize::U32 => 4,
        }
    }
}

pub trait SizedValue {}

impl SizedValue for ValueSize {}

#[derive(Debug, Clone)]
pub struct Variable<T: SizedValue> {
    pub position: usize,
    pub size: T,
}

#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub location: Location,
    pub start_adress: usize,
    pub end_adress: usize,
    pub data: Bytes,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub memory: Vec<MemoryChunk>,
    pub pid: Pid,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessError {
    NotFoundError,
    #[allow(dead_code)]
    UnknownError,
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::NotFoundError => write!(f, "Process was not found"),
            ProcessError::UnknownError => write!(f, "Unknown error occured"),
        }
    }
}

//Constructs process from its id
impl TryFrom<Pid> for Process {
    type Error = ProcessError;

    fn try_from(pid: Pid) -> Result<Self, Self::Error> {
        let memory = get_memory_chunks(pid).map_err(|_| ProcessError::NotFoundError)?;
        let name = get_name(pid)?;
        return Ok(Process { name, pid, memory });
    }
}

//Finds process by its name
impl Process {
    pub fn find_by_name(name: &str) -> Option<Vec<Self>> {
        let processes = find_processes(name);
        if processes.len() == 0 {
            return None;
        } else {
            return Some(
                processes
                    .into_iter()
                    .map(|item| Self::try_from(Pid::from_raw(item.pid)).unwrap())
                    .collect(),
            );
        }
    }

    pub fn refresh_memory(&mut self) -> Result<(), ProcessError> {
        self.memory = get_memory_chunks(self.pid)?;
        return Ok(());
    }

    pub fn write(&self, value: &Variable<ValueSize>, new_value: usize) -> Result<(), ProcessError> {
        let new_value_i32 = new_value as i32;
        let new_buffer = new_value_i32.to_le_bytes();

        let remote_iov = &[RemoteIoVec {
            len: match value.size {
                ValueSize::U8 => 1,
                ValueSize::U16 => 2,
                ValueSize::U32 => 4,
            },
            base: value.position,
        }];

        let iov = [std::io::IoSlice::new(&new_buffer)];

        process_vm_writev(self.pid, &iov, remote_iov).map_err(|_| ProcessError::UnknownError)?;
        Ok(())
    }

    pub async fn watch_range(
        &self,
        start: usize,
        end: usize,
        pull_ms: u64,
    ) -> Result<Receiver<Bytes>, ProcessError> {
        let remote_iov = [RemoteIoVec {
            len: end - start,
            base: start,
        }];
        let id = self.pid.clone();
        let (sender, reciever) = channel(10);
        tokio::spawn(async move {
            loop {
                let mut buffer = BytesMut::new();
                let mut iov = [IoSliceMut::new(&mut buffer)];
                process_vm_readv(id, &mut iov, &remote_iov).expect("Failed to read from memory");
                sender.send(buffer.into()).await.unwrap();
                thread::sleep(time::Duration::from_millis(pull_ms));
            }
        });
        return Ok(reciever);
    }

    pub async fn watch_value(
        &self,
        value: &Variable<ValueSize>,
        timeout: u64,
    ) -> Result<Receiver<u32>, ProcessError> {
        let remote_iov = [RemoteIoVec {
            len: match value.size {
                ValueSize::U8 => 1,
                ValueSize::U16 => 2,
                ValueSize::U32 => 4,
            },
            base: value.position,
        }];
        let id = self.pid.clone();
        let (sender, reciever) = channel(10);
        tokio::spawn(async move {
            let mut buffer = [0u8; 4];
            loop {
                let mut iov = [IoSliceMut::new(&mut buffer)];
                process_vm_readv(id, &mut iov, &remote_iov).expect("Failed to read from memory");
                let val = u32::from_le_bytes(buffer);
                sender.send(val).await.unwrap();

                thread::sleep(time::Duration::from_millis(timeout));
            }
        });
        return Ok(reciever);
    }

    pub fn get_value(&self, value: &Variable<ValueSize>) -> Result<u32, ProcessError> {
        let size_in_bytes = match value.size {
            ValueSize::U8 => 1,
            ValueSize::U16 => 2,
            ValueSize::U32 => 4,
        };
        let remote_iov = &[RemoteIoVec {
            len: size_in_bytes,
            base: value.position,
        }];
        let mut buffer = BytesMut::with_capacity(4);
        unsafe { buffer.set_len(4) }
        let mut iov = [IoSliceMut::new(&mut buffer)];
        process_vm_readv(self.pid, &mut iov, remote_iov).unwrap();
        let (chunk, _) = buffer.as_chunks::<4>();
        Ok(u32::from_le_bytes(chunk[0]))
    }
}

//helper functions

fn get_memory_chunks(pid: Pid) -> Result<Vec<MemoryChunk>, ProcessError> {
    let maps_path = format!("/proc/{}/maps", pid);
    let mut result = Vec::new();
    let maps_content =
        std::fs::read_to_string(&maps_path).map_err(|_| ProcessError::NotFoundError)?;
    let lines = maps_content.lines();
    for line in lines {
        let mut chunks = line.split(" ");
        let range = chunks.nth(0).expect("to exist");
        let (start, finish) = range.split_once("-").expect("to exist");
        let location = chunks.last().expect("to exist");
        if location != "[heap]" && location != "[stack]" {
            continue;
        }
        let start_adress = usize::from_str_radix(start, 16).expect("to be number");
        let end_adress = usize::from_str_radix(finish, 16).expect("to be number");
        println!(
            "found range {:x} - {:x} on {}",
            start_adress, end_adress, location
        );
        let data = get_raw_memory(pid, start_adress, end_adress).unwrap();
        if location == "[heap]" {
            result.push(MemoryChunk {
                location: Location::Heap,
                start_adress,
                end_adress,
                data,
            });
            continue;
        }
        if location == "[stack]" {
            result.push(MemoryChunk {
                location: Location::Stack,
                start_adress,
                end_adress,
                data,
            });
            continue;
        }
    }
    if result.len() > 0 {
        return Ok(result);
    } else {
        return Err(ProcessError::UnknownError);
    }
}

pub fn get_raw_memory(
    pid: Pid,
    start_adress: usize,
    end_adress: usize,
) -> Result<Bytes, ProcessError> {
    let range_len = end_adress - start_adress;
    let remote_iov: &mut [RemoteIoVec] = &mut [RemoteIoVec {
        len: range_len,
        base: start_adress,
    }];
    let mut stack_bytes = BytesMut::with_capacity(range_len);
    unsafe {
        stack_bytes.set_len(range_len);
    }
    let mut iov = [IoSliceMut::new(&mut stack_bytes)];
    process_vm_readv(pid, &mut iov, remote_iov).expect("Failed to read from memory");
    return Ok(stack_bytes.into());
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessListItem {
    pub name: String,
    pub pid: i32,
}

pub fn find_processes(query: &str) -> Vec<ProcessListItem> {
    let processes = std::fs::read_dir("/proc").expect("to exist");
    let mut result = Vec::new();
    for process in processes {
        if let Ok(process) = process && process
                .metadata()
                .map_or(false, |metadata| metadata.is_dir())
                == true
                && process
                    .file_name()
                    .to_string_lossy()
                    .chars()
                    .into_iter()
                    .all(|x| x.is_digit(10))
            {
                let status = std::fs::read_to_string(format!(
                    "/proc/{}/status",
                    process.file_name().to_string_lossy()
                ))
                .expect("to exist");
                let name = status
                    .lines()
                    .next()
                    .expect("first line to cantain name")
                    .split_whitespace()
                    .last()
                    .expect("last thing in row to be name without witespaces");
                if name.to_lowercase().contains(query.to_lowercase().trim()) {
                    let id = process
                              .file_name()
                              .to_string_lossy()
                              .parse()
                              .expect("to be numeric valid process id");
                    result.push(
                        ProcessListItem {
                            pid: id,
                            name: name.into(),
                        },
                    );
                }
        }
    }
    return result;
}

fn get_name(pid: Pid) -> Result<String, ProcessError> {
    let name = std::fs::read_to_string(format!("/proc/{}/status", pid))
        .map_err(|_| ProcessError::NotFoundError)?
        .lines()
        .next()
        .ok_or(ProcessError::NotFoundError)?
        .split_whitespace()
        .last()
        .ok_or(ProcessError::NotFoundError)?
        .to_string();
    return Ok(name);
}
