use std::{fmt::Display, io::IoSliceMut};

use bytes::{Bytes, BytesMut};
use nix::{
    sys::uio::{process_vm_readv, process_vm_writev, RemoteIoVec},
    unistd::Pid,
};
use serde::{de::Error, ser::SerializeStruct, Deserialize, Serialize};
use tokio::{
    sync::mpsc::{channel, Receiver},
    task::AbortHandle,
    time::{self, sleep},
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

impl<'de> Deserialize<'de> for ValueSize {
    fn deserialize<D>(deserializer: D) -> Result<ValueSize, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: u32 = u32::deserialize(deserializer)?;
        return match value {
            1 => Ok(ValueSize::U8),
            2 => Ok(ValueSize::U16),
            4 => Ok(ValueSize::U32),
            other => Err(Error::custom(format!(
                "Value should be 1 | 2 | 4 but got: {}",
                other
            ))),
        };
    }
}

impl Serialize for ValueSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let result: u32 = match self {
            ValueSize::U8 => 1,
            ValueSize::U16 => 2,
            ValueSize::U32 => 4,
        };
        return serializer.serialize_u32(result);
    }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Variable {
    pub position: usize,
    pub size: ValueSize,
}

#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub location: Location,
    pub start_adress: usize,
    pub end_adress: usize,
    pub data: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BufferValue {
    pub offset: usize,
    pub value: usize,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub data: Vec<BufferValue>,
    pub sizing: ValueSize,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub memory: Vec<MemoryChunk>,
    pub buffer: Option<Buffer>,
    pub pid: Pid,
    pub name: String,
}

impl Serialize for Process {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("process", 3)?;
        state.serialize_field("pid", &self.pid.as_raw())?;
        state.end()
    }
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
            ProcessError::NotFoundError => write!(f, "Process is not found"),
            ProcessError::UnknownError => write!(f, "Unknown error occured"),
        }
    }
}

/// Constructs process from its id
impl TryFrom<Pid> for Process {
    type Error = ProcessError;

    fn try_from(pid: Pid) -> Result<Self, Self::Error> {
        let memory = get_memory_chunks(pid).map_err(|_| ProcessError::NotFoundError)?;
        let name = get_name(pid)?;
        let buffer = None;
        return Ok(Process {
            name,
            pid,
            memory,
            buffer,
        });
    }
}

impl Process {
    /// Finds process by its name
    pub fn find_by_name(name: &str) -> Option<Vec<Self>> {
        let processes = find_processes(name);
        if processes.is_empty() {
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

    /// Resets memory state with latest state
    pub fn refresh_memory(&mut self) -> Result<(), ProcessError> {
        self.memory = get_memory_chunks(self.pid)?;
        return Ok(());
    }

    /// Writes variable in place of old one
    pub fn write(&self, value: &Variable, new_value: usize) -> Result<(), ProcessError> {
        write_value(self.pid, value, new_value)
    }

    pub fn watch_range(
        &self,
        start: usize,
        end: usize,
        poll_ms: u64,
    ) -> Result<(Receiver<Bytes>, AbortHandle), ProcessError> {
        let remote_iov = [RemoteIoVec {
            len: end - start,
            base: start,
        }];
        let (sender, reciever) = channel(10);
        let mut prev = BytesMut::new();
        let id = self.pid;
        let handle = tokio::spawn(async move {
            loop {
                let mut buffer = BytesMut::new();
                let mut iov = [IoSliceMut::new(&mut buffer)];
                process_vm_readv(id, &mut iov, &remote_iov).expect("Failed to read from memory");
                if buffer != prev {
                    prev = buffer.clone();
                    sender.send(buffer.into()).await.unwrap();
                }
                sleep(time::Duration::from_millis(poll_ms)).await;
            }
        });
        return Ok((reciever, handle.abort_handle()));
    }

    /// Watches vector of values
    pub async fn watch_values(
        &self,
        values: Vec<Variable>,
        timeout: u64,
    ) -> Result<(Receiver<Vec<u32>>, AbortHandle), ProcessError> {
        let id = self.pid;
        let (sender, reciever) = channel(10);
        let handle = tokio::spawn(async move {
            let mut buffer = [0u8; 4];
            let mut prev = Vec::new();
            loop {
                let mut result = Vec::new();
                for value in values.iter() {
                    //NOTE: maybe mutate it?
                    let remote_iov = [RemoteIoVec {
                        len: value.size.into(),
                        base: value.position,
                    }];
                    let mut iov = [IoSliceMut::new(&mut buffer)];
                    process_vm_readv(id, &mut iov, &remote_iov)
                        .expect("Failed to read from memory");
                    let val = u32::from_le_bytes(buffer);
                    result.push(val);
                }
                if prev != result {
                    prev = result.clone();
                    sender.send(result).await.unwrap();
                }
                sleep(time::Duration::from_millis(timeout)).await;
            }
        });
        return Ok((reciever, handle.abort_handle()));
    }

    /// Watches single value
    pub async fn watch_value(
        &self,
        value: &Variable,
        timeout: u64,
    ) -> Result<(Receiver<u32>, AbortHandle), ProcessError> {
        let remote_iov = [RemoteIoVec {
            len: value.size.into(),
            base: value.position,
        }];
        let id = self.pid;
        let (sender, reciever) = channel(10);
        let handle = tokio::spawn(async move {
            let mut buffer = [0u8; 4];
            let mut prev = 0;
            loop {
                let mut iov = [IoSliceMut::new(&mut buffer)];
                process_vm_readv(id, &mut iov, &remote_iov).expect("Failed to read from memory");
                //TODO: Figure out how to handle situations we dont know type of desired value
                let val = u32::from_le_bytes(buffer);
                if val != prev {
                    sender.send(val).await.unwrap();
                    prev = val;
                }

                sleep(time::Duration::from_millis(timeout)).await;
            }
        });
        return Ok((reciever, handle.abort_handle()));
    }

    /// Populates buffer from memory with provided values
    pub fn populate_buffer_with_value(
        &mut self,
        value: usize,
        sizing: usize,
    ) -> Result<Vec<BufferValue>, String> {
        let sizing: ValueSize = sizing.try_into().unwrap();
        self.refresh_memory().expect("to sucessfuly refresh");
        let mut result = Vec::new();
        for location in self.memory.iter() {
            for (offset, bytes) in location.data.windows(sizing.into()).enumerate() {
                if let Ok(sized_bytes) = bytes.try_into() {
                    let v = u32::from_le_bytes(sized_bytes) as usize;
                    if v == value {
                        let chunk = BufferValue {
                            value: v,
                            offset: offset + location.start_adress,
                        };
                        result.push(chunk);
                    }
                }
            }
        }
        self.buffer = Some(Buffer {
            data: result.clone(),
            sizing,
        });
        return Ok(result.into_iter().take(100).collect());
    }

    /// Populates buffer with process memory with given sizing
    pub fn populate_buffer(&mut self, sizing: usize) -> Result<(), ProcessError> {
        let sizing: ValueSize = sizing.try_into().unwrap();
        self.refresh_memory().expect("to sucessfuly refresh");
        let mut result = Vec::new();
        for location in self.memory.iter() {
            for (offset, bytes) in location.data.windows(sizing.into()).enumerate() {
                if let Ok(sized_bytes) = bytes.try_into() {
                    let v = u32::from_le_bytes(sized_bytes) as usize;
                    let value = BufferValue {
                        value: v,
                        offset: offset + location.start_adress,
                    };
                    result.push(value);
                }
            }
        }
        self.buffer = Some(Buffer {
            data: result,
            sizing,
        });
        Ok(())
    }

    /// Filters buffer comparing it with process memory
    pub fn expect_change(&mut self, is_changed: bool) -> Result<Vec<BufferValue>, ProcessError> {
        self.update_buffer().expect("to update");
        if let Some(buffer) = &mut self.buffer {
            buffer.data.retain(|x| match is_changed {
                true => {
                    get_value(
                        self.pid,
                        &Variable {
                            size: buffer.sizing,
                            position: x.offset,
                        },
                    )
                    .unwrap() as usize
                        != x.value
                }
                false => {
                    get_value(
                        self.pid,
                        &Variable {
                            size: buffer.sizing,
                            position: x.offset,
                        },
                    )
                    .unwrap() as usize
                        == x.value
                }
            });
            return Ok(buffer.data.to_owned());
        } else {
            return Err(ProcessError::UnknownError);
        }
    }

    /// Filters buffer making sure all values are equal provided value
    pub fn scan_next(&mut self, value: usize) -> Vec<BufferValue> {
        let _ = self.update_buffer();
        if let Some(buffer) = &mut self.buffer {
            buffer.data.retain(|x| {
                get_value(
                    self.pid,
                    &Variable {
                        size: buffer.sizing,
                        position: x.offset,
                    },
                )
                .unwrap() as usize
                    == value
            });
            return buffer.data.clone().into_iter().take(100).collect();
        } else {
            return Vec::with_capacity(0);
        }
    }

    /// Updates values on buffer
    pub fn update_buffer(&mut self) -> Result<(), ProcessError> {
        if let Some(buffer) = &mut self.buffer {
            for item in &mut buffer.data {
                item.value = get_value(
                    self.pid,
                    &Variable {
                        position: item.offset,
                        size: buffer.sizing,
                    },
                )? as usize;
            }
            return Ok(());
        } else {
            return Err(ProcessError::UnknownError);
        }
    }

    pub fn get_value(&self, value: &Variable) -> Result<u32, ProcessError> {
        return get_value(self.pid, value);
    }
}

//helper functions

/// Writes variable in place of old one
fn write_value(pid: Pid, value: &Variable, new_value: usize) -> Result<(), ProcessError> {
    let new_value_i32 = new_value as i32;
    let new_buffer = new_value_i32.to_le_bytes();

    let remote_iov = &[RemoteIoVec {
        len: value.size.into(),
        base: value.position,
    }];

    let iov = [std::io::IoSlice::new(&new_buffer)];

    process_vm_writev(pid, &iov, remote_iov).map_err(|_| ProcessError::UnknownError)?;
    Ok(())
}

/// Gets single value
fn get_value(pid: Pid, value: &Variable) -> Result<u32, ProcessError> {
    let remote_iov = &[RemoteIoVec {
        len: value.size.into(),
        base: value.position,
    }];
    let mut buffer = BytesMut::with_capacity(4);
    // SAFETY length is declared             ^ here
    unsafe { buffer.set_len(4) }
    let mut iov = [IoSliceMut::new(&mut buffer)];
    process_vm_readv(pid, &mut iov, remote_iov).unwrap();
    let (chunk, _) = buffer.as_chunks::<4>();
    Ok(u32::from_le_bytes(chunk[0]))
}

fn get_memory_chunks(pid: Pid) -> Result<Vec<MemoryChunk>, ProcessError> {
    let maps_path = format!("/proc/{}/maps", pid);
    let mut result = Vec::new();
    let maps_content =
        std::fs::read_to_string(&maps_path).map_err(|_| ProcessError::NotFoundError)?;
    let lines = maps_content.lines();
    for line in lines {
        let mut chunks = line.split(' ');
        let range = chunks.next().expect("to exist");
        let (start, finish) = range.split_once('-').expect("to exist");
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
    if !result.is_empty() {
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
    //SAFETY: We are setting SAME length and reading it in remove_iov
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
                && process
                    .file_name()
                    .to_string_lossy()
                    .chars()
                    .all(|x| x.is_ascii_digit())
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
