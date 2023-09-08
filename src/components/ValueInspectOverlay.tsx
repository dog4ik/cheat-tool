import { createSignal, onMount } from "solid-js";
import {
  emitRustEvent,
  invokeRust,
  MemoryChunk,
  useRustEvent,
} from "../RustBridge";

type Props = {
  onClose: () => void;
  variable: MemoryChunk;
};

type ValueProps = {
  offBy: number;
  variable: MemoryChunk;
  isActive: boolean;
  onClick: () => void;
};

const Value = (props: ValueProps) => {
  return (
    <div
      onClick={props.onClick}
      class={`flex group relative justify-center rounded-xl items-center p-3 ${
        props.isActive ? "bg-white text-black" : "bg-black text-white"
      }`}
    >
      <div class="group-hover:flex absolute hidden bg-neutral-500 bottom-0 right-0 p-2 translate-y-full z-10 translate-x-full justify-center items-center">
        <span>{props.offBy}</span>
      </div>
      <span>{props.variable.value}</span>
    </div>
  );
};

const ValueInspectOverlay = (props: Props) => {
  const [isClosing, setIsClosing] = createSignal(false);
  const [variable, setVariable] = createSignal(props.variable);
  const [neighbors, setNeighbors] = createSignal<MemoryChunk[]>([]);
  const [writeValue, setWriteValue] = createSignal<number>();
  async function handleClose() {
    await unlisten.then((fn) => fn());
    await emitRustEvent("unlisten_value", undefined);

    setIsClosing(true);
    setTimeout(() => {
      props.onClose();
    }, 200);
  }

  async function watchValue() {
    console.log("starting to watch value");
    let value = variable();
    await invokeRust("watch_value", {
      position: value.offset,
      size: value.size,
    });
  }

  async function switchValue(val: MemoryChunk) {
    await unlisten.then((fn) => fn());
    emitRustEvent("unlisten_value", undefined);
    setVariable(val);
    let values = await invokeRust("get_neighbors", {
      offset: variable().offset,
    });
    setNeighbors(values);
    watchValue();
    unlisten = useRustEvent("value_update", (event) => {
      console.log("got " + event.payload);
      setVariable({ ...variable(), value: event.payload });
    });
  }

  onMount(async () => {
    let values = await invokeRust("get_neighbors", {
      offset: variable().offset,
    });
    setNeighbors(values);
    await watchValue();
  });

  let unlisten = useRustEvent("value_update", (event) => {
    console.log("got " + event.payload);
    setVariable({ ...variable(), value: event.payload });
  });
  async function handleWrite() {
    let value = writeValue();
    if (value !== undefined) {
      await invokeRust("write_value", {
        value,
        variable: {
          size: props.variable.size,
          position: props.variable.offset,
        },
      });
    }
  }

  async function bhop() {
    await invokeRust("bhop", {
      size: variable().size,
      offset: variable().offset,
    });
  }

  return (
    <div
      onClick={handleClose}
      class={`fixed inset-0 z-40 w-screen h-screen flex justify-center items-center bg-black/80 ${
        isClosing() ? "opacity-0" : "animate-fade-in"
      } transition-opacity duration-200`}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        class="w-3/4 h-3/4 text-white bg-neutral-950 rounded-xl flex flex-col gap-4 items-center p-4"
      >
        <span class="text-5xl">{variable().value}</span>
        <span title={props.variable.offset.toString()} class="text-4xl">
          0x{variable().offset.toString(16)}
        </span>
        <button class="px-3 py-2 rounded-xl bg-green-500" onClick={watchValue}>
          Watch it
        </button>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleWrite();
          }}
        >
          <input
            onInput={(e) => setWriteValue(e.target.valueAsNumber)}
            placeholder="Value"
            class="px-2 rounded-xl text-black"
            type="number"
          />
          <button class="bg-green-500 px-3 py-2 rounded-xl" type="submit">
            Write {writeValue()} in value
          </button>
        </form>
        <button class="bg-red-500 p-2 rounded-xl" onClick={bhop}>
          BHOP
        </button>
        <div class="flex gap-2 flex-wrap items-center">
          {neighbors().map((item, idx) => (
            <Value
              onClick={() => {
                switchValue(item);
              }}
              isActive={item.offset === variable().offset}
              variable={item}
              offBy={(idx - 25) * 4}
            ></Value>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ValueInspectOverlay;
