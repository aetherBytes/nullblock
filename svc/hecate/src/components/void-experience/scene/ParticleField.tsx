import React, { useRef, useMemo } from 'react';
import { useFrame, useThree } from '@react-three/fiber';
import { Points, PointMaterial } from '@react-three/drei';
import * as THREE from 'three';

interface ParticleFieldProps {
  count?: number;
  radius?: number;
}

const ParticleField: React.FC<ParticleFieldProps> = ({
  count = 2000,
  radius = 15
}) => {
  const pointsRef = useRef<THREE.Points>(null);
  const mouseRef = useRef(new THREE.Vector3(0, 0, 0));
  const { viewport } = useThree();

  // Track mouse position in 3D space
  React.useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      // Convert mouse position to normalized device coordinates
      const x = (event.clientX / window.innerWidth) * 2 - 1;
      const y = -(event.clientY / window.innerHeight) * 2 + 1;
      // Map to 3D space (approximate depth)
      mouseRef.current.set(x * viewport.width * 0.5, y * viewport.height * 0.5, 0);
    };

    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, [viewport]);

  // Generate initial particle positions - solar corona/flare distribution
  const [positions, velocities, particleData] = useMemo(() => {
    const pos = new Float32Array(count * 3);
    const vel = new Float32Array(count * 3);
    const data = new Float32Array(count * 4); // [baseRadius, speed, phase, type]

    for (let i = 0; i < count; i++) {
      const i3 = i * 3;
      const i4 = i * 4;

      // Particle type: 0 = core glow, 1 = corona, 2 = flare/streamer
      const type = Math.random();
      let r, theta, phi;

      if (type < 0.3) {
        // Core particles - dense near center
        r = Math.random() * radius * 0.3;
        theta = Math.random() * Math.PI * 2;
        phi = Math.acos(2 * Math.random() - 1);
      } else if (type < 0.7) {
        // Corona particles - medium distance, more uniform
        r = radius * 0.3 + Math.random() * radius * 0.4;
        theta = Math.random() * Math.PI * 2;
        phi = Math.acos(2 * Math.random() - 1);
      } else {
        // Flare/streamer particles - emanate outward in streams
        r = Math.random() * radius;
        // Concentrate along certain angles for streamer effect
        const streamAngle = Math.floor(Math.random() * 8) * (Math.PI / 4);
        theta = streamAngle + (Math.random() - 0.5) * 0.5;
        phi = Math.acos(2 * Math.random() - 1);
      }

      pos[i3] = r * Math.sin(phi) * Math.cos(theta);
      pos[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      pos[i3 + 2] = r * Math.cos(phi);

      // Outward velocity (solar wind effect) - stronger for outer particles
      const outwardSpeed = 0.01 + (r / radius) * 0.03;
      const nx = pos[i3] / (r || 1);
      const ny = pos[i3 + 1] / (r || 1);
      const nz = pos[i3 + 2] / (r || 1);

      vel[i3] = nx * outwardSpeed + (Math.random() - 0.5) * 0.01;
      vel[i3 + 1] = ny * outwardSpeed + (Math.random() - 0.5) * 0.01;
      vel[i3 + 2] = nz * outwardSpeed + (Math.random() - 0.5) * 0.01;

      // Store particle data
      data[i4] = r; // base radius
      data[i4 + 1] = 0.5 + Math.random() * 1.5; // speed multiplier
      data[i4 + 2] = Math.random() * Math.PI * 2; // phase offset
      data[i4 + 3] = type; // particle type
    }

    return [pos, vel, data];
  }, [count, radius]);

  useFrame((state) => {
    if (!pointsRef.current) return;

    const time = state.clock.elapsedTime;
    const positionAttr = pointsRef.current.geometry.attributes.position;
    const posArray = positionAttr.array as Float32Array;
    const mouse = mouseRef.current;

    for (let i = 0; i < count; i++) {
      const i3 = i * 3;
      const i4 = i * 4;

      const baseRadius = particleData[i4];
      const speedMult = particleData[i4 + 1];
      const phase = particleData[i4 + 2];
      const type = particleData[i4 + 3];

      // Current position
      let x = posArray[i3];
      let y = posArray[i3 + 1];
      let z = posArray[i3 + 2];

      // Distance from center
      const dist = Math.sqrt(x * x + y * y + z * z);

      // Outward flow with pulsing (solar flare effect)
      const pulse = 1 + Math.sin(time * 0.5 + phase) * 0.3;
      const flowSpeed = (type > 0.7 ? 0.015 : 0.008) * speedMult * pulse;

      // Normalize direction
      const nx = x / (dist || 1);
      const ny = y / (dist || 1);
      const nz = z / (dist || 1);

      // Apply outward velocity
      x += nx * flowSpeed;
      y += ny * flowSpeed;
      z += nz * flowSpeed;

      // Add swirling motion (magnetic field lines)
      const swirl = 0.002 * speedMult;
      x += Math.sin(time * 0.3 + phase) * swirl;
      y += Math.cos(time * 0.25 + phase) * swirl;
      z += Math.sin(time * 0.2 + phase + dist) * swirl * 0.5;

      // Mouse interaction - particles are attracted/repelled
      const dx = x - mouse.x;
      const dy = y - mouse.y;
      const dz = z - mouse.z;
      const mouseDist = Math.sqrt(dx * dx + dy * dy + dz * dz);

      if (mouseDist < 4) {
        // Gentle repulsion from mouse (like magnetic field interaction)
        const repelStrength = (4 - mouseDist) * 0.003;
        x += (dx / mouseDist) * repelStrength;
        y += (dy / mouseDist) * repelStrength;
        z += (dz / mouseDist) * repelStrength;
      }

      // Reset particles that drift too far - respawn near center
      if (dist > radius * 1.2) {
        const newR = Math.random() * radius * 0.3;
        const newTheta = Math.random() * Math.PI * 2;
        const newPhi = Math.acos(2 * Math.random() - 1);
        x = newR * Math.sin(newPhi) * Math.cos(newTheta);
        y = newR * Math.sin(newPhi) * Math.sin(newTheta);
        z = newR * Math.cos(newPhi);
      }

      posArray[i3] = x;
      posArray[i3 + 1] = y;
      posArray[i3 + 2] = z;
    }

    positionAttr.needsUpdate = true;

    // Very slow global rotation
    pointsRef.current.rotation.y = time * 0.01;
  });

  return (
    <Points ref={pointsRef} positions={positions} stride={3} frustumCulled={false}>
      <PointMaterial
        transparent
        color="#a0a8ff"
        size={0.025}
        sizeAttenuation
        depthWrite={false}
        opacity={0.8}
        blending={THREE.AdditiveBlending}
      />
    </Points>
  );
};

export default ParticleField;
