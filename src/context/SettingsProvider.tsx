import { createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { JSX } from "solid-js/jsx-runtime";

type Metric = "h" | "d" | "b";
type Sizing = 1 | 2 | 4;
type Theme = "dark" | "light";

type SettingsState = {
  metric: Metric;
  sizing: Sizing;
  theme: Theme;
};
type SettingsFuctions = {
  setMetric: (metric: Metric) => void;
  setSizing: (sizing: Sizing) => void;
};

type SettingsContextValues = [SettingsState, SettingsFuctions];

export const SettingsContext = createContext<SettingsContextValues>([
  {} as SettingsState,
  {} as SettingsFuctions,
]);

type Props = {
  children: JSX.Element;
};

export function SettingsProvider(props: Props) {
  const [state, setState] = createStore<SettingsState>({
    metric: "h",
    sizing: 4,
    theme: "dark",
  } as SettingsState);

  function setMetric(metric: Metric) {
    setState("metric", metric);
  }

  function setSizing(sizing: Sizing) {
    setState("sizing", sizing);
  }

  const settings: SettingsContextValues = [
    state,
    {
      setMetric,
      setSizing,
    },
  ];

  return (
    <SettingsContext.Provider value={settings}>
      {props.children}
    </SettingsContext.Provider>
  );
}

export const useSettings = () => useContext(SettingsContext);

export default SettingsProvider;
