import type { Meta, StoryObj } from "@storybook/react-vite";
import { TimeControls } from "./TimeControls";
import { useSimClock } from "@/lib/simClock";

function Live() {
  const clock = useSimClock();
  return (
    <div className="w-[640px] p-6">
      <TimeControls
        simTimeMs={clock.simTimeMs}
        speed={clock.speed}
        playing={clock.playing}
        onSpeedChange={clock.setSpeed}
        onTogglePlay={clock.toggle}
        onNow={clock.now}
      />
      <p className="mt-3 font-mono text-xs text-[var(--color-muted)]">
        simTime advances at speed × realDt; pause/play and speed swap re-anchor cleanly.
      </p>
    </div>
  );
}

const meta: Meta<typeof TimeControls> = {
  title: "Components/TimeControls",
  component: TimeControls,
  parameters: { layout: "fullscreen" },
};
export default meta;

type Story = StoryObj<typeof TimeControls>;

export const Default: Story = {
  render: () => <Live />,
};

export const Static1x: Story = {
  args: {
    simTimeMs: Date.UTC(2024, 0, 15, 12, 0, 0),
    speed: 1,
    playing: false,
    onSpeedChange: () => {},
    onTogglePlay: () => {},
    onNow: () => {},
  },
};

export const Static1000x: Story = {
  args: {
    simTimeMs: Date.UTC(2024, 0, 15, 12, 0, 0),
    speed: 1000,
    playing: true,
    onSpeedChange: () => {},
    onTogglePlay: () => {},
    onNow: () => {},
  },
};
