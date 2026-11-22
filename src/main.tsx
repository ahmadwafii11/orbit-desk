import React from "react";
import ReactDOM from "react-dom/client";

import { detectWidgetModule } from "./lib/widget";
import "./index.css";

// Kode window utama dan kode widget dimuat terpisah (dynamic import). 
// Jadi window widget tidak ikut memuat seluruh app. 
// Ini menjaga agar pemakaian memori widget tetap kecil.
async function bootstrap() {
  const root = ReactDOM.createRoot(
    document.getElementById("root") as HTMLElement,
  );

  const widgetModule = detectWidgetModule();

  if (widgetModule) {
    document.documentElement.dataset.mode = "widget";
    const { WidgetApp } = await import("./modules/widgets/WidgetApp");

    root.render(
      <React.StrictMode>
        <WidgetApp moduleId={widgetModule} />
      </React.StrictMode>,
    );
    return;
  }

  const { default: App } = await import("./App");

  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

bootstrap();