import React, { useRef, useMemo, useCallback } from 'react';
import { useFrame, useThree } from '@react-three/fiber';
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

// Create a circular particle texture with color parameter
const createParticleTexture = (color: 'black' | 'silver' | 'white') => {
  const size = 64;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(
    size / 2, size / 2, 0,
    size / 2, size / 2, size / 2
  );

  if (color === 'black') {
    gradient.addColorStop(0, 'rgba(0, 0, 0, 1)');
    gradient.addColorStop(0.3, 'rgba(0, 0, 0, 0.8)');
    gradient.addColorStop(0.6, 'rgba(0, 0, 0, 0.4)');
    gradient.addColorStop(1, 'rgba(0, 0, 0, 0)');
  } else if (color === 'silver') {
    gradient.addColorStop(0, 'rgba(200, 200, 210, 1)');
    gradient.addColorStop(0.3, 'rgba(180, 180, 195, 0.8)');
    gradient.addColorStop(0.6, 'rgba(160, 160, 180, 0.4)');
    gradient.addColorStop(1, 'rgba(140, 140, 160, 0)');
  } else {
    gradient.addColorStop(0, 'rgba(255, 255, 255, 1)');
    gradient.addColorStop(0.3, 'rgba(255, 255, 255, 0.8)');
    gradient.addColorStop(0.6, 'rgba(255, 255, 255, 0.4)');
    gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');
  }

  ctx.fillStyle = gradient;
  ctx.beginPath();
  ctx.arc(size / 2, size / 2, size / 2, 0, Math.PI * 2);
  ctx.fill();

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

// Particle data structure for interactive solar particles
interface SolarParticle {
  basePosition: THREE.Vector3;
  currentPosition: THREE.Vector3;
  targetPosition: THREE.Vector3;
  velocity: THREE.Vector3;
  orbitSpeed: number;
  orbitAxis: THREE.Vector3;
  phase: number;
  size: number;
  type: 'black' | 'silver' | 'white';
}

const CrossroadsOrb: React.FC<CrossroadsOrbProps> = ({ position = [0, 0, 0] }) => {
  const groupRef = useRef<THREE.Group>(null);
  const sunRef = useRef<THREE.Mesh>(null);
  const glowRef = useRef<THREE.Sprite>(null);
  const glow2Ref = useRef<THREE.Sprite>(null);
  const blackParticlesRef = useRef<THREE.Points>(null);
  const silverParticlesRef = useRef<THREE.Points>(null);
  const whiteParticlesRef = useRef<THREE.Points>(null);
  const solarFlareRef = useRef<THREE.Points>(null);

  const { camera, raycaster, pointer } = useThree();
  const mouseWorldPos = useRef(new THREE.Vector3());
  const isHovered = useRef(false);

  const sunRadius = 1.6;
  const interactionRadius = 4.0;
  const avoidanceStrength = 0.8;
  const returnSpeed = 0.05;

  // Create textures once
  const glowTexture = useMemo(() => createGlowTexture(), []);
  const blackTexture = useMemo(() => createParticleTexture('black'), []);
  const silverTexture = useMemo(() => createParticleTexture('silver'), []);
  const whiteTexture = useMemo(() => createParticleTexture('white'), []);

  // Generate particles with orbital properties
  const generateParticles = useCallback((
    count: number,
    minRadius: number,
    maxRadius: number,
    type: 'black' | 'silver' | 'white'
  ): SolarParticle[] => {
    const particles: SolarParticle[] = [];
    for (let i = 0; i < count; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = minRadius + Math.random() * (maxRadius - minRadius);

      const pos = new THREE.Vector3(
        r * Math.sin(phi) * Math.cos(theta),
        r * Math.sin(phi) * Math.sin(theta),
        r * Math.cos(phi)
      );

      // Random orbit axis for swirling effect
      const orbitAxis = new THREE.Vector3(
        Math.random() - 0.5,
        Math.random() - 0.5,
        Math.random() - 0.5
      ).normalize();

      particles.push({
        basePosition: pos.clone(),
        currentPosition: pos.clone(),
        targetPosition: pos.clone(),
        velocity: new THREE.Vector3(),
        orbitSpeed: 0.1 + Math.random() * 0.3,
        orbitAxis,
        phase: Math.random() * Math.PI * 2,
        size: 0.08 + Math.random() * 0.12,
        type,
      });
    }
    return particles;
  }, []);

  // Black particles - inner layer
  const blackParticles = useMemo(() =>
    generateParticles(60, sunRadius + 0.1, sunRadius + 1.5, 'black'),
    [generateParticles, sunRadius]
  );

  // Silver particles - mid layer (solar fissures)
  const silverParticles = useMemo(() =>
    generateParticles(40, sunRadius + 0.3, sunRadius + 2.0, 'silver'),
    [generateParticles, sunRadius]
  );

  // White particles - outer rays
  const whiteParticles = useMemo(() =>
    generateParticles(25, sunRadius + 0.5, sunRadius + 2.5, 'white'),
    [generateParticles, sunRadius]
  );

  // Solar flare particles - dramatic rays shooting out
  const solarFlares = useMemo(() => {
    const flares: SolarParticle[] = [];
    const rayCount = 8;
    const particlesPerRay = 12;

    for (let ray = 0; ray < rayCount; ray++) {
      const rayTheta = (ray / rayCount) * Math.PI * 2 + Math.random() * 0.3;
      const rayPhi = Math.PI / 2 + (Math.random() - 0.5) * 0.8;

      for (let p = 0; p < particlesPerRay; p++) {
        const t = p / particlesPerRay;
        const r = sunRadius + 0.2 + t * 3.0;
        const spread = t * 0.3;

        const pos = new THREE.Vector3(
          r * Math.sin(rayPhi + (Math.random() - 0.5) * spread) * Math.cos(rayTheta + (Math.random() - 0.5) * spread),
          r * Math.sin(rayPhi + (Math.random() - 0.5) * spread) * Math.sin(rayTheta + (Math.random() - 0.5) * spread),
          r * Math.cos(rayPhi + (Math.random() - 0.5) * spread)
        );

        flares.push({
          basePosition: pos.clone(),
          currentPosition: pos.clone(),
          targetPosition: pos.clone(),
          velocity: new THREE.Vector3(),
          orbitSpeed: 0.05 + Math.random() * 0.1,
          orbitAxis: new THREE.Vector3(0, 1, 0),
          phase: Math.random() * Math.PI * 2,
          size: 0.06 + (1 - t) * 0.1,
          type: Math.random() > 0.5 ? 'silver' : 'white',
        });
      }
    }
    return flares;
  }, [sunRadius]);

  // Create Float32Arrays for GPU
  const blackPositions = useMemo(() => new Float32Array(blackParticles.length * 3), [blackParticles]);
  const silverPositions = useMemo(() => new Float32Array(silverParticles.length * 3), [silverParticles]);
  const whitePositions = useMemo(() => new Float32Array(whiteParticles.length * 3), [whiteParticles]);
  const flarePositions = useMemo(() => new Float32Array(solarFlares.length * 3), [solarFlares]);

  // Update mouse world position
  const updateMousePosition = useCallback(() => {
    if (!groupRef.current) return;

    raycaster.setFromCamera(pointer, camera);
    const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
    raycaster.ray.intersectPlane(plane, mouseWorldPos.current);

    // Transform to group local space
    const groupWorldPos = new THREE.Vector3();
    groupRef.current.getWorldPosition(groupWorldPos);
    mouseWorldPos.current.sub(groupWorldPos);
  }, [camera, raycaster, pointer]);

  // Animate particle with swirling orbit and mouse avoidance
  const animateParticle = useCallback((
    particle: SolarParticle,
    time: number,
    deltaTime: number
  ) => {
    // Orbital swirl around base position
    const orbitAngle = time * particle.orbitSpeed + particle.phase;
    const orbitRadius = 0.15;

    // Create swirling motion
    const swirl = new THREE.Vector3(
      Math.sin(orbitAngle) * orbitRadius,
      Math.cos(orbitAngle * 1.3) * orbitRadius * 0.5,
      Math.sin(orbitAngle * 0.7) * orbitRadius * 0.8
    );

    // Apply orbit axis rotation
    const quaternion = new THREE.Quaternion();
    quaternion.setFromAxisAngle(particle.orbitAxis, orbitAngle * 0.2);
    swirl.applyQuaternion(quaternion);

    // Set target to base + swirl
    particle.targetPosition.copy(particle.basePosition).add(swirl);

    // Mouse avoidance
    if (isHovered.current) {
      const toMouse = new THREE.Vector3().subVectors(particle.currentPosition, mouseWorldPos.current);
      const distance = toMouse.length();

      if (distance < interactionRadius && distance > 0.1) {
        const avoidance = toMouse.normalize().multiplyScalar(
          avoidanceStrength * (1 - distance / interactionRadius)
        );
        particle.targetPosition.add(avoidance);
      }
    }

    // Smooth interpolation toward target
    particle.currentPosition.lerp(particle.targetPosition, returnSpeed + deltaTime * 2);

    return particle.currentPosition;
  }, [interactionRadius, avoidanceStrength, returnSpeed]);

  // Update positions array from particles
  const updatePositionsArray = useCallback((
    particles: SolarParticle[],
    positions: Float32Array,
    time: number,
    deltaTime: number
  ) => {
    for (let i = 0; i < particles.length; i++) {
      const pos = animateParticle(particles[i], time, deltaTime);
      const i3 = i * 3;
      positions[i3] = pos.x;
      positions[i3 + 1] = pos.y;
      positions[i3 + 2] = pos.z;
    }
  }, [animateParticle]);

  useFrame((state, delta) => {
    const time = state.clock.elapsedTime;

    updateMousePosition();

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

    // Update all particle systems
    updatePositionsArray(blackParticles, blackPositions, time, delta);
    updatePositionsArray(silverParticles, silverPositions, time, delta);
    updatePositionsArray(whiteParticles, whitePositions, time, delta);
    updatePositionsArray(solarFlares, flarePositions, time, delta);

    // Notify GPU of position updates
    if (blackParticlesRef.current) {
      blackParticlesRef.current.geometry.attributes.position.needsUpdate = true;
      blackParticlesRef.current.rotation.y = time * 0.02;
    }
    if (silverParticlesRef.current) {
      silverParticlesRef.current.geometry.attributes.position.needsUpdate = true;
      silverParticlesRef.current.rotation.y = -time * 0.015;
    }
    if (whiteParticlesRef.current) {
      whiteParticlesRef.current.geometry.attributes.position.needsUpdate = true;
      whiteParticlesRef.current.rotation.y = time * 0.01;
    }
    if (solarFlareRef.current) {
      solarFlareRef.current.geometry.attributes.position.needsUpdate = true;
      solarFlareRef.current.rotation.y = time * 0.008;
    }
  });

  const handlePointerEnter = useCallback(() => {
    isHovered.current = true;
  }, []);

  const handlePointerLeave = useCallback(() => {
    isHovered.current = false;
  }, []);

  return (
    <group ref={groupRef} position={position}>
      {/* Invisible interaction sphere */}
      <mesh
        onPointerEnter={handlePointerEnter}
        onPointerLeave={handlePointerLeave}
        visible={false}
      >
        <sphereGeometry args={[sunRadius + 3, 16, 16]} />
        <meshBasicMaterial transparent opacity={0} />
      </mesh>

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

      {/* Black particles - inner swirl */}
      <points ref={blackParticlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={blackParticles.length}
            array={blackPositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={blackTexture}
          size={0.18}
          transparent
          opacity={0.9}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.NormalBlending}
        />
      </points>

      {/* Silver particles - solar fissures */}
      <points ref={silverParticlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={silverParticles.length}
            array={silverPositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={silverTexture}
          size={0.14}
          transparent
          opacity={0.85}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </points>

      {/* White particles - bright rays */}
      <points ref={whiteParticlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={whiteParticles.length}
            array={whitePositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={whiteTexture}
          size={0.12}
          transparent
          opacity={0.9}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </points>

      {/* Solar flares - dramatic rays */}
      <points ref={solarFlareRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={solarFlares.length}
            array={flarePositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={silverTexture}
          size={0.1}
          transparent
          opacity={0.7}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.AdditiveBlending}
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
