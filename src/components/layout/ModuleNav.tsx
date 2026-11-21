import { ChevronLeft, ChevronRight } from "lucide-react";

import { useAppStore } from "../../stores/appStores";
import { modules } from "../../modules/registry";

export function ModuleNav() {
  const activeModule = useAppStore((s) => s.activeModule);
  const setActiveModule = useAppStore((s) => s.setActiveModule);

  const activeIndex = modules.findIndex((m) => m.id === activeModule);

  const navigate = (direction: number) => {
    const next = (activeIndex + direction + modules.length) % modules.length;
    setActiveModule(modules[next].id);
  };

  return (
    <section className="navigation-section">
      <button
        className="nav-arrow"
        onClick={() => navigate(-1)}
        title="Previous module"
      >
        <ChevronLeft size={17} />
      </button>

      <nav className="module-navigation">
        {modules.map((item) => {
          const Icon = item.icon;
          const isActive = activeModule === item.id;

          return (
            <button
              key={item.id}
              className={`module-button ${isActive ? "active" : ""}`}
              onClick={() => setActiveModule(item.id)}
              title={item.label}
              aria-label={item.label}
              aria-current={isActive ? "page" : undefined}
            >
              <Icon size={19} strokeWidth={1.8} />
              <span className="module-tooltip">{item.label}</span>
            </button>
          );
        })}
      </nav>

      <button
        className="nav-arrow"
        onClick={() => navigate(1)}
        title="Next module"
      >
        <ChevronRight size={17} />
      </button>
    </section>
  );
}