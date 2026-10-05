import { AnimatedOutlet } from "./AnimatedOutlet";
import { Sidebar } from "./Sidebar";
import { Toaster } from "../Toaster";
import { TitleBar } from "./TitleBar";

export function AppShell() {
  return (
    <div className="shell">
      <TitleBar />
      <div className="shell__body">
        <Sidebar />
        <AnimatedOutlet />
      </div>
      <Toaster />
    </div>
  );
}
