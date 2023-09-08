import { Route, Routes, useNavigate } from "@solidjs/router";
import SideBar from "./components/SideBar";
import SettingsProvider from "./context/SettingsProvider";
import ProcessPage from "./pages/ProcessPage";
import ScriptsPage from "./pages/ScriptsPage";
import SelectProcessPage from "./pages/SelectProcessPage";
import SettingsPage from "./pages/SettingsPage";
import VariablesPage from "./pages/VariablesPage";
import CurrentProcessProvider from "./context/CurrentProcessProvider";

function NavigatorComponent() {
  const navigator = useNavigate();
  navigator("/search");
  return null;
}

function App() {
  return (
    <CurrentProcessProvider>
      <NavigatorComponent />
      <SettingsProvider>
        <main class="bg-neutral-950 flex text-white">
          <SideBar />
          <div class="pt-10 w-full">
            <Routes>
              <Route component={SelectProcessPage} path="/search" />
              <Route component={ProcessPage} path="/process" />
              <Route component={ScriptsPage} path="/scripts" />
              <Route component={SettingsPage} path="/settings" />
              <Route component={VariablesPage} path="/variables" />
            </Routes>
          </div>
        </main>
      </SettingsProvider>
    </CurrentProcessProvider>
  );
}

export default App;
