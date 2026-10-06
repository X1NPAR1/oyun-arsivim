import { AnimatedOutlet } from "./AnimatedOutlet";
import { useStageStore } from "../../stores/stage";
import { Sidebar } from "./Sidebar";
import { Stage } from "./Stage";
import { Toaster } from "../Toaster";
import { TitleBar } from "./TitleBar";
import { GameConflictDialog } from "../GameConflictDialog";
import { useRunningSync } from "../../hooks/useRunning";

export function AppShell() {
  useRunningSync();
  const stageOn = useStageStore((st) => st.image !== null);
  return (
    <div className={`shell${stageOn ? " has-stage" : ""}`}>
      <Stage />
      <TitleBar />
      <div className="shell__body">
        <Sidebar />
        <AnimatedOutlet />
      </div>
      <GameConflictDialog />
      <Toaster />
    </div>
  );
}
