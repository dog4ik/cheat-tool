import { JSX, createSignal, onMount } from "solid-js";
import { DbVariable, invokeRust } from "../RustBridge";

type RawProps = {
  isEven: boolean;
  processId: number;
  name: string;
  description?: string;
  offset: number;
  id: number;
  onRemove: (id: number) => void;
};

type TableProps = {
  children: JSX.Element[];
};

function Table(props: TableProps) {
  return (
    <div class="grid flex-1 grid-cols-4">
      <div class="text-center p-2">Name</div>
      <div class="text-center p-2">Adress</div>
      <div class="text-center p-2">Pid</div>
      <div class="text-center p-2">Remove</div>
      {props.children}
    </div>
  );
}

function Row(props: RawProps) {
  return (
    <>
      <div class={`p-2 ${props.isEven ? "bg-neutral-950" : "bg-neutral-800"}`}>
        <span class="truncate">{props.name}</span>
      </div>
      <div
        class={`p-2 text-center ${
          props.isEven ? "bg-neutral-950" : "bg-neutral-800"
        }`}
      >
        <span>0x{props.offset.toString(16)}</span>
      </div>
      <div
        class={`p-2 text-center ${
          props.isEven ? "bg-neutral-950" : "bg-neutral-800"
        }`}
      >
        <span>{props.processId}</span>
      </div>
      <div
        class={`p-2 flex justify-center ${
          props.isEven ? "bg-neutral-950" : "bg-neutral-800"
        }`}
      >
        <button class="aspect-square" onClick={() => props.onRemove(props.id)}>
          <img src="/x.svg" alt="Close icon" />
        </button>
      </div>
    </>
  );
}
const VariablesPage = () => {
  const [variables, setVariables] = createSignal<DbVariable[]>([]);

  async function handleRemove(id: number) {
    await invokeRust("delete_variable_by_id", { id });
  }

  onMount(async () => {
    const dbVars = await invokeRust("get_variables", {});
    setVariables(dbVars);
  });
  return (
    <Table>
      {variables().map((v, idx) => (
        <Row
          id={v.id}
          isEven={(idx & 1) === 0}
          processId={v.process_id}
          name={v.name}
          offset={v.offset}
          description={v.description}
          onRemove={handleRemove}
        />
      ))}
    </Table>
  );
};

export default VariablesPage;
