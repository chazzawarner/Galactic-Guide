import { useEffect, useRef, useState } from "react";

export type Speed = 1 | 10 | 100 | 1000;

export interface SimClock {
  simTimeMs: number;
  speed: Speed;
  playing: boolean;
  setSpeed: (s: Speed) => void;
  toggle: () => void;
  now: () => void;
}

// Drives `simTime` forward at `speed * realDt` while playing.
// Real time elapsed since mount is the only state we track; we recompute simTime
// from speed + base anchor on every speed/play change so transitions are clean.
export function useSimClock(initialEpochMs: number = Date.now()): SimClock {
  const [playing, setPlaying] = useState(true);
  const [speed, setSpeedState] = useState<Speed>(1);
  const [simTimeMs, setSimTimeMs] = useState(initialEpochMs);

  // Anchor: at `realAnchor`, simTime was `simAnchor`. Advances are computed off this.
  const realAnchorRef = useRef(performance.now());
  const simAnchorRef = useRef(initialEpochMs);

  const reanchor = (newSim: number) => {
    realAnchorRef.current = performance.now();
    simAnchorRef.current = newSim;
  };

  const setSpeed = (s: Speed) => {
    reanchor(simTimeMs);
    setSpeedState(s);
  };

  const toggle = () => {
    if (!playing) reanchor(simTimeMs);
    setPlaying((p) => !p);
  };

  const now = () => {
    const real = Date.now();
    setSimTimeMs(real);
    reanchor(real);
  };

  useEffect(() => {
    if (!playing) return;
    let raf = 0;
    const loop = () => {
      const realDt = performance.now() - realAnchorRef.current;
      setSimTimeMs(simAnchorRef.current + realDt * speed);
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, [playing, speed]);

  return { simTimeMs, speed, playing, setSpeed, toggle, now };
}
