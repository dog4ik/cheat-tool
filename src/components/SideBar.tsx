import { A } from "@solidjs/router";
import { JSX } from "solid-js/jsx-runtime";
import { useCurrentProcess } from "../context/CurrentProcessProvider";
import { Process } from "../RustBridge";

type SideItemProps = {
  name: string;
  children: JSX.Element;
  href: string;
};

type ProcessDisplayProps = {
  process?: Process;
  isLive: boolean;
};

function ProcessDisplay(props: ProcessDisplayProps) {
  if (props.process?.name == undefined) {
    return <div>Process is not defined</div>;
  }
  return (
    <div class="flex justify-center items-center gap-4">
      <div
        class={`w-5 h-5 rounded-full ${
          props.isLive ? "bg-green-500" : "bg-red-500"
        }`}
      ></div>
      <div>
        {props.process.name} ({props.process.pid})
      </div>
    </div>
  );
}

function SideItem(props: SideItemProps) {
  return (
    <A
      href={props.href}
      class="flex flex-col items-center hover:bg-neutral-800 duration-200 justify-center"
      activeClass="bg-sky-500 hover:bg-sky-400"
    >
      {props.children}
      <span>{props.name}</span>
    </A>
  );
}
const SideBar = () => {
  const [processValues] = useCurrentProcess();
  return (
    <div class="w-24 h-screen flex shrink-0 justify-between flex-col">
      <div>
        <SideItem name="Search" href="/search">
          <img class="w-10 h-24" src="search.svg" />
        </SideItem>
        <SideItem name="Process" href="/process">
          <img class="w-10 h-24 fill-white stroke-white" src="cpu.svg" />
        </SideItem>
        <SideItem name="Variables" href="/variables">
          <img class="w-10 h-24 fill-white stroke-white" src="eye.svg" />
        </SideItem>
        <SideItem name="Scripts" href="/scripts">
          <img class="w-10 h-24" src="code.svg" />
        </SideItem>
        <SideItem name="Settings" href="/settings">
          <img class="w-10 h-24" src="settings.svg" />
        </SideItem>
      </div>
      <div class="mb-10 flex justify-center items-center">
        <div>{processValues.currentProcess?.pid}</div>
      </div>
    </div>
  );
};

export default SideBar;
