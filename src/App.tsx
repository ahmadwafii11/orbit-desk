import { Header } from "./components/layout/Header";
import { ModuleNav } from "./components/layout/ModuleNav";
import { Footer } from "./components/layout/Footer";
import { moduleMap } from "./modules/registry";
import { useAppStore } from "./stores/appStores";

export default function App() {
  const activeModule = useAppStore((s) => s.activeModule);
  const ActiveComponent = moduleMap[activeModule].component;

  return (
    <main className="orbit-app">
      <Header />
      <ModuleNav />

      <section className="main-content" key={activeModule}>
        <ActiveComponent />
      </section>

      <Footer />
    </main>
  );
}