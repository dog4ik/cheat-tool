import { A } from "@solidjs/router";
import { createSignal } from "solid-js";
import ValueInspectOverlay from "../components/ValueInspectOverlay";
import { invokeRust, MemoryChunk } from "../RustBridge";

type ValueProps = {
  offset: number;
  value: number;
  onClick: () => void;
};
const Value = (props: ValueProps) => {
  return (
    <div
      onClick={props.onClick}
      class="rounded-xl relative cursor-pointer border bg-white text-black flex group justify-center items-center p-5"
    >
      <div class="absolute z-10 group-hover:block pointer-events-none bg-neutral-400 p-2 hidden top-1/2 left-1/2 rounded-xl">
        0x{props.offset.toString(16)}
      </div>
      {props.value}
    </div>
  );
};

const ProcessPage = () => {
  const [memory, setMemory] = createSignal<MemoryChunk[]>();
  const [desiredValue, setDesiredValue] = createSignal<number>();
  const [isOverlayOpen, setIsOverlayOpen] = createSignal(false);
  const [selectedValue, setSelectedValue] = createSignal<MemoryChunk>();

  async function getMemory(value: number) {
    let memory = await invokeRust("populate_buffer_with_value", {
      sizing: 4,
      value,
    });
    setMemory(memory);
  }

  async function scanNext(value: number) {
    let memory = await invokeRust("scan_next", {
      value,
    });
    setMemory(memory);
  }

  async function expectNoChange() {
    let value = desiredValue();
    if (!value) return;
    let memory = await invokeRust("expect_change", { is_changed: false });
    setMemory(memory);
  }

  async function expectChange() {
    let value = desiredValue();
    if (!value) return;
    let memory = await invokeRust("expect_change", { is_changed: true });
    setMemory(memory);
  }

  return (
    <>
      {isOverlayOpen() && (
        <ValueInspectOverlay
          variable={selectedValue()!}
          onClose={() => setIsOverlayOpen(false)}
        />
      )}
      <div class="w-full relative min-h-screen bg-neutral-900 flex flex-col">
        <A href="/" class="absolute top-0 left-0">
          Back
        </A>
        <div class="flex flex-col gap-5 items-center bg-white rounded-xl p-5">
          <input
            class="bg-neutral-700 py-3 px-2 rounded-xl text-white"
            placeholder="input value"
            value={desiredValue()}
            onInput={(e) => setDesiredValue(e.target.valueAsNumber)}
            type="number"
          />
          <div class="flex gap-5 ">
            <button
              onClick={expectNoChange}
              class="bg-neutral-900 text-white p-4 rounded-xl"
            >
              Expect no change
            </button>
            <div>
              <button
                class="bg-green-500 rounded-xl p-2"
                onClick={() => {
                  let val = desiredValue();
                  if (val !== undefined) {
                    scanNext(val);
                  }
                }}
              >
                Scan next
              </button>
              <button
                class="bg-green-500 rounded-xl p-2"
                onClick={() => {
                  let val = desiredValue();
                  if (val !== undefined) {
                    getMemory(val);
                  }
                }}
              >
                Look for value
              </button>
            </div>
            <button
              onClick={expectChange}
              class="bg-neutral-900 text-white p-4 rounded-xl"
            >
              Expect change
            </button>
          </div>
        </div>
        <div class="flex flex-col gap-5">
          <div class="flex gap-3 flex-wrap">
            {memory()?.map((chunk) => {
              return (
                <Value
                  onClick={() => {
                    setSelectedValue(chunk);
                    setIsOverlayOpen(true);
                  }}
                  value={chunk.value}
                  offset={chunk.offset}
                />
              );
            })}
          </div>
        </div>
      </div>
    </>
  );
};

export default ProcessPage;
