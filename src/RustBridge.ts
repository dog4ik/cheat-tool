import { invoke } from "@tauri-apps/api/tauri";
import { emit, EventCallback, listen, UnlistenFn } from "@tauri-apps/api/event";

export type ProcessListItem = {
  pid: number;
  name: string;
};

export type MemoryChunk = {
  size: ValueSize;
  value: number;
  offset: number;
};

export type RustSettings = {
  value_size: ValueSize;
};

export type ValueSize = 1 | 2 | 4;

export type Value = {
  position: number;
  size: ValueSize;
};

export type RustFunctions = {
  select_process: (args: { pid: number }) => string;
  get_process_list: (args: { query: string }) => ProcessListItem[];
  get_process_memory: (args: {
    sizing: ValueSize;
    value: number;
  }) => MemoryChunk[];
  refresh_memory: () => void;
  set_desired_value: (args: { value: number }) => void;
  get_value_from_memory: (args: {
    value: number;
    sizing: number;
  }) => MemoryChunk[];
  expect_change: () => MemoryChunk[];
  expect_no_change: () => MemoryChunk[];
  reset_values: () => void;
  reset_state: () => void;
  scan_next: (args: { value: number }) => MemoryChunk[];
  watch_value: (args: { variable: Value }) => number;
  watch_values: (args: { variables: Value[] }) => number;
  write_value: (args: { variable: Value; value: number }) => void;
  get_neighbors: (args: { offset: number }) => MemoryChunk[];
  change_settings: (args: { settings: RustSettings }) => void;
  press_space: () => void;
  bhop: (args: { size: number; offset: number }) => void;
};

export type RustEvents = {
  value_update: number;
  values_update: number[];
};

export type EmitEvents = {
  unlisten_value: undefined;
  unlisten_values: undefined;
};

export async function useRustEvent<T extends keyof RustEvents>(
  eventName: T,
  cb: EventCallback<RustEvents[T]>
): Promise<UnlistenFn> {
  const unlisten = await listen(eventName, cb);
  return unlisten;
}

export async function emitRustEvent<T extends keyof EmitEvents>(
  eventName: T,
  payload: EmitEvents[T]
) {
  await emit(eventName, payload);
}

export async function invokeRust<T extends keyof RustFunctions>(
  cmd: T,
  invokeArgs: Parameters<RustFunctions[T]>[0]
): Promise<ReturnType<RustFunctions[T]>> {
  return invoke<ReturnType<RustFunctions[T]>>(cmd, invokeArgs);
}
