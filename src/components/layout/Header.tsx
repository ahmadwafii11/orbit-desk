import { Settings2 } from "lucide-react";

import { useAppStore } from "../../stores/appStores";
import { Icon } from "./Icon";

export function Header() {
    const setActiveModule = useAppStore((s) => s.setActiveModule);

    return(
       <>
            <header className="app-header">
                <div className="brand">
                    <Icon size={29}/>

                    <span>
                        Orbit<span className="brand-light">Desk</span>
                    </span>

                    <div className="header-right">
                        <span className="status-indicator"/>
                        <span className="header-status">Local Workspace</span>

                        <button
                            className="icon-button"
                            title="Settings"
                            onClick={() => setActiveModule("system")}
                        >
                            <Settings2 size={17}/>
                        </button>
                    </div>
                </div>
            </header>
       </>
    );
}