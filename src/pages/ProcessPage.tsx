import { createSignal, JSXElement, onMount } from "solid-js";
import { invokeRust } from "../types";

type CardProps = {
  cols: number;
  rows: number;
  children: JSXElement;
};
const Card = (props: CardProps) => {
  return (
    <div
      style={`grid-column: span ${props.cols} / span ${props.cols}; grid-row: span ${props.rows} / span ${props.cols}`}
      class="rounded-xl bg-white p-5"
    >
      {props.children}
    </div>
  );
};

const ProcessPage = () => {
  const [memory, setMemory] = createSignal<number[]>();
  onMount(async () => {
    let memory = await invokeRust("get_process_memory", { sizing: 4 });
    setMemory(memory.slice(0, 1000));
  });
  return (
    <div class="w-screen min-h-screen bg-neutral-900 grid grid-cols-5 grid-rows-5">
      <Card cols={1} rows={1}>
        <div>
          {memory()?.map((item) => (
            <div>{item}</div>
          ))}
        </div>
      </Card>
    </div>
  );
};

export default ProcessPage;
