import React, { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

interface ParticleFieldProps {
  count?: number;
}

const ParticleField: React.FC<ParticleFieldProps> = ({
  count = 5000
}) => {
  const coreRef = useRef<THREE.Points>(null);
  const hazeRef = useRef<THREE.Points>(null);
  const sparksRef = useRef<THREE.Points>(null);
  const ambientRef = useRef<THREE.Points>(null);

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
        colors[i3] = 0.9; colors[i3 + 1] = 0.95; colors[i3 + 2] = 1.0;
      } else if (distFromCenter > 0.5 && colorRoll < 0.15) {
        // Purple edge glow
        colors[i3] = 0.6; colors[i3 + 1] = 0.3; colors[i3 + 2] = 0.8;
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

  // HAZE - Dark nebula wisps around the black star
  const hazeData = useMemo(() => {
    const hazeCount = Math.floor(count * 0.2); // Sparser for emptier void
    const positions = new Float32Array(hazeCount * 3);
    const colors = new Float32Array(hazeCount * 3);
    const sizes = new Float32Array(hazeCount);
    const phases = new Float32Array(hazeCount);
    const basePositions = new Float32Array(hazeCount * 3);

    for (let i = 0; i < hazeCount; i++) {
      const i3 = i * 3;

      // Wider distribution - dark wisps
      const r = 3 + Math.pow(Math.random(), 0.5) * 15;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      const x = r * Math.sin(phi) * Math.cos(theta);
      const y = r * Math.sin(phi) * Math.sin(theta);
      const z = r * Math.cos(phi);

      positions[i3] = x;
      positions[i3 + 1] = y;
      positions[i3 + 2] = z;
      basePositions[i3] = x;
      basePositions[i3 + 1] = y;
      basePositions[i3 + 2] = z;

      // Mostly dark with rare dim color hints
      const colorRoll = Math.random();

      if (colorRoll < 0.05) {
        // Rare dim purple wisp
        colors[i3] = 0.25; colors[i3 + 1] = 0.15; colors[i3 + 2] = 0.35;
      } else if (colorRoll < 0.1) {
        // Rare dim blue wisp
        colors[i3] = 0.15; colors[i3 + 1] = 0.2; colors[i3 + 2] = 0.35;
      } else {
        // Dark grey wisps
        const darkness = 0.08 + Math.random() * 0.12;
        colors[i3] = darkness;
        colors[i3 + 1] = darkness;
        colors[i3 + 2] = darkness * 1.1;
      }

      // Larger, softer wisps
      const sizeRoll = Math.random();
      if (sizeRoll < 0.3) {
        sizes[i] = 0.08 + Math.random() * 0.1;
      } else if (sizeRoll < 0.7) {
        sizes[i] = 0.15 + Math.random() * 0.15;
      } else {
        sizes[i] = 0.25 + Math.random() * 0.2;
      }
      phases[i] = Math.random() * Math.PI * 2;
    }

    return { positions, colors, sizes, phases, basePositions, count: hazeCount };
  }, [count]);

  // SPARKS - Distant stars drifting outward into the void
  const sparksData = useMemo(() => {
    const sparkCount = Math.floor(count * 0.25); // Fewer for lonelier void
    const positions = new Float32Array(sparkCount * 3);
    const colors = new Float32Array(sparkCount * 3);
    const sizes = new Float32Array(sparkCount);
    const life = new Float32Array(sparkCount * 2);
    const velocities = new Float32Array(sparkCount * 3);
    const phaseOffsets = new Float32Array(sparkCount);
    const speedMultipliers = new Float32Array(sparkCount);

    for (let i = 0; i < sparkCount; i++) {
      const i3 = i * 3;

      const r = Math.random() * 1.5;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      // Mostly dim with occasional bright distant stars
      const colorRoll = Math.random();
      if (colorRoll < 0.03) {
        // Rare bright white star
        colors[i3] = 0.9; colors[i3 + 1] = 0.92; colors[i3 + 2] = 1.0;
      } else if (colorRoll < 0.06) {
        // Rare dim purple
        colors[i3] = 0.4; colors[i3 + 1] = 0.25; colors[i3 + 2] = 0.6;
      } else if (colorRoll < 0.09) {
        // Rare dim blue
        colors[i3] = 0.25; colors[i3 + 1] = 0.35; colors[i3 + 2] = 0.6;
      } else {
        // Dim grey/silver - distant stars
        const dim = 0.15 + Math.random() * 0.25;
        colors[i3] = dim;
        colors[i3 + 1] = dim;
        colors[i3 + 2] = dim * 1.1;
      }

      // Mix of sizes - mostly small distant points
      const sizeRoll = Math.random();
      if (sizeRoll < 0.5) {
        sizes[i] = 0.03 + Math.random() * 0.05; // Small distant
      } else if (sizeRoll < 0.8) {
        sizes[i] = 0.08 + Math.random() * 0.1; // Medium
      } else if (sizeRoll < 0.95) {
        sizes[i] = 0.15 + Math.random() * 0.15; // Larger
      } else {
        sizes[i] = 0.3 + Math.random() * 0.2; // Rare bright stars
      }

      life[i * 2] = Math.random() * 30;
      life[i * 2 + 1] = 30 + Math.random() * 40;

      // Velocity direction
      const vTheta = Math.random() * Math.PI * 2;
      const vPhi = Math.acos(2 * Math.random() - 1);
      velocities[i3] = Math.sin(vPhi) * Math.cos(vTheta);
      velocities[i3 + 1] = Math.sin(vPhi) * Math.sin(vTheta);
      velocities[i3 + 2] = Math.cos(vPhi);

      phaseOffsets[i] = Math.random() * 8;
      speedMultipliers[i] = 0.2 + Math.random() * 1.2; // Slower, more varied
    }

    return { positions, colors, sizes, life, velocities, phaseOffsets, speedMultipliers, count: sparkCount };
  }, [count]);

  // AMBIENT - Very distant, sparse twinkling stars in the void
  const ambientData = useMemo(() => {
    const ambientCount = Math.floor(count * 0.1); // Sparse for empty void feeling
    const positions = new Float32Array(ambientCount * 3);
    const colors = new Float32Array(ambientCount * 3);
    const sizes = new Float32Array(ambientCount);
    const twinkle = new Float32Array(ambientCount * 2);

    for (let i = 0; i < ambientCount; i++) {
      const i3 = i * 3;

      // Very far out in the void
      const r = 25 + Math.random() * 50;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);

      // Mostly dim with rare brighter stars
      const colorRoll = Math.random();
      if (colorRoll < 0.02) {
        // Rare bright distant star
        colors[i3] = 0.7; colors[i3 + 1] = 0.75; colors[i3 + 2] = 0.9;
      } else if (colorRoll < 0.05) {
        // Dim blue
        colors[i3] = 0.15; colors[i3 + 1] = 0.2; colors[i3 + 2] = 0.35;
      } else {
        // Very dim pinpoints
        const dim = 0.08 + Math.random() * 0.15;
        colors[i3] = dim; colors[i3 + 1] = dim; colors[i3 + 2] = dim * 1.1;
      }

      // Tiny distant pinpoints
      const sizeRoll = Math.random();
      if (sizeRoll < 0.7) {
        sizes[i] = 0.01 + Math.random() * 0.02;
      } else if (sizeRoll < 0.95) {
        sizes[i] = 0.03 + Math.random() * 0.04;
      } else {
        sizes[i] = 0.06 + Math.random() * 0.05;
      }
      twinkle[i * 2] = Math.random() * Math.PI * 2;
      twinkle[i * 2 + 1] = 0.3 + Math.random() * 1.5; // Slower twinkle
    }

    return { positions, colors, sizes, twinkle, count: ambientCount };
  }, [count]);

  // Geometries & Materials
  const coreGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(coreData.positions.slice(), 3));
    geo.setAttribute('color', new THREE.BufferAttribute(coreData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(coreData.sizes, 1));
    return geo;
  }, [coreData]);

  const coreMat = useMemo(() => new THREE.ShaderMaterial({
    uniforms: { time: { value: 0 } },
    vertexShader: `
      attribute float size;
      attribute vec3 color;
      varying vec3 vColor;
      varying float vAlpha;
      uniform float time;
      void main() {
        vColor = color;
        vAlpha = 0.85 + sin(time * 2.5 + position.x * 8.0) * 0.15;
        vec4 mv = modelViewMatrix * vec4(position, 1.0);
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
  }), []);

  const hazeGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(hazeData.positions.slice(), 3));
    geo.setAttribute('color', new THREE.BufferAttribute(hazeData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(hazeData.sizes, 1));
    return geo;
  }, [hazeData]);

  const hazeMat = useMemo(() => new THREE.ShaderMaterial({
    uniforms: { time: { value: 0 } },
    vertexShader: `
      attribute float size;
      attribute vec3 color;
      varying vec3 vColor;
      varying float vDist;
      void main() {
        vColor = color;
        vDist = length(position);
        vec4 mv = modelViewMatrix * vec4(position, 1.0);
        gl_PointSize = size * (380.0 / -mv.z);
        gl_Position = projectionMatrix * mv;
      }
    `,
    fragmentShader: `
      varying vec3 vColor;
      varying float vDist;
      void main() {
        float d = length(gl_PointCoord - 0.5);
        if (d > 0.5) discard;
        float fade = smoothstep(15.0, 2.0, vDist);
        float alpha = (1.0 - d * 2.0) * 0.5 * fade;
        gl_FragColor = vec4(vColor, alpha);
      }
    `,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  }), []);

  const sparksGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(sparksData.positions.slice(), 3));
    geo.setAttribute('color', new THREE.BufferAttribute(sparksData.colors, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(sparksData.sizes, 1));
    return geo;
  }, [sparksData]);

  const sparksMat = useMemo(() => new THREE.ShaderMaterial({
    uniforms: { time: { value: 0 } },
    vertexShader: `
      attribute float size;
      attribute vec3 color;
      varying vec3 vColor;
      varying float vDist;
      void main() {
        vColor = color;
        vDist = length(position);
        vec4 mv = modelViewMatrix * vec4(position, 1.0);
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
        float glow = 1.0 - smoothstep(0.0, 0.5, d);
        float alpha = glow * 0.8;
        gl_FragColor = vec4(vColor, alpha);
      }
    `,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  }), []);

  const ambientGeo = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(ambientData.positions.slice(), 3));
    geo.setAttribute('color', new THREE.BufferAttribute(ambientData.colors.slice(), 3));
    geo.setAttribute('size', new THREE.BufferAttribute(ambientData.sizes.slice(), 1));
    return geo;
  }, [ambientData]);

  const ambientMat = useMemo(() => new THREE.ShaderMaterial({
    uniforms: { time: { value: 0 } },
    vertexShader: `
      attribute float size;
      attribute vec3 color;
      varying vec3 vColor;
      uniform float time;
      void main() {
        vColor = color;
        vec4 mv = modelViewMatrix * vec4(position, 1.0);
        gl_PointSize = size * (300.0 / -mv.z);
        gl_Position = projectionMatrix * mv;
      }
    `,
    fragmentShader: `
      varying vec3 vColor;
      void main() {
        float d = length(gl_PointCoord - 0.5);
        if (d > 0.5) discard;
        float alpha = (1.0 - d * 2.0) * 0.6;
        gl_FragColor = vec4(vColor, alpha);
      }
    `,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  }), []);

  useFrame((state) => {
    const time = state.clock.elapsedTime;
    const delta = state.clock.getDelta() || 0.016;

    // Animate CORE - just gentle rotation, no breathing/pulsing
    if (coreRef.current) {
      const pos = coreRef.current.geometry.attributes.position;
      const arr = pos.array as Float32Array;

      for (let i = 0; i < coreData.count; i++) {
        const i3 = i * 3;
        let x = arr[i3], y = arr[i3 + 1], z = arr[i3 + 2];
        const dist = Math.sqrt(x * x + y * y + z * z);

        // Gentle orbital rotation only
        const orbitSpeed = 0.008 / (dist + 0.5);
        const cos = Math.cos(orbitSpeed);
        const sin = Math.sin(orbitSpeed);
        const nx = x * cos - z * sin;
        const nz = x * sin + z * cos;
        x = nx; z = nz;

        arr[i3] = x; arr[i3 + 1] = y; arr[i3 + 2] = z;
      }
      pos.needsUpdate = true;
      (coreMat.uniforms.time as any).value = time;
    }

    // Animate HAZE - very slow rotation only
    if (hazeRef.current) {
      const pos = hazeRef.current.geometry.attributes.position;
      const arr = pos.array as Float32Array;

      for (let i = 0; i < hazeData.count; i++) {
        const i3 = i * 3;
        let x = arr[i3], y = arr[i3 + 1], z = arr[i3 + 2];

        // Very slow rotation only - no oscillation
        const orbitSpeed = 0.001;
        const cos = Math.cos(orbitSpeed);
        const sin = Math.sin(orbitSpeed);
        const nx = x * cos - z * sin;
        const nz = x * sin + z * cos;
        x = nx; z = nz;

        arr[i3] = x; arr[i3 + 1] = y; arr[i3 + 2] = z;
      }
      pos.needsUpdate = true;
    }

    // Animate SPARKS - slowly drift outward, then gentle oscillation in the void
    if (sparksRef.current) {
      const pos = sparksRef.current.geometry.attributes.position;
      const arr = pos.array as Float32Array;

      const maxDistance = 45; // How far stars drift out

      for (let i = 0; i < sparksData.count; i++) {
        const i3 = i * 3;
        let x = arr[i3], y = arr[i3 + 1], z = arr[i3 + 2];
        const dist = Math.sqrt(x * x + y * y + z * z);

        const speedMult = sparksData.speedMultipliers[i];
        const phase = sparksData.phaseOffsets[i];

        if (dist < maxDistance) {
          // Slow, peaceful drift outward
          const speed = 0.08 * speedMult;
          x += sparksData.velocities[i3] * speed;
          y += sparksData.velocities[i3 + 1] * speed;
          z += sparksData.velocities[i3 + 2] * speed;
        } else {
          // Very gentle breathing oscillation
          const oscillate = Math.sin(time * 0.2 + phase) * 0.008;
          x += sparksData.velocities[i3] * oscillate;
          y += sparksData.velocities[i3 + 1] * oscillate;
          z += sparksData.velocities[i3 + 2] * oscillate;
        }

        // Very slow rotation for all particles
        const rotSpeed = 0.0008 * speedMult;
        const cos = Math.cos(rotSpeed);
        const sin = Math.sin(rotSpeed);
        const nx = x * cos - z * sin;
        const nz = x * sin + z * cos;

        arr[i3] = nx; arr[i3 + 1] = y; arr[i3 + 2] = nz;
      }
      pos.needsUpdate = true;
    }

    // Animate AMBIENT - slow twinkle in the distant void
    if (ambientRef.current) {
      const colors = ambientRef.current.geometry.attributes.color;
      const colorArr = colors.array as Float32Array;
      const sizes = ambientRef.current.geometry.attributes.size;
      const sizeArr = sizes.array as Float32Array;

      for (let i = 0; i < ambientData.count; i++) {
        const phase = ambientData.twinkle[i * 2];
        const speed = ambientData.twinkle[i * 2 + 1];

        // Subtle twinkle
        const twinkle = 0.7 + Math.sin(time * speed + phase) * 0.3;

        const i3 = i * 3;
        colorArr[i3] = ambientData.colors[i3] * twinkle;
        colorArr[i3 + 1] = ambientData.colors[i3 + 1] * twinkle;
        colorArr[i3 + 2] = ambientData.colors[i3 + 2] * twinkle;

        sizeArr[i] = ambientData.sizes[i] * (0.9 + twinkle * 0.2);
      }
      colors.needsUpdate = true;
      sizes.needsUpdate = true;

      // Very slow rotation - lonely void feeling
      ambientRef.current.rotation.y = time * 0.001;
    }
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
