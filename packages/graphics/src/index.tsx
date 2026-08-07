import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { useEffect, useMemo, useRef } from "react";
import { CatmullRomCurve3, Color, MathUtils, Vector3 } from "three";
import type { Group, Mesh } from "three";

export interface AmbientSceneProps {
  motionEnabled?: boolean;
}

interface FloatingShapeProps {
  motionEnabled: boolean;
  position: [number, number, number];
  rotation: [number, number, number];
  scale: [number, number, number];
  speed: number;
  color?: string;
}

function FloatingShape({
  motionEnabled,
  position,
  rotation,
  scale,
  speed,
  color = "#152747",
}: FloatingShapeProps) {
  const group = useRef<Group>(null);

  useFrame(({ clock }, delta) => {
    if (!motionEnabled || !group.current) return;
    const elapsed = clock.elapsedTime;
    group.current.rotation.x += delta * speed * 0.16;
    group.current.rotation.y += delta * speed * 0.22;
    group.current.position.y =
      position[1] + Math.sin(elapsed * speed + position[0]) * 0.16;
  });

  return (
    <group ref={group} position={position} rotation={rotation} scale={scale}>
      <mesh castShadow receiveShadow>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial
          color={color}
          metalness={0.92}
          roughness={0.2}
          envMapIntensity={0.9}
        />
      </mesh>
      <mesh scale={1.012}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial
          color="#4ca8ff"
          transparent
          opacity={0.22}
          wireframe
        />
      </mesh>
    </group>
  );
}

interface PortalFrameProps {
  motionEnabled: boolean;
}

function PortalFrame({ motionEnabled }: PortalFrameProps) {
  const group = useRef<Group>(null);

  useFrame(({ clock }, delta) => {
    if (!motionEnabled || !group.current) return;
    group.current.rotation.z += delta * 0.035;
    group.current.rotation.y = Math.sin(clock.elapsedTime * 0.22) * 0.18;
  });

  return (
    <group ref={group} position={[3.2, 0.45, -2.4]} rotation={[0.25, -0.5, 0]}>
      <mesh>
        <torusGeometry args={[1.45, 0.16, 10, 8]} />
        <meshStandardMaterial
          color="#172d52"
          emissive="#397dff"
          emissiveIntensity={0.42}
          metalness={0.88}
          roughness={0.2}
        />
      </mesh>
      <mesh scale={1.08}>
        <torusGeometry args={[1.45, 0.035, 8, 72]} />
        <meshBasicMaterial color="#84d5ff" transparent opacity={0.55} />
      </mesh>
    </group>
  );
}

function LightTrail({
  color,
  points,
}: {
  color: string;
  points: [number, number, number][];
}) {
  const curve = useMemo(
    () => new CatmullRomCurve3(points.map((point) => new Vector3(...point))),
    [points],
  );

  return (
    <mesh>
      <tubeGeometry args={[curve, 48, 0.018, 6, false]} />
      <meshBasicMaterial color={color} transparent opacity={0.72} />
    </mesh>
  );
}

function CameraRig({ motionEnabled }: { motionEnabled: boolean }) {
  const { camera } = useThree();
  const target = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const onPointerMove = (event: PointerEvent) => {
      target.current = {
        x: (event.clientX / window.innerWidth) * 2 - 1,
        y: -((event.clientY / window.innerHeight) * 2 - 1),
      };
    };
    const onFocusMove = (event: Event) => {
      const detail = (event as CustomEvent<{ x: number; y: number }>).detail;
      if (detail) target.current = detail;
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("limen-focus-move", onFocusMove);
    return () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("limen-focus-move", onFocusMove);
    };
  }, []);

  useFrame((_, delta) => {
    if (!motionEnabled) return;
    const strength = Math.min(delta * 2.4, 1);
    camera.position.x = MathUtils.lerp(
      camera.position.x,
      target.current.x * 0.42,
      strength,
    );
    camera.position.y = MathUtils.lerp(
      camera.position.y,
      0.2 + target.current.y * 0.24,
      strength,
    );
    camera.lookAt(0, 0, -2.6);
  });

  return null;
}

function SpatialWorld({ motionEnabled }: Required<AmbientSceneProps>) {
  const floor = useRef<Mesh>(null);

  useFrame(({ clock }) => {
    if (!motionEnabled || !floor.current) return;
    const material = floor.current.material;
    if (Array.isArray(material)) return;
    material.opacity = 0.11 + Math.sin(clock.elapsedTime * 0.38) * 0.018;
  });

  return (
    <>
      <color attach="background" args={[new Color("#010306")]} />
      <fog attach="fog" args={["#030712", 7, 20]} />
      <ambientLight intensity={0.42} color="#5673a8" />
      <directionalLight position={[-5, 7, 5]} intensity={3.2} color="#8fc9ff" />
      <pointLight
        position={[4, 1, 1]}
        intensity={15}
        distance={9}
        color="#755bff"
      />
      <pointLight
        position={[-4, -1, 2]}
        intensity={11}
        distance={8}
        color="#208dff"
      />

      <CameraRig motionEnabled={motionEnabled} />
      <PortalFrame motionEnabled={motionEnabled} />

      <FloatingShape
        motionEnabled={motionEnabled}
        position={[-4.8, 1.7, -2.6]}
        rotation={[0.35, 0.65, 0.2]}
        scale={[2.4, 0.46, 0.72]}
        speed={0.42}
      />
      <FloatingShape
        motionEnabled={motionEnabled}
        position={[-3.6, -1.35, -1.4]}
        rotation={[0.45, -0.35, 0.5]}
        scale={[1.6, 0.75, 0.85]}
        speed={0.58}
        color="#0c1c35"
      />
      <FloatingShape
        motionEnabled={motionEnabled}
        position={[5.1, 2.25, -3.8]}
        rotation={[0.2, 0.4, -0.35]}
        scale={[2.1, 0.5, 0.6]}
        speed={0.34}
      />
      <FloatingShape
        motionEnabled={motionEnabled}
        position={[5.1, -1.9, -2.2]}
        rotation={[-0.45, 0.8, 0.25]}
        scale={[1.4, 0.65, 0.72]}
        speed={0.5}
        color="#111b35"
      />
      <FloatingShape
        motionEnabled={motionEnabled}
        position={[0.2, 2.9, -6]}
        rotation={[0.6, 0.2, 0.65]}
        scale={[0.75, 0.75, 0.75]}
        speed={0.72}
      />
      <FloatingShape
        motionEnabled={motionEnabled}
        position={[-0.7, -0.6, -7]}
        rotation={[0.2, 0.9, 0.15]}
        scale={[0.42, 0.42, 0.42]}
        speed={0.86}
      />

      <LightTrail
        color="#43c8ff"
        points={[
          [-7, -0.7, -3],
          [-3, 0.4, -4],
          [0.5, -0.2, -5],
          [6, 1.5, -5],
        ]}
      />
      <LightTrail
        color="#985cff"
        points={[
          [-5, 2.7, -7],
          [-1, 2, -5],
          [2, 2.4, -4],
          [7, 0.8, -4],
        ]}
      />

      <mesh
        ref={floor}
        position={[0, -2.9, -2.2]}
        rotation={[-Math.PI / 2, 0, 0]}
        receiveShadow
      >
        <planeGeometry args={[28, 18, 1, 1]} />
        <meshStandardMaterial
          color="#071329"
          emissive="#123b78"
          emissiveIntensity={0.28}
          metalness={0.8}
          roughness={0.12}
          transparent
          opacity={0.12}
        />
      </mesh>
    </>
  );
}

export function AmbientScene({ motionEnabled = true }: AmbientSceneProps) {
  return (
    <div className="ambient-scene" aria-hidden="true">
      <div className="ambient-scene__fallback" />
      <Canvas
        className="ambient-scene__canvas"
        camera={{ fov: 47, near: 0.1, far: 40, position: [0, 0.2, 7.8] }}
        dpr={[0.7, 1.35]}
        frameloop={motionEnabled ? "always" : "demand"}
        gl={{
          alpha: false,
          antialias: true,
          powerPreference: "high-performance",
        }}
      >
        <SpatialWorld motionEnabled={motionEnabled} />
      </Canvas>
      <div className="ambient-scene__mist" />
      <div className="ambient-scene__vignette" />
      <div className="ambient-scene__grain" />
    </div>
  );
}
