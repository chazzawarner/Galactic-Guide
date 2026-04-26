import { Dashboard } from "@/components/Dashboard";
import { issTrajectory } from "@/data/trajectory";
import { satellites } from "@/data/satellites";

export default function App() {
  return (
    <div className="h-full w-full">
      <Dashboard
        satellites={satellites}
        selected={satellites[0]}
        trajectory={issTrajectory}
        layout="sidebar"
      />
    </div>
  );
}
