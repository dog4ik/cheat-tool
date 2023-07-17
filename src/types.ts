import { invoke } from "@tauri-apps/api/tauri";

type First<T extends readonly [any, any]> = T extends readonly [
  infer B,
  infer _
]
  ? B
  : never;

type Second<T extends readonly [any, any]> = T extends readonly [
  infer _,
  infer B
]
  ? B
  : never;

export type ProcessListItem = {
  pid: number;
  name: string;
};

export type RustFuctions = {
  select_process: (pid: { pid: number }) => string;
  get_process_list: (query: { query: string }) => ProcessListItem[];
  get_process_memory: (sizing: { sizing: 1 | 2 | 4 }) => number[];
  get_all_possible_memory: (sizing: { sizing: 1 | 2 | 4 }) => number[];
  refresh_memory: () => void;
  set_desired_value: (value: { value: number }) => void;
};

export async function invokeRust<T extends keyof RustFuctions>(
  cmd: T,
  invokeArgs: Parameters<RustFuctions[T]>[0]
): Promise<ReturnType<RustFuctions[T]>> {
  return invoke<ReturnType<RustFuctions[T]>>(cmd, invokeArgs);
}
