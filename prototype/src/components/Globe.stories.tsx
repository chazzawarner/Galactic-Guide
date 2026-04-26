import type { Meta, StoryObj } from "@storybook/react-vite";
import { Globe } from "./Globe";
import { issTrajectory } from "@/data/trajectory";
import { useSimClock } from "@/lib/simClock";
import { TimeControls } from "./TimeControls";

function Live() {
  const clock = useSimClock(Date.parse(issTrajectory.start_at));
  return (
    <div className="flex h-screen w-screen flex-col bg-[var(--color-bg)]">
      <div className="flex-1">
        <Globe trajectory={issTrajectory} simTimeMs={clock.simTimeMs} />
      </div>
      <div className="border-t border-[var(--color-border)] p-3">
        <TimeControls
          simTimeMs={clock.simTimeMs}
          speed={clock.speed}
          playing={clock.playing}
          onSpeedChange={clock.setSpeed}
          onTogglePlay={clock.toggle}
          onNow={clock.now}
        />
      </div>
    </div>
  );
}

const meta: Meta<typeof Globe> = {
  title: "Components/Globe",
  component: Globe,
  parameters: { layout: "fullscreen" },
};
export default meta;

type Story = StoryObj<typeof Globe>;

export const ISSOrbit: Story = {
  render: () => <Live />,
};
