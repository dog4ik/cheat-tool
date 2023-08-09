import { A } from "@solidjs/router";
import { JSX } from "solid-js/jsx-runtime";

type SideItemProps = {
  name: string;
  children: JSX.Element;
  href: string;
};
const SideItem = (props: SideItemProps) => {
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
};
const SideBar = () => {
  return (
    <div class="w-24 h-screen flex shrink-0 flex-col">
      <SideItem name="Search" href="search">
        <img class="w-10 h-24" src="search.svg" />
      </SideItem>
      <SideItem name="Process" href="process">
        <img class="w-10 h-24 fill-white stroke-white" src="cpu.svg" />
      </SideItem>
      <SideItem name="Variables" href="variables">
        <img class="w-10 h-24 fill-white stroke-white" src="eye.svg" />
      </SideItem>
      <SideItem name="Scripts" href="scripts">
        <img class="w-10 h-24" src="code.svg" />
      </SideItem>
      <SideItem name="Settings" href="settings">
        <img class="w-10 h-24" src="settings.svg" />
      </SideItem>
    </div>
  );
};

export default SideBar;
