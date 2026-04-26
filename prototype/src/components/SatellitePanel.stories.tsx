import type { Meta, StoryObj } from "@storybook/react-vite";
import { SatellitePanel } from "./SatellitePanel";
import { satellites, elementsByNorad } from "@/data/satellites";

const meta: Meta<typeof SatellitePanel> = {
  title: "Components/SatellitePanel",
  component: SatellitePanel,
  parameters: { layout: "centered" },
};
export default meta;

type Story = StoryObj<typeof SatellitePanel>;

const make = (norad: number): Story => ({
  args: {
    satellite: satellites.find((s) => s.norad_id === norad)!,
    elements: elementsByNorad[norad],
  },
});

export const ISS = make(25544);
export const Hubble = make(20580);
export const Starlink = make(44713);
export const GPS = make(32260);
export const NOAA = make(33591);
