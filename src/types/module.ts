import type { ComponentType } from "react";
import type { LucideIcon } from "lucide-react";

export type ModuleId = 
 | "dashboard"
 | "storage"
 | "weather"
 | "music"
 | "notes"
 | "system";
 
 export interface ModuleDefinition {
    id: ModuleId;
    label: string;
    icon: LucideIcon;
    component: ComponentType;
 }