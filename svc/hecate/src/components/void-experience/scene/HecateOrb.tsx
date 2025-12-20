import React, { useRef } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

interface HecateOrbProps {
  position?: [number, number, number];
}

const HecateOrb: React.FC<HecateOrbProps> = ({ position = [0, 0, 0] }) => {
  const groupRef = useRef<THREE.Group>(null);
  const meshRef = useRef<THREE.Mesh>(null);
  const materialRef = useRef<THREE.MeshStandardMaterial>(null);
  const glowMaterialRef = useRef<THREE.MeshBasicMaterial>(null);

  const baseEmissiveIntensity = 0.8;

  useFrame((state) => {
    const time = state.clock.elapsedTime;

    if (groupRef.current) {
      // Floating motion - gentle Y oscillation
      groupRef.current.position.y = position[1] + Math.sin(time * 0.5) * 0.08;
      // Gentle rotation
      groupRef.current.rotation.y = time * 0.1;
    }

    if (materialRef.current) {
      // Pulsing emissive intensity - synaptic heartbeat
      const pulse = Math.sin(time * 2) * 0.3 + Math.sin(time * 0.7) * 0.15;
      materialRef.current.emissiveIntensity = baseEmissiveIntensity + pulse;
    }

    if (glowMaterialRef.current) {
      // Outer glow opacity pulse
      glowMaterialRef.current.opacity = 0.15 + Math.sin(time * 1.5) * 0.05;
    }

    if (meshRef.current) {
      // Subtle breathing scale
      const scale = 1 + Math.sin(time * 1.2) * 0.015;
      meshRef.current.scale.setScalar(scale);
    }
  });

  return (
    <group ref={groupRef} position={position}>
      {/* Core orb */}
      <mesh ref={meshRef} castShadow>
        <sphereGeometry args={[0.35, 64, 64]} />
        <meshStandardMaterial
          ref={materialRef}
          color="#e6c200"
          emissive="#e6c200"
          emissiveIntensity={baseEmissiveIntensity}
          metalness={0.7}
          roughness={0.2}
          envMapIntensity={1}
        />
      </mesh>

      {/* Inner bright core */}
      <mesh>
        <sphereGeometry args={[0.15, 32, 32]} />
        <meshBasicMaterial color="#ffffff" transparent opacity={0.9} />
      </mesh>

      {/* Outer glow shell */}
      <mesh>
        <sphereGeometry args={[0.5, 32, 32]} />
        <meshBasicMaterial
          ref={glowMaterialRef}
          color="#e6c200"
          transparent
          opacity={0.15}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Outermost halo */}
      <mesh>
        <sphereGeometry args={[0.7, 16, 16]} />
        <meshBasicMaterial
          color="#e8e8e8"
          transparent
          opacity={0.05}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Point light emanating from orb */}
      <pointLight
        color="#e6c200"
        intensity={2}
        distance={8}
        decay={2}
      />
    </group>
  );
};

export default HecateOrb;
