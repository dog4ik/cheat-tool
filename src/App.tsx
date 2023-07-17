import { createSignal, onCleanup } from "solid-js";
import ProcessPage from "./pages/ProcessPage";
import SelectProcessPage from "./pages/SelectProcessPage";

const routes = {
  "/": SelectProcessPage,
  "/process": ProcessPage,
};
function App() {
  const [route, setRoute] = createSignal(window.location.pathname);

  const handleRouteChange = () => {
    setRoute(window.location.pathname);
  };

  window.addEventListener("popstate", handleRouteChange);

  onCleanup(() => {
    window.removeEventListener("popstate", handleRouteChange);
  });

  return (
    <>
      {Object.entries(routes).map(([path, Component]) => {
        if (route() === path) {
          return <Component />;
        }
      })}
    </>
  );
}

export default App;
