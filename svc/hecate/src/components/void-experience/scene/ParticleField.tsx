import React, { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import { Points, PointMaterial } from '@react-three/drei';
import * as THREE from 'three';

interface ParticleFieldProps {
  count?: number;
  radius?: number;
}

const ParticleField: React.FC<ParticleFieldProps> = ({
  count = 1500,
  radius = 12
}) => {
  const pointsRef = useRef<THREE.Points>(null);

  // Generate initial particle positions in a sphere
  const [positions, velocities] = useMemo(() => {
    const pos = new Float32Array(count * 3);
    const vel = new Float32Array(count * 3);

    for (let i = 0; i < count; i++) {
      const i3 = i * 3;

      // Random position within a sphere
      const r = Math.random() * radius;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      pos[i3] = r * Math.sin(phi) * Math.cos(theta);
      pos[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      pos[i3 + 2] = r * Math.cos(phi);

      // Random drift velocity (very slow)
      vel[i3] = (Math.random() - 0.5) * 0.02;
      vel[i3 + 1] = (Math.random() - 0.5) * 0.02;
      vel[i3 + 2] = (Math.random() - 0.5) * 0.02;
    }

    return [pos, vel];
  }, [count, radius]);

  // Store random values for each particle (for individual animation offsets)
  const randomOffsets = useMemo(() => {
    return new Float32Array(count).map(() => Math.random() * Math.PI * 2);
  }, [count]);

  useFrame((state) => {
    if (!pointsRef.current) return;

    const time = state.clock.elapsedTime;
    const positionAttr = pointsRef.current.geometry.attributes.position;
    const posArray = positionAttr.array as Float32Array;

    for (let i = 0; i < count; i++) {
      const i3 = i * 3;
      const offset = randomOffsets[i];

      // Drift with sine-wave motion
      posArray[i3] += velocities[i3] * 0.1;
      posArray[i3] += Math.sin(time * 0.2 + offset) * 0.001;

      posArray[i3 + 1] += velocities[i3 + 1] * 0.1;
      posArray[i3 + 1] += Math.cos(time * 0.15 + offset) * 0.001;

      posArray[i3 + 2] += velocities[i3 + 2] * 0.1;
      posArray[i3 + 2] += Math.sin(time * 0.1 + offset) * 0.001;

      // Wrap particles that drift too far
      const distSq =
        posArray[i3] ** 2 +
        posArray[i3 + 1] ** 2 +
        posArray[i3 + 2] ** 2;

      if (distSq > radius * radius * 1.5) {
        // Reset to opposite side
        const scale = -0.5;
        posArray[i3] *= scale;
        posArray[i3 + 1] *= scale;
        posArray[i3 + 2] *= scale;
      }
    }

    positionAttr.needsUpdate = true;

    // Slow global rotation
    pointsRef.current.rotation.y = time * 0.02;
    pointsRef.current.rotation.x = Math.sin(time * 0.05) * 0.1;
  });

  return (
    <Points ref={pointsRef} positions={positions} stride={3} frustumCulled={false}>
      <PointMaterial
        transparent
        color="#ffffff"
        size={0.02}
        sizeAttenuation
        depthWrite={false}
        opacity={0.7}
        blending={THREE.AdditiveBlending}
      />
    </Points>
  );
};

export default ParticleField;
