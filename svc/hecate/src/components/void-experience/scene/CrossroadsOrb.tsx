import React, { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

interface CrossroadsOrbProps {
  position?: [number, number, number];
}

// Create a radial gradient texture for smooth glow
const createGlowTexture = () => {
  const size = 256;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(
    size / 2, size / 2, 0,
    size / 2, size / 2, size / 2
  );
  gradient.addColorStop(0, 'rgba(255, 255, 255, 0.8)');
  gradient.addColorStop(0.2, 'rgba(255, 255, 255, 0.4)');
  gradient.addColorStop(0.4, 'rgba(240, 240, 255, 0.2)');
  gradient.addColorStop(0.6, 'rgba(220, 220, 255, 0.1)');
  gradient.addColorStop(0.8, 'rgba(200, 200, 255, 0.05)');
  gradient.addColorStop(1, 'rgba(180, 180, 255, 0)');

  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

// Create a circular particle texture
const createParticleTexture = () => {
  const size = 64;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(
    size / 2, size / 2, 0,
    size / 2, size / 2, size / 2
  );
  gradient.addColorStop(0, 'rgba(0, 0, 0, 1)');
  gradient.addColorStop(0.3, 'rgba(0, 0, 0, 0.8)');
  gradient.addColorStop(0.6, 'rgba(0, 0, 0, 0.4)');
  gradient.addColorStop(1, 'rgba(0, 0, 0, 0)');

  ctx.fillStyle = gradient;
  ctx.beginPath();
  ctx.arc(size / 2, size / 2, size / 2, 0, Math.PI * 2);
  ctx.fill();

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

const CrossroadsOrb: React.FC<CrossroadsOrbProps> = ({ position = [0, 0, 0] }) => {
  const groupRef = useRef<THREE.Group>(null);
  const sunRef = useRef<THREE.Mesh>(null);
  const glowRef = useRef<THREE.Sprite>(null);
  const glow2Ref = useRef<THREE.Sprite>(null);
  const particlesRef = useRef<THREE.Points>(null);
  const outerParticlesRef = useRef<THREE.Points>(null);

  const sunRadius = 1.6;

  // Create textures once
  const glowTexture = useMemo(() => createGlowTexture(), []);
  const particleTexture = useMemo(() => createParticleTexture(), []);

  // Sparse ephemeral particles - drifting randomly
  const particles = useMemo(() => {
    const count = 80;
    const positions = new Float32Array(count * 3);
    const offsets = new Float32Array(count * 3); // For random drift

    for (let i = 0; i < count; i++) {
      const i3 = i * 3;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = sunRadius + 0.2 + Math.random() * 2.0;

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      // Random drift directions
      offsets[i3] = (Math.random() - 0.5) * 2;
      offsets[i3 + 1] = (Math.random() - 0.5) * 2;
      offsets[i3 + 2] = (Math.random() - 0.5) * 2;
    }

    return { positions, offsets };
  }, [sunRadius]);

  // Outer wisps
  const outerParticles = useMemo(() => {
    const count = 50;
    const positions = new Float32Array(count * 3);
    const offsets = new Float32Array(count * 3);

    for (let i = 0; i < count; i++) {
      const i3 = i * 3;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = sunRadius + 2.0 + Math.random() * 2.5;

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      offsets[i3] = (Math.random() - 0.5) * 2;
      offsets[i3 + 1] = (Math.random() - 0.5) * 2;
      offsets[i3 + 2] = (Math.random() - 0.5) * 2;
    }

    return { positions, offsets };
  }, [sunRadius]);

  // Store base positions for drift animation
  const basePositions = useRef<Float32Array | null>(null);
  const outerBasePositions = useRef<Float32Array | null>(null);

  useFrame((state) => {
    const time = state.clock.elapsedTime;

    if (groupRef.current) {
      groupRef.current.position.y = position[1] + Math.sin(time * 0.5) * 0.1;
    }

    if (sunRef.current) {
      sunRef.current.rotation.y = time * 0.05;
    }

    // Pulse the glow
    if (glowRef.current) {
      const pulse = 9 + Math.sin(time * 1.5) * 0.5;
      glowRef.current.scale.set(pulse, pulse, 1);
    }
    if (glow2Ref.current) {
      const pulse = 12 + Math.sin(time * 1.2 + 1) * 0.6;
      glow2Ref.current.scale.set(pulse, pulse, 1);
    }

    // Animate particles with ephemeral random drift
    if (particlesRef.current) {
      if (!basePositions.current) {
        basePositions.current = new Float32Array(particles.positions);
      }

      const positions = particlesRef.current.geometry.attributes.position.array as Float32Array;
      const count = positions.length / 3;

      for (let i = 0; i < count; i++) {
        const i3 = i * 3;

        // Gentle random drift with noise
        const driftX = Math.sin(time * 0.3 + i * 0.5) * particles.offsets[i3] * 0.15;
        const driftY = Math.cos(time * 0.4 + i * 0.7) * particles.offsets[i3 + 1] * 0.15;
        const driftZ = Math.sin(time * 0.35 + i * 0.6) * particles.offsets[i3 + 2] * 0.15;

        positions[i3] = basePositions.current[i3] + driftX;
        positions[i3 + 1] = basePositions.current[i3 + 1] + driftY;
        positions[i3 + 2] = basePositions.current[i3 + 2] + driftZ;
      }
      particlesRef.current.geometry.attributes.position.needsUpdate = true;
      particlesRef.current.rotation.y = time * 0.02;
    }

    // Outer particles drift
    if (outerParticlesRef.current) {
      if (!outerBasePositions.current) {
        outerBasePositions.current = new Float32Array(outerParticles.positions);
      }

      const positions = outerParticlesRef.current.geometry.attributes.position.array as Float32Array;
      const count = positions.length / 3;

      for (let i = 0; i < count; i++) {
        const i3 = i * 3;

        const driftX = Math.sin(time * 0.2 + i * 0.8) * outerParticles.offsets[i3] * 0.2;
        const driftY = Math.cos(time * 0.25 + i * 0.9) * outerParticles.offsets[i3 + 1] * 0.2;
        const driftZ = Math.sin(time * 0.22 + i * 0.7) * outerParticles.offsets[i3 + 2] * 0.2;

        positions[i3] = outerBasePositions.current[i3] + driftX;
        positions[i3 + 1] = outerBasePositions.current[i3 + 1] + driftY;
        positions[i3 + 2] = outerBasePositions.current[i3 + 2] + driftZ;
      }
      outerParticlesRef.current.geometry.attributes.position.needsUpdate = true;
      outerParticlesRef.current.rotation.y = -time * 0.01;
    }
  });

  return (
    <group ref={groupRef} position={position}>
      {/* Corona glow - back layer */}
      <sprite ref={glow2Ref} scale={[12, 12, 1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.4}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Corona glow - front layer */}
      <sprite ref={glowRef} scale={[9, 9, 1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.6}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Core black sun sphere */}
      <mesh ref={sunRef}>
        <sphereGeometry args={[sunRadius, 64, 64]} />
        <meshBasicMaterial color="#000000" />
      </mesh>

      {/* Ephemeral drifting particles - circular */}
      <points ref={particlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={particles.positions.length / 3}
            array={particles.positions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={particleTexture}
          size={0.15}
          transparent
          opacity={0.85}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.NormalBlending}
        />
      </points>

      {/* Outer wisps - circular */}
      <points ref={outerParticlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={outerParticles.positions.length / 3}
            array={outerParticles.positions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={particleTexture}
          size={0.12}
          transparent
          opacity={0.7}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.NormalBlending}
        />
      </points>

      {/* Central white light - illuminates surrounding agents */}
      <pointLight
        color="#ffffff"
        intensity={5}
        distance={12}
        decay={2}
      />
    </group>
  );
};

export default CrossroadsOrb;
