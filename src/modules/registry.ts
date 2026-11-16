import {
  Activity,
  CloudSun,
  Folder,
  LayoutDashboard,
  Music2,
  NotebookPen,
} from "lucide-react";

import { Dashboard } from "./dashboard/Dashboard";
import { Storage } from "./storage/Storage";
import { Weather } from "./weather/Weather";
import { Music } from "./music/Music";
import { Notes } from "./notes/Notes";
import { System } from "./system/System";
import { ModuleDefinition, ModuleId } from "../types";

// Untuk menambah modul baru (News, Football, dst.):
// 1. tambahkan id di ModuleId (types.ts)
// 2. buat folder + komponen di src/modules/<nama>/
// 3. tambahkan satu entri di array ini
export const modules: ModuleDefinition[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, component: Dashboard },
  { id: "storage", label: "Storage", icon: Folder, component: Storage },
  { id: "weather", label: "Weather", icon: CloudSun, component: Weather },
  { id: "music", label: "Music", icon: Music2, component: Music },
  { id: "notes", label: "Notes", icon: NotebookPen, component: Notes },
  { id: "system", label: "System", icon: Activity, component: System },
];

export const moduleMap = Object.fromEntries(
  modules.map((m) => [m.id, m]),
) as Record<ModuleId, ModuleDefinition>;