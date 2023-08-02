import { Route, Routes } from "@solidjs/router";
import SideBar from "./components/SideBar";
import SettingsProvider from "./context/SettingsProvider";
import ProcessPage from "./pages/ProcessPage";
import ScriptsPage from "./pages/ScriptsPage";
import SelectProcessPage from "./pages/SelectProcessPage";
import SettingsPage from "./pages/SettingsPage";
import VariablesPage from "./pages/VariablesPage";

function App() {
  return (
    <SettingsProvider>
      <main class="bg-neutral-950 flex text-white">
        <SideBar />
        <Routes>
          <Route component={SelectProcessPage} path="/search" />
          <Route component={ProcessPage} path="/process" />
          <Route component={ScriptsPage} path="/scripts" />
          <Route component={SettingsPage} path="/settings" />
          <Route component={VariablesPage} path="/variables" />
        </Routes>
      </main>
    </SettingsProvider>
  );
}

export default App;
