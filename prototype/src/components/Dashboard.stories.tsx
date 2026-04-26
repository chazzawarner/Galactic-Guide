import type { Meta, StoryObj } from "@storybook/react-vite";
import { Dashboard } from "./Dashboard";
import { satellites } from "@/data/satellites";
import { issTrajectory } from "@/data/trajectory";

const meta: Meta<typeof Dashboard> = {
  title: "Components/Dashboard",
  component: Dashboard,
  parameters: { layout: "fullscreen" },
  args: {
    satellites,
    selected: satellites[0],
    trajectory: issTrajectory,
  },
};
export default meta;

type Story = StoryObj<typeof Dashboard>;

export const Sidebar: Story = { args: { layout: "sidebar" } };
export const Drawer: Story = { args: { layout: "drawer" } };
export const Overlay: Story = { args: { layout: "overlay" } };
export const BottomSheet: Story = { args: { layout: "bottom-sheet" } };
