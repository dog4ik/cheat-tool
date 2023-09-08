import { JSX, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";

import { Process } from "../RustBridge";

type CurrentProcessState = {
  currentProcess?: Process;
  isLive: boolean;
};

type CurrentProcessFunctions = {
  setProcess: (process: Process) => void;
  setIsLive: (isLive: boolean) => void;
};

type CurrentProcessContextValues = [
  CurrentProcessState,
  CurrentProcessFunctions
];

export const CurrentProcessContext = createContext<CurrentProcessContextValues>(
  [{} as CurrentProcessState, {} as CurrentProcessFunctions]
);

type Props = {
  children: JSX.Element;
};

export function CurrentProcessProvider(props: Props) {
  const [state, setState] = createStore<CurrentProcessState>({
    isLive: false,
  } as CurrentProcessState);

  function setProcess(process: Process) {
    setState("currentProcess", process);
  }

  function setIsLive(isLive: boolean) {
    setState("isLive", isLive);
  }

  const currentProcess: CurrentProcessContextValues = [
    state,
    {
      setIsLive,
      setProcess,
    },
  ];

  return (
    <CurrentProcessContext.Provider value={currentProcess}>
      {props.children}
    </CurrentProcessContext.Provider>
  );
}

export const useCurrentProcess = () => useContext(CurrentProcessContext);

export default CurrentProcessProvider;
