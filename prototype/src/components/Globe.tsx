import { Canvas, useFrame, useLoader } from "@react-three/fiber";
import { OrbitControls, Stars } from "@react-three/drei";
import { useEffect, useMemo, useRef } from "react";
import * as THREE from "three";
import { interpolateSamples } from "@/lib/interpolate";
import { gmstFromUnix } from "@/lib/gmst";
import type { TrajectoryWindow } from "@/data/trajectory";

const EARTH_RADIUS_KM = 6378.137;

// Coordinate-frame derivation (verified by the reference markers below).
//
// 1. Scene mapping: Earth radius = 1 scene unit, Z_eci → Y_three, Y_eci → -Z_three.
//    See docs/architecture.md § Coordinate frames & rendering.
// 2. SphereGeometry default UVs put texture column u=0 at the -X equator,
//    so an equirectangular Earth texture (longitude 0 = column u=0.5) places
//    Greenwich at Earth-mesh local (1, 0, 0).
// 3. At GMST=0 the prime meridian aligns with the vernal equinox (ECI +X = scene +X).
//    Rotating the Earth mesh by +GMST around scene Y (= ECI Z = north) is
//    counterclockwise viewed from the north pole — matches Earth's eastward spin.
// Toggle `showReferences` in stories to visually confirm: the red marker should
// hug the prime meridian on the texture, the green marker should sit on the
// north pole, and the blue marker should sit at the international date line.
function eciKmToScene([x, y, z]: [number, number, number]): [number, number, number] {
  return [x / EARTH_RADIUS_KM, z / EARTH_RADIUS_KM, -y / EARTH_RADIUS_KM];
}

interface SceneProps {
  trajectory: TrajectoryWindow;
  simTimeMs: number;
  showReferences?: boolean;
}

function ReferenceMarkers() {
  // Parented to the Earth mesh so the markers rotate with the texture.
  // Red: Greenwich (lon 0, lat 0). Green: north pole. Blue: ±180° meridian.
  return (
    <>
      <mesh position={[1.005, 0, 0]}>
        <sphereGeometry args={[0.02, 12, 12]} />
        <meshBasicMaterial color={0xff4d4d} />
      </mesh>
      <mesh position={[0, 1.005, 0]}>
        <sphereGeometry args={[0.02, 12, 12]} />
        <meshBasicMaterial color={0x9aff5a} />
      </mesh>
      <mesh position={[-1.005, 0, 0]}>
        <sphereGeometry args={[0.02, 12, 12]} />
        <meshBasicMaterial color={0x4da3ff} />
      </mesh>
    </>
  );
}

function Earth({ simTimeMs, showReferences }: { simTimeMs: number; showReferences?: boolean }) {
  const texture = useLoader(THREE.TextureLoader, "/textures/earth.png");
  const ref = useRef<THREE.Mesh>(null);

  useFrame(() => {
    if (ref.current) ref.current.rotation.y = gmstFromUnix(simTimeMs / 1000);
  });

  return (
    <mesh ref={ref}>
      <sphereGeometry args={[1, 64, 64]} />
      <meshStandardMaterial map={texture} roughness={1} metalness={0} />
      {showReferences && <ReferenceMarkers />}
    </mesh>
  );
}

function OrbitPolyline({ trajectory }: { trajectory: TrajectoryWindow }) {
  const line = useMemo(() => {
    const points = trajectory.samples.map(
      (s) => new THREE.Vector3(...eciKmToScene(s.r_km)),
    );
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.LineBasicMaterial({
      color: 0x6cc1ff,
      transparent: true,
      opacity: 0.7,
    });
    return new THREE.Line(geometry, material);
  }, [trajectory]);

  useEffect(() => {
    return () => {
      line.geometry.dispose();
      (line.material as THREE.Material).dispose();
    };
  }, [line]);

  return <primitive object={line} />;
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

function Scene({ trajectory, simTimeMs, showReferences }: SceneProps) {
  return (
    <>
      <ambientLight intensity={0.25} />
      <directionalLight position={[5, 3, 5]} intensity={1.1} />
      <Stars radius={50} depth={20} count={2000} factor={2} fade />
      <Earth simTimeMs={simTimeMs} showReferences={showReferences} />
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
  showReferences?: boolean;
}

export function Globe({ trajectory, simTimeMs, className, showReferences }: GlobeProps) {
  return (
    <div className={className} style={{ width: "100%", height: "100%" }}>
      <Canvas camera={{ position: [3, 1.5, 3], fov: 45 }} dpr={[1, 2]}>
        <Scene trajectory={trajectory} simTimeMs={simTimeMs} showReferences={showReferences} />
      </Canvas>
    </div>
  );
}
