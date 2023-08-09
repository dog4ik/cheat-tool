import { useNavigate } from "@solidjs/router";
import { createSignal, onMount } from "solid-js";
import { invokeRust, ProcessListItem } from "../RustBridge";

function SelectProcessPage() {
  const [processId, setProcessId] = createSignal("");
  const [name, setName] = createSignal("");
  const [error, setError] = createSignal("");
  const [selectedProcess, setSelectedProcess] = createSignal<ProcessListItem>();
  const [processes, setProcesses] = createSignal<ProcessListItem[]>([]);
  const navigate = useNavigate();

  async function setProcess() {
    let selected = selectedProcess();
    if (selected)
      await invokeRust("select_process", { pid: selected.pid })
        .then((data) => {
          setProcessId(data);
          setError("");
        })
        .catch((err) => setError(err));
  }

  async function getProcessList(query: string) {
    await invokeRust("get_process_list", { query }).then((data) => {
      setProcesses(data);
    });
  }

  async function handleInput(input: string) {
    setName(input);
    getProcessList(input);
  }
  onMount(async () => {
    getProcessList("");
  });

  return (
    <div class="w-full bg-neutral-900 h-screen min-h-screen flex flex-col gap-5 text-white justify-center items-center">
      <div class="flex gap-4 items-center">
        <h1 class="text-4xl">Select process</h1>
        <button onClick={() => handleInput(name())} class="p-4">
          <img src="refresh.svg" />
        </button>
      </div>
      <ul class="overflow-y-scroll w-5/6 h-2/3">
        {processes().map((process) => (
          <li
            class={`grid grid-cols-2 cursor-pointer justify-between items-center py-3 px-5 ${
              process.pid === selectedProcess()?.pid
                ? "text-black bg-white"
                : "hover:bg-neutral-950"
            }`}
            onClick={() => setSelectedProcess(process)}
          >
            <span class="text-start">{process.name}</span>
            <span class="text-end">{process.pid}</span>
          </li>
        ))}
      </ul>

      <form
        class="flex flex-col gap-5"
        onSubmit={(e) => {
          e.preventDefault();
          setProcess();
          navigate("/process");
        }}
      >
        <input
          class="text-black py-1 px-3 rounded-md"
          id="greet-input"
          value={name()}
          onInput={(e) => {
            let val = e.currentTarget.value;
            handleInput(val);
          }}
          placeholder="Enter a name..."
        />
        <button class="px-2 py-1 bg-red-500 rounded-md" type="submit">
          Select
        </button>
      </form>
      {processId() && <span class="text-xl">{processId()}</span>}
      {error() && (
        <span class="text-xl p-1 rounded-md bg-red-500">{error()}</span>
      )}
    </div>
  );
}

export default SelectProcessPage;
