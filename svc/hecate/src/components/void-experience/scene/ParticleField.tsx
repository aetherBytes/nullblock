import React, { useRef, useMemo } from 'react';
import { useFrame, useThree } from '@react-three/fiber';
import { Points, PointMaterial } from '@react-three/drei';
import * as THREE from 'three';

interface ParticleFieldProps {
  count?: number;
  radius?: number;
}

const ParticleField: React.FC<ParticleFieldProps> = ({
  count = 3000,
  radius = 20
}) => {
  const coreRef = useRef<THREE.Points>(null);
  const coronaRef = useRef<THREE.Points>(null);
  const flareRef = useRef<THREE.Points>(null);
  const mouseRef = useRef(new THREE.Vector3(0, 0, 0));
  const { viewport, camera } = useThree();

  // Track mouse position in 3D space with better depth mapping
  React.useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      const x = (event.clientX / window.innerWidth) * 2 - 1;
      const y = -(event.clientY / window.innerHeight) * 2 + 1;
      // Better 3D mapping using camera distance
      const depth = camera.position.z * 0.6;
      mouseRef.current.set(x * viewport.width * 0.4, y * viewport.height * 0.4, depth * 0.2);
    };

    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, [viewport, camera]);

  // Core particles - dense bright center
  const coreParticles = useMemo(() => {
    const coreCount = Math.floor(count * 0.25);
    const pos = new Float32Array(coreCount * 3);
    const data = new Float32Array(coreCount * 2); // [phase, speed]

    for (let i = 0; i < coreCount; i++) {
      const i3 = i * 3;
      const r = Math.random() * radius * 0.15;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      pos[i3] = r * Math.sin(phi) * Math.cos(theta);
      pos[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      pos[i3 + 2] = r * Math.cos(phi);

      data[i * 2] = Math.random() * Math.PI * 2;
      data[i * 2 + 1] = 0.3 + Math.random() * 0.5;
    }
    return { positions: pos, data, count: coreCount };
  }, [count, radius]);

  // Corona particles - medium distance, flowing outward
  const coronaParticles = useMemo(() => {
    const coronaCount = Math.floor(count * 0.45);
    const pos = new Float32Array(coronaCount * 3);
    const data = new Float32Array(coronaCount * 2);

    for (let i = 0; i < coronaCount; i++) {
      const i3 = i * 3;
      const r = radius * 0.15 + Math.random() * radius * 0.5;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      pos[i3] = r * Math.sin(phi) * Math.cos(theta);
      pos[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      pos[i3 + 2] = r * Math.cos(phi);

      data[i * 2] = Math.random() * Math.PI * 2;
      data[i * 2 + 1] = 0.5 + Math.random() * 1.0;
    }
    return { positions: pos, data, count: coronaCount };
  }, [count, radius]);

  // Flare/streamer particles - long streams emanating outward
  const flareParticles = useMemo(() => {
    const flareCount = Math.floor(count * 0.3);
    const pos = new Float32Array(flareCount * 3);
    const data = new Float32Array(flareCount * 3); // [phase, speed, streamAngle]

    for (let i = 0; i < flareCount; i++) {
      const i3 = i * 3;
      // Create 6-8 distinct streams
      const streamAngle = Math.floor(Math.random() * 6) * (Math.PI / 3);
      const streamSpread = (Math.random() - 0.5) * 0.4;
      const r = Math.random() * radius;
      const theta = streamAngle + streamSpread;
      const phi = Math.acos(2 * Math.random() - 1);

      pos[i3] = r * Math.sin(phi) * Math.cos(theta);
      pos[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      pos[i3 + 2] = r * Math.cos(phi);

      data[i * 3] = Math.random() * Math.PI * 2;
      data[i * 3 + 1] = 1.0 + Math.random() * 1.5;
      data[i * 3 + 2] = streamAngle;
    }
    return { positions: pos, data, count: flareCount };
  }, [count, radius]);

  useFrame((state) => {
    const time = state.clock.elapsedTime;
    const mouse = mouseRef.current;

    // Animate core particles - gentle pulsing
    if (coreRef.current) {
      const posAttr = coreRef.current.geometry.attributes.position;
      const posArray = posAttr.array as Float32Array;

      for (let i = 0; i < coreParticles.count; i++) {
        const i3 = i * 3;
        const phase = coreParticles.data[i * 2];
        const speed = coreParticles.data[i * 2 + 1];

        let x = posArray[i3];
        let y = posArray[i3 + 1];
        let z = posArray[i3 + 2];
        const dist = Math.sqrt(x * x + y * y + z * z);

        // Gentle breathing motion
        const pulse = 1 + Math.sin(time * 0.8 + phase) * 0.15;
        const scale = pulse * speed * 0.002;
        const nx = x / (dist || 1);
        const ny = y / (dist || 1);
        const nz = z / (dist || 1);

        x += nx * scale + Math.sin(time + phase) * 0.003;
        y += ny * scale + Math.cos(time * 0.9 + phase) * 0.003;
        z += nz * scale;

        // Mouse interaction - strong attraction to cursor
        const dx = mouse.x - x;
        const dy = mouse.y - y;
        const dz = mouse.z - z;
        const mouseDist = Math.sqrt(dx * dx + dy * dy + dz * dz);

        if (mouseDist < 6) {
          const attractStrength = (6 - mouseDist) * 0.008;
          x += dx * attractStrength * 0.1;
          y += dy * attractStrength * 0.1;
          z += dz * attractStrength * 0.05;
        }

        // Keep near center
        if (dist > radius * 0.2) {
          x *= 0.98;
          y *= 0.98;
          z *= 0.98;
        }

        posArray[i3] = x;
        posArray[i3 + 1] = y;
        posArray[i3 + 2] = z;
      }
      posAttr.needsUpdate = true;
      coreRef.current.rotation.y = time * 0.02;
    }

    // Animate corona particles - flowing outward
    if (coronaRef.current) {
      const posAttr = coronaRef.current.geometry.attributes.position;
      const posArray = posAttr.array as Float32Array;

      for (let i = 0; i < coronaParticles.count; i++) {
        const i3 = i * 3;
        const phase = coronaParticles.data[i * 2];
        const speed = coronaParticles.data[i * 2 + 1];

        let x = posArray[i3];
        let y = posArray[i3 + 1];
        let z = posArray[i3 + 2];
        const dist = Math.sqrt(x * x + y * y + z * z);

        // Outward flow
        const nx = x / (dist || 1);
        const ny = y / (dist || 1);
        const nz = z / (dist || 1);
        const flowSpeed = 0.012 * speed * (1 + Math.sin(time * 0.5 + phase) * 0.3);

        x += nx * flowSpeed;
        y += ny * flowSpeed;
        z += nz * flowSpeed;

        // Swirl motion
        x += Math.sin(time * 0.4 + phase) * 0.004;
        y += Math.cos(time * 0.35 + phase) * 0.004;

        // Mouse interaction - repel from cursor
        const dx = x - mouse.x;
        const dy = y - mouse.y;
        const dz = z - mouse.z;
        const mouseDist = Math.sqrt(dx * dx + dy * dy + dz * dz);

        if (mouseDist < 5) {
          const repelStrength = (5 - mouseDist) * 0.015;
          x += (dx / mouseDist) * repelStrength;
          y += (dy / mouseDist) * repelStrength;
          z += (dz / mouseDist) * repelStrength * 0.5;
        }

        // Reset if too far
        if (dist > radius * 0.8) {
          const newR = radius * 0.15 + Math.random() * radius * 0.1;
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
      posAttr.needsUpdate = true;
      coronaRef.current.rotation.y = time * 0.015;
    }

    // Animate flare particles - fast outward streams
    if (flareRef.current) {
      const posAttr = flareRef.current.geometry.attributes.position;
      const posArray = posAttr.array as Float32Array;

      for (let i = 0; i < flareParticles.count; i++) {
        const i3 = i * 3;
        const phase = flareParticles.data[i * 3];
        const speed = flareParticles.data[i * 3 + 1];

        let x = posArray[i3];
        let y = posArray[i3 + 1];
        let z = posArray[i3 + 2];
        const dist = Math.sqrt(x * x + y * y + z * z);

        // Fast outward flow
        const nx = x / (dist || 1);
        const ny = y / (dist || 1);
        const nz = z / (dist || 1);
        const flowSpeed = 0.025 * speed * (1 + Math.sin(time * 0.3 + phase) * 0.5);

        x += nx * flowSpeed;
        y += ny * flowSpeed;
        z += nz * flowSpeed;

        // Mouse interaction - dramatic scatter
        const dx = x - mouse.x;
        const dy = y - mouse.y;
        const dz = z - mouse.z;
        const mouseDist = Math.sqrt(dx * dx + dy * dy + dz * dz);

        if (mouseDist < 4) {
          const scatterStrength = (4 - mouseDist) * 0.025;
          x += (dx / mouseDist) * scatterStrength;
          y += (dy / mouseDist) * scatterStrength;
          z += (dz / mouseDist) * scatterStrength * 0.5;
        }

        // Reset if too far
        if (dist > radius) {
          const streamAngle = flareParticles.data[i * 3 + 2];
          const streamSpread = (Math.random() - 0.5) * 0.4;
          const newR = Math.random() * radius * 0.2;
          const newTheta = streamAngle + streamSpread;
          const newPhi = Math.acos(2 * Math.random() - 1);
          x = newR * Math.sin(newPhi) * Math.cos(newTheta);
          y = newR * Math.sin(newPhi) * Math.sin(newTheta);
          z = newR * Math.cos(newPhi);
        }

        posArray[i3] = x;
        posArray[i3 + 1] = y;
        posArray[i3 + 2] = z;
      }
      posAttr.needsUpdate = true;
      flareRef.current.rotation.y = time * 0.008;
    }
  });

  return (
    <group>
      {/* Core - bright white center */}
      <Points ref={coreRef} positions={coreParticles.positions} stride={3} frustumCulled={false}>
        <PointMaterial
          transparent
          color="#ffffff"
          size={0.06}
          sizeAttenuation
          depthWrite={false}
          opacity={0.95}
          blending={THREE.AdditiveBlending}
        />
      </Points>

      {/* Corona - silver/grey flowing particles */}
      <Points ref={coronaRef} positions={coronaParticles.positions} stride={3} frustumCulled={false}>
        <PointMaterial
          transparent
          color="#c0c0c8"
          size={0.04}
          sizeAttenuation
          depthWrite={false}
          opacity={0.75}
          blending={THREE.AdditiveBlending}
        />
      </Points>

      {/* Flares - dim grey streamers */}
      <Points ref={flareRef} positions={flareParticles.positions} stride={3} frustumCulled={false}>
        <PointMaterial
          transparent
          color="#888890"
          size={0.03}
          sizeAttenuation
          depthWrite={false}
          opacity={0.5}
          blending={THREE.AdditiveBlending}
        />
      </Points>
    </group>
  );
};

export default ParticleField;
