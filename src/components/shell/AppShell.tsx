import { AnimatedOutlet } from "./AnimatedOutlet";
import { useStageStore } from "../../stores/stage";
import { Sidebar } from "./Sidebar";
import { Stage } from "./Stage";
import { Toaster } from "../Toaster";
import { TitleBar } from "./TitleBar";

export function AppShell() {
  const stageOn = useStageStore((st) => st.image !== null);
  return (
    <div className={`shell${stageOn ? " has-stage" : ""}`}>
      <Stage />
      <TitleBar />
      <div className="shell__body">
        <Sidebar />
        <AnimatedOutlet />
      </div>
      <Toaster />
    </div>
  );
}
