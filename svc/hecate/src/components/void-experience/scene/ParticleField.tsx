import { useFrame } from '@react-three/fiber';
import React, { useRef, useMemo } from 'react';
import * as THREE from 'three';

interface ParticleFieldProps {
  count?: number;
}

const ParticleField: React.FC<ParticleFieldProps> = ({ count = 5000 }) => {
  const coreRef = useRef<THREE.Points>(null);
  const hazeRef = useRef<THREE.Points>(null);
  const sparksRef = useRef<THREE.Points>(null);
  const ambientRef = useRef<THREE.Points>(null);

  // Shared time refs for shader uniforms
  const coreTimeRef = useRef({ value: 0 });
  const hazeTimeRef = useRef({ value: 0 });
  const sparksTimeRef = useRef({ value: 0 });
  const ambientTimeRef = useRef({ value: 0 });

  // CORE - Dense dark star center with intense edge glow
  const coreData = useMemo(() => {
    const coreCount = Math.floor(count * 0.4);
    const positions = new Float32Array(coreCount * 3);
    const colors = new Float32Array(coreCount * 3);
    const sizes = new Float32Array(coreCount);

    for (let i = 0; i < coreCount; i++) {
      const i3 = i * 3;

      // Tighter core with dense edge corona
      const r = Math.pow(Math.random(), 0.7) * 2.5;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      // Dark star colors - mostly dark with occasional bright edge flares
      const distFromCenter = r / 2.5;
      const colorRoll = Math.random();

      if (distFromCenter > 0.7 && colorRoll < 0.3) {
        // Edge corona - bright white/blue flares
        colors[i3] = 0.9;
        colors[i3 + 1] = 0.95;
        colors[i3 + 2] = 1.0;
      } else if (distFromCenter > 0.5 && colorRoll < 0.15) {
        // Purple edge glow
        colors[i3] = 0.6;
        colors[i3 + 1] = 0.3;
        colors[i3 + 2] = 0.8;
      } else {
        // Dark core - very dim
        const darkness = 0.05 + Math.random() * 0.15;

        colors[i3] = darkness;
        colors[i3 + 1] = darkness;
        colors[i3 + 2] = darkness * 1.2;
      }

      // Varied sizes
      const sizeRoll = Math.random();

      if (sizeRoll < 0.4) {
        sizes[i] = 0.02 + Math.random() * 0.03;
      } else if (sizeRoll < 0.8) {
        sizes[i] = 0.05 + Math.random() * 0.06;
      } else {
        sizes[i] = 0.1 + Math.random() * 0.08;
      }
    }

    return { positions, colors, sizes, count: coreCount };
  }, [count]);

  // HAZE - Cosmic nebula waves emanating from the black star
  const hazeData = useMemo(() => {
    const hazeCount = Math.floor(count * 0.5); // More density for smaller particles
    const positions = new Float32Array(hazeCount * 3);
    const colors = new Float32Array(hazeCount * 3);
    const sizes = new Float32Array(hazeCount);
    const phases = new Float32Array(hazeCount);
    const basePositions = new Float32Array(hazeCount * 3);
    const waveOffsets = new Float32Array(hazeCount); // For wave animation

    for (let i = 0; i < hazeCount; i++) {
      const i3 = i * 3;

      // Distribute in tendrils/arms radiating outward
      const arm = Math.floor(Math.random() * 5); // 5 nebula arms
      const armAngle = (arm / 5) * Math.PI * 2;
      const spread = Math.random() * 0.8; // How much it spreads from arm

      const r = 2 + Math.pow(Math.random(), 0.4) * 18;
      const theta = armAngle + spread + (Math.random() - 0.5) * 0.5;
      const phi = Math.PI * 0.5 + (Math.random() - 0.5) * 1.2; // Flatten toward equator

      const x = r * Math.sin(phi) * Math.cos(theta);
      const y = r * Math.sin(phi) * Math.sin(theta) * 0.4; // Flatten vertically
      const z = r * Math.cos(phi);

      positions[i3] = x;
      positions[i3 + 1] = y;
      positions[i3 + 2] = z;
      basePositions[i3] = x;
      basePositions[i3 + 1] = y;
      basePositions[i3 + 2] = z;

      // Cosmic colors - purples, blues, with hints of pink
      const colorRoll = Math.random();
      const distFactor = r / 20;

      if (colorRoll < 0.25) {
        // Deep purple nebula
        colors[i3] = 0.3 + distFactor * 0.2;
        colors[i3 + 1] = 0.1 + distFactor * 0.1;
        colors[i3 + 2] = 0.5 + distFactor * 0.3;
      } else if (colorRoll < 0.45) {
        // Blue cosmic gas
        colors[i3] = 0.15 + distFactor * 0.1;
        colors[i3 + 1] = 0.25 + distFactor * 0.2;
        colors[i3 + 2] = 0.55 + distFactor * 0.3;
      } else if (colorRoll < 0.55) {
        // Pink/magenta wisps
        colors[i3] = 0.45 + distFactor * 0.2;
        colors[i3 + 1] = 0.15 + distFactor * 0.1;
        colors[i3 + 2] = 0.4 + distFactor * 0.2;
      } else {
        // Dark cosmic dust
        const darkness = 0.1 + Math.random() * 0.15 + distFactor * 0.1;

        colors[i3] = darkness;
        colors[i3 + 1] = darkness * 0.9;
        colors[i3 + 2] = darkness * 1.3;
      }

      // Smaller sizes with more density
      const sizeRoll = Math.random();

      if (sizeRoll < 0.4) {
        sizes[i] = 0.04 + Math.random() * 0.05;
      } else if (sizeRoll < 0.8) {
        sizes[i] = 0.08 + Math.random() * 0.08;
      } else {
        sizes[i] = 0.14 + Math.random() * 0.1;
      }

      phases[i] = Math.random() * Math.PI * 2;
      waveOffsets[i] = Math.random() * Math.PI * 2;
    }

    return { positions, colors, sizes, phases, basePositions, waveOffsets, count: hazeCount };
  }, [count]);

  // SPARKS - Energy particles flowing outward then orbiting
  const sparksData = useMemo(() => {
    const sparkCount = Math.floor(count * 0.4); // More density
    const positions = new Float32Array(sparkCount * 3);
    const colors = new Float32Array(sparkCount * 3);
    const sizes = new Float32Array(sparkCount);
    const velocities = new Float32Array(sparkCount * 3);
    const phaseOffsets = new Float32Array(sparkCount);
    const speedMultipliers = new Float32Array(sparkCount);
    const targetRadius = new Float32Array(sparkCount); // Where each particle settles

    for (let i = 0; i < sparkCount; i++) {
      const i3 = i * 3;

      const r = Math.random() * 2;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      // Cosmic energy colors
      const colorRoll = Math.random();

      if (colorRoll < 0.15) {
        // Bright cosmic white
        colors[i3] = 0.85;
        colors[i3 + 1] = 0.9;
        colors[i3 + 2] = 1.0;
      } else if (colorRoll < 0.35) {
        // Purple energy
        colors[i3] = 0.5;
        colors[i3 + 1] = 0.3;
        colors[i3 + 2] = 0.8;
      } else if (colorRoll < 0.5) {
        // Blue energy
        colors[i3] = 0.3;
        colors[i3 + 1] = 0.5;
        colors[i3 + 2] = 0.9;
      } else if (colorRoll < 0.6) {
        // Pink energy
        colors[i3] = 0.7;
        colors[i3 + 1] = 0.35;
        colors[i3 + 2] = 0.6;
      } else {
        // Silver cosmic dust
        const dim = 0.25 + Math.random() * 0.35;

        colors[i3] = dim;
        colors[i3 + 1] = dim;
        colors[i3 + 2] = dim * 1.15;
      }

      // Smaller sizes for less polka-dot look
      const sizeRoll = Math.random();

      if (sizeRoll < 0.5) {
        sizes[i] = 0.018 + Math.random() * 0.025;
      } else if (sizeRoll < 0.8) {
        sizes[i] = 0.04 + Math.random() * 0.045;
      } else if (sizeRoll < 0.95) {
        sizes[i] = 0.08 + Math.random() * 0.05;
      } else {
        sizes[i] = 0.12 + Math.random() * 0.06;
      }

      // Velocity direction
      const vTheta = Math.random() * Math.PI * 2;
      const vPhi = Math.acos(2 * Math.random() - 1);

      velocities[i3] = Math.sin(vPhi) * Math.cos(vTheta);
      velocities[i3 + 1] = Math.sin(vPhi) * Math.sin(vTheta);
      velocities[i3 + 2] = Math.cos(vPhi);

      phaseOffsets[i] = Math.random() * Math.PI * 2;
      speedMultipliers[i] = 0.3 + Number(Math.random());
      // Each particle has a target radius it settles at (stays on canvas)
      targetRadius[i] = 8 + Math.random() * 14; // 8-22 range
    }

    return {
      positions,
      colors,
      sizes,
      velocities,
      phaseOffsets,
      speedMultipliers,
      targetRadius,
      count: sparkCount,
    };
  }, [count]);

  // AMBIENT - Background starfield that stays visible
  const ambientData = useMemo(() => {
    const ambientCount = Math.floor(count * 0.3); // More density
    const positions = new Float32Array(ambientCount * 3);
    const colors = new Float32Array(ambientCount * 3);
    const sizes = new Float32Array(ambientCount);
    const twinkle = new Float32Array(ambientCount * 2);

    for (let i = 0; i < ambientCount; i++) {
      const i3 = i * 3;

      // Distributed throughout visible range
      const r = 12 + Math.random() * 25; // Closer so they're visible
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      // More visible colors
      const colorRoll = Math.random();

      if (colorRoll < 0.1) {
        // Bright white stars
        colors[i3] = 0.8;
        colors[i3 + 1] = 0.85;
        colors[i3 + 2] = 1.0;
      } else if (colorRoll < 0.25) {
        // Blue-white stars
        colors[i3] = 0.5;
        colors[i3 + 1] = 0.6;
        colors[i3 + 2] = 0.9;
      } else if (colorRoll < 0.35) {
        // Purple tinted
        colors[i3] = 0.55;
        colors[i3 + 1] = 0.4;
        colors[i3 + 2] = 0.75;
      } else {
        // Silver/white - varied brightness
        const brightness = 0.3 + Math.random() * 0.4;

        colors[i3] = brightness;
        colors[i3 + 1] = brightness;
        colors[i3 + 2] = brightness * 1.1;
      }

      // Smaller star sizes with more density
      const sizeRoll = Math.random();

      if (sizeRoll < 0.5) {
        sizes[i] = 0.01 + Math.random() * 0.018;
      } else if (sizeRoll < 0.8) {
        sizes[i] = 0.025 + Math.random() * 0.03;
      } else if (sizeRoll < 0.95) {
        sizes[i] = 0.05 + Math.random() * 0.035;
      } else {
        sizes[i] = 0.07 + Math.random() * 0.04; // Brighter stars
      }

      twinkle[i * 2] = Math.random() * Math.PI * 2;
      twinkle[i * 2 + 1] = 0.4 + Math.random() * 1.2;
    }

    return { positions, colors, sizes, twinkle, count: ambientCount };
  }, [count]);

  // Geometries & Materials (removed .slice() - buffers updated each frame anyway)
  const coreGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();

    geo.setAttribute('position', new THREE.BufferAttribute(coreData.positions, 3));
    geo.setAttribute('color', new THREE.BufferAttribute(coreData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(coreData.sizes, 1));

    return geo;
  }, [coreData]);

  const coreMat = useMemo(
    () =>
      new THREE.ShaderMaterial({
        uniforms: { time: coreTimeRef.current },
        vertexShader: `
      attribute float size;
      attribute vec3 color;
      varying vec3 vColor;
      varying float vAlpha;
      uniform float time;
      void main() {
        vColor = color;

        // GPU orbital rotation
        float dist = length(position);
        float orbitSpeed = 0.008 / (dist + 0.5);
        float angle = time * orbitSpeed;
        float cosA = cos(angle);
        float sinA = sin(angle);
        vec3 rotatedPos = vec3(
          position.x * cosA - position.z * sinA,
          position.y,
          position.x * sinA + position.z * cosA
        );

        vAlpha = 0.85 + sin(time * 2.5 + rotatedPos.x * 8.0) * 0.15;
        vec4 mv = modelViewMatrix * vec4(rotatedPos, 1.0);
        gl_PointSize = size * (420.0 / -mv.z);
        gl_Position = projectionMatrix * mv;
      }
    `,
        fragmentShader: `
      varying vec3 vColor;
      varying float vAlpha;
      void main() {
        float d = length(gl_PointCoord - 0.5);
        if (d > 0.5) discard;
        float alpha = (1.0 - d * 2.0) * vAlpha;
        gl_FragColor = vec4(vColor, alpha);
      }
    `,
        transparent: true,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      }),
    [],
  );

  const hazeGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();

    geo.setAttribute('position', new THREE.BufferAttribute(hazeData.basePositions, 3));
    geo.setAttribute('color', new THREE.BufferAttribute(hazeData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(hazeData.sizes, 1));
    geo.setAttribute('waveOffset', new THREE.BufferAttribute(hazeData.waveOffsets, 1));

    return geo;
  }, [hazeData]);

  const hazeMat = useMemo(
    () =>
      new THREE.ShaderMaterial({
        uniforms: { time: hazeTimeRef.current },
        vertexShader: `
      attribute float size;
      attribute vec3 color;
      attribute float waveOffset;
      varying vec3 vColor;
      varying float vDist;
      uniform float time;
      void main() {
        vColor = color;

        // GPU wave animation
        float dist = length(position);
        float wave = sin(time * 0.3 - dist * 0.15 + waveOffset) * 0.8;
        float breathe = sin(time * 0.15 + waveOffset) * 0.3;

        // Direction from center
        vec3 dir = position / (dist + 0.1);

        // Apply wave displacement
        vec3 animPos = position + dir * wave + dir * breathe;
        animPos.y = position.y + dir.y * wave * 0.3 + dir.y * breathe * 0.3;

        // Slow rotation
        float rotSpeed = 0.002 * time;
        float cosR = cos(rotSpeed);
        float sinR = sin(rotSpeed);
        vec3 rotatedPos = vec3(
          animPos.x * cosR - animPos.z * sinR,
          animPos.y,
          animPos.x * sinR + animPos.z * cosR
        );

        vDist = length(rotatedPos);
        vec4 mv = modelViewMatrix * vec4(rotatedPos, 1.0);
        gl_PointSize = size * (600.0 / -mv.z);
        gl_Position = projectionMatrix * mv;
      }
    `,
        fragmentShader: `
      varying vec3 vColor;
      varying float vDist;
      void main() {
        float d = length(gl_PointCoord - 0.5);
        if (d > 0.5) discard;
        // Very soft gaussian-like falloff for nebula effect
        float falloff = exp(-d * d * 8.0);
        float alpha = falloff * 0.35;
        gl_FragColor = vec4(vColor * 1.1, alpha);
      }
    `,
        transparent: true,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      }),
    [],
  );

  const sparksGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();

    geo.setAttribute('position', new THREE.BufferAttribute(sparksData.positions, 3));
    geo.setAttribute('color', new THREE.BufferAttribute(sparksData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(sparksData.sizes, 1));
    geo.setAttribute('velocity', new THREE.BufferAttribute(sparksData.velocities, 3));
    geo.setAttribute('phaseOffset', new THREE.BufferAttribute(sparksData.phaseOffsets, 1));
    geo.setAttribute('speedMult', new THREE.BufferAttribute(sparksData.speedMultipliers, 1));
    geo.setAttribute('targetRadius', new THREE.BufferAttribute(sparksData.targetRadius, 1));

    return geo;
  }, [sparksData]);

  const sparksMat = useMemo(
    () =>
      new THREE.ShaderMaterial({
        uniforms: { time: sparksTimeRef.current },
        vertexShader: `
      attribute float size;
      attribute vec3 color;
      attribute vec3 velocity;
      attribute float phaseOffset;
      attribute float speedMult;
      attribute float targetRadius;
      varying vec3 vColor;
      varying float vDist;
      uniform float time;
      void main() {
        vColor = color;

        // GPU expansion + orbit animation
        // Calculate how far particle should have expanded
        float expandTime = time * 0.06 * speedMult;
        vec3 expandedPos = position + velocity * expandTime;
        float dist = length(expandedPos);

        // Clamp to target radius (particle stops expanding when it reaches target)
        if (dist > targetRadius) {
          expandedPos = normalize(expandedPos) * targetRadius;
          dist = targetRadius;

          // At target - gentle wave motion
          float wave = sin(time * 0.25 - dist * 0.1 + phaseOffset) * 0.15;
          vec3 dir = expandedPos / (dist + 0.1);
          expandedPos += dir * wave;
          expandedPos.y += dir.y * wave * 0.5;
        }

        // Slow orbital rotation
        float rotSpeed = 0.0015 * speedMult * time;
        float cosR = cos(rotSpeed);
        float sinR = sin(rotSpeed);
        vec3 rotatedPos = vec3(
          expandedPos.x * cosR - expandedPos.z * sinR,
          expandedPos.y,
          expandedPos.x * sinR + expandedPos.z * cosR
        );

        vDist = length(rotatedPos);
        vec4 mv = modelViewMatrix * vec4(rotatedPos, 1.0);
        gl_PointSize = size * (550.0 / -mv.z);
        gl_Position = projectionMatrix * mv;
      }
    `,
        fragmentShader: `
      varying vec3 vColor;
      varying float vDist;
      void main() {
        float d = length(gl_PointCoord - 0.5);
        if (d > 0.5) discard;
        // Soft diffuse glow - not a hard circle
        float falloff = exp(-d * d * 6.0);
        float alpha = falloff * 0.5;
        gl_FragColor = vec4(vColor, alpha);
      }
    `,
        transparent: true,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      }),
    [],
  );

  const ambientGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();

    geo.setAttribute('position', new THREE.BufferAttribute(ambientData.positions, 3));
    geo.setAttribute('color', new THREE.BufferAttribute(ambientData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(ambientData.sizes, 1));
    geo.setAttribute('twinkle', new THREE.BufferAttribute(ambientData.twinkle, 2));

    return geo;
  }, [ambientData]);

  const ambientMat = useMemo(
    () =>
      new THREE.ShaderMaterial({
        uniforms: { time: ambientTimeRef.current },
        vertexShader: `
      attribute float size;
      attribute vec3 color;
      attribute vec2 twinkle; // x = phase, y = speed
      varying vec3 vColor;
      varying float vTwinkle;
      uniform float time;
      void main() {
        // GPU twinkling
        float twinkleVal = 0.75 + sin(time * twinkle.y + twinkle.x) * 0.25;
        vColor = color * twinkleVal;
        vTwinkle = twinkleVal;

        // Slow rotation around origin
        float rotAngle = time * 0.002;
        float cosR = cos(rotAngle);
        float sinR = sin(rotAngle);
        vec3 rotatedPos = vec3(
          position.x * cosR - position.z * sinR,
          position.y,
          position.x * sinR + position.z * cosR
        );

        float animSize = size * (0.85 + twinkleVal * 0.3);
        vec4 mv = modelViewMatrix * vec4(rotatedPos, 1.0);
        gl_PointSize = animSize * (400.0 / -mv.z);
        gl_Position = projectionMatrix * mv;
      }
    `,
        fragmentShader: `
      varying vec3 vColor;
      varying float vTwinkle;
      void main() {
        float d = length(gl_PointCoord - 0.5);
        if (d > 0.5) discard;
        // Stars: bright center with soft halo
        float core = exp(-d * d * 25.0); // Tight bright center
        float halo = exp(-d * d * 4.0) * 0.4; // Soft outer glow
        float alpha = core + halo;
        gl_FragColor = vec4(vColor * (1.0 + core * 0.5), alpha * 0.7);
      }
    `,
        transparent: true,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      }),
    [],
  );

  // GPU-based animation: just update time uniforms, all calculations happen in shaders
  useFrame((state) => {
    const time = state.clock.elapsedTime;

    coreTimeRef.current.value = time;
    hazeTimeRef.current.value = time;
    sparksTimeRef.current.value = time;
    ambientTimeRef.current.value = time;
  });

  return (
    <group>
      <points ref={ambientRef} geometry={ambientGeo} material={ambientMat} />
      <points ref={hazeRef} geometry={hazeGeo} material={hazeMat} />
      <points ref={sparksRef} geometry={sparksGeo} material={sparksMat} />
      <points ref={coreRef} geometry={coreGeo} material={coreMat} />
    </group>
  );
};

export default ParticleField;
