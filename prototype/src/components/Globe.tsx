import { Canvas, useFrame, useLoader } from "@react-three/fiber";
import { OrbitControls, Stars } from "@react-three/drei";
import { useMemo, useRef } from "react";
import * as THREE from "three";
import { interpolateSamples } from "@/lib/interpolate";
import { gmstFromUnix } from "@/lib/gmst";
import type { TrajectoryWindow } from "@/data/trajectory";

const EARTH_RADIUS_KM = 6378.137;

// Convert ECI km vector to scene coordinates: Earth radius = 1 scene unit,
// Nyx Z_eci → three.js Y. (See docs/architecture.md § Coordinate frames.)
function eciKmToScene([x, y, z]: [number, number, number]): [number, number, number] {
  return [x / EARTH_RADIUS_KM, z / EARTH_RADIUS_KM, -y / EARTH_RADIUS_KM];
}

interface SceneProps {
  trajectory: TrajectoryWindow;
  simTimeMs: number;
}

function Earth({ simTimeMs }: { simTimeMs: number }) {
  const texture = useLoader(THREE.TextureLoader, "/textures/earth.png");
  const ref = useRef<THREE.Mesh>(null);

  useFrame(() => {
    if (ref.current) ref.current.rotation.y = gmstFromUnix(simTimeMs / 1000);
  });

  return (
    <mesh ref={ref}>
      <sphereGeometry args={[1, 64, 64]} />
      <meshStandardMaterial map={texture} roughness={1} metalness={0} />
    </mesh>
  );
}

function OrbitPolyline({ trajectory }: { trajectory: TrajectoryWindow }) {
  const points = useMemo(() => {
    return trajectory.samples.map((s) => new THREE.Vector3(...eciKmToScene(s.r_km)));
  }, [trajectory]);

  const geometry = useMemo(() => {
    const g = new THREE.BufferGeometry().setFromPoints(points);
    return g;
  }, [points]);

  return (
    <primitive
      object={
        new THREE.Line(
          geometry,
          new THREE.LineBasicMaterial({ color: 0x6cc1ff, transparent: true, opacity: 0.7 }),
        )
      }
    />
  );
}

function Marker({ trajectory, simTimeMs }: SceneProps) {
  const ref = useRef<THREE.Mesh>(null);
  const startMs = useMemo(() => Date.parse(trajectory.start_at), [trajectory]);
  const durationMs = trajectory.duration_s * 1000;

  useFrame(() => {
    const elapsed = (simTimeMs - startMs) % durationMs;
    const t = (elapsed + durationMs) % durationMs;
    const r = interpolateSamples(trajectory.samples, t / 1000);
    const [x, y, z] = eciKmToScene(r);
    ref.current?.position.set(x, y, z);
  });

  return (
    <mesh ref={ref}>
      <sphereGeometry args={[0.025, 16, 16]} />
      <meshBasicMaterial color={0x9aff5a} />
    </mesh>
  );
}

function Scene({ trajectory, simTimeMs }: SceneProps) {
  return (
    <>
      <ambientLight intensity={0.25} />
      <directionalLight position={[5, 3, 5]} intensity={1.1} />
      <Stars radius={50} depth={20} count={2000} factor={2} fade />
      <Earth simTimeMs={simTimeMs} />
      <OrbitPolyline trajectory={trajectory} />
      <Marker trajectory={trajectory} simTimeMs={simTimeMs} />
      <OrbitControls enablePan={false} minDistance={1.5} maxDistance={8} />
    </>
  );
}

export interface GlobeProps {
  trajectory: TrajectoryWindow;
  simTimeMs: number;
  className?: string;
}

export function Globe({ trajectory, simTimeMs, className }: GlobeProps) {
  return (
    <div className={className} style={{ width: "100%", height: "100%" }}>
      <Canvas camera={{ position: [3, 1.5, 3], fov: 45 }} dpr={[1, 2]}>
        <Scene trajectory={trajectory} simTimeMs={simTimeMs} />
      </Canvas>
    </div>
  );
}
