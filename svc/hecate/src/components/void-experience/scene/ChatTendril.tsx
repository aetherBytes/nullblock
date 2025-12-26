import React, { useRef, useMemo, useEffect } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

interface ChatTendrilProps {
  startPos: THREE.Vector3 | React.MutableRefObject<THREE.Vector3>;
  endPos: THREE.Vector3 | React.MutableRefObject<THREE.Vector3>;
  direction: 'outgoing' | 'incoming';
  color?: THREE.Color;
  onComplete?: () => void;
  onReachTarget?: () => void;
  duration?: number;
}

// Tendril shader material - adapted from CrossroadsOrb DendriteMaterial
class TendrilMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uGrowth: { value: 0 },
        uFade: { value: 1 },
        uThickness: { value: 1.0 },
        uDirection: { value: 1.0 },
        uOpacity: { value: 1.0 },
        uColor: { value: new THREE.Color(0x4a9eff) },
      },
      vertexShader: `
        uniform float uTime;
        uniform float uGrowth;
        uniform float uThickness;
        uniform float uDirection;
        varying vec2 vUv;
        varying float vProgress;
        varying float vTaper;
        varying vec3 vNormal;

        void main() {
          vUv = uv;
          vProgress = uv.y;
          vNormal = normalize(normalMatrix * normal);

          vec3 pos = position;

          float taperProgress = uDirection > 0.0 ? vProgress : 1.0 - vProgress;
          float taperCurve = pow(taperProgress, 0.6);
          vTaper = taperCurve;

          float taperAmount = 0.08 + 0.92 * taperCurve;
          pos.x *= taperAmount * uThickness;
          pos.z *= taperAmount * uThickness;

          float thinness = 1.0 - taperProgress;
          float tipPinning = smoothstep(0.0, 0.15, taperProgress);
          float basePinning = smoothstep(0.0, 0.15, thinness);
          float waveStrength = thinness * thinness * tipPinning * basePinning;

          float wave1 = sin(thinness * 5.0 + uTime * 2.5) * 0.06 * waveStrength;
          float wave2 = sin(thinness * 8.0 - uTime * 3.5) * 0.04 * waveStrength;
          pos.x += wave1;
          pos.z += wave2;

          float spiral = sin(thinness * 4.0 + uTime * 1.5) * 0.02 * waveStrength;
          pos.x += cos(uTime * 2.0) * spiral;
          pos.z += sin(uTime * 2.0) * spiral;

          gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
        }
      `,
      fragmentShader: `
        uniform float uTime;
        uniform float uGrowth;
        uniform float uFade;
        uniform float uThickness;
        uniform float uDirection;
        uniform float uOpacity;
        uniform vec3 uColor;
        varying vec2 vUv;
        varying float vProgress;
        varying float vTaper;
        varying vec3 vNormal;

        vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }

        float snoise(vec2 v) {
          const vec4 C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
          vec2 i = floor(v + dot(v, C.yy));
          vec2 x0 = v - i + dot(i, C.xx);
          vec2 i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
          vec4 x12 = x0.xyxy + C.xxzz;
          x12.xy -= i1;
          i = mod289(i);
          vec3 p = permute(permute(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0));
          vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
          m = m*m; m = m*m;
          vec3 x = 2.0 * fract(p * C.www) - 1.0;
          vec3 h = abs(x) - 0.5;
          vec3 ox = floor(x + 0.5);
          vec3 a0 = x - ox;
          m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);
          vec3 g;
          g.x = a0.x * x0.x + h.x * x0.y;
          g.yz = a0.yz * x12.xz + h.yz * x12.yw;
          return 130.0 * dot(m, g);
        }

        void main() {
          float time = uTime * 0.08;

          float growthFront;
          bool shouldDiscard;

          if (uDirection > 0.0) {
            float threshold = 1.0 - uGrowth;
            shouldDiscard = vProgress < threshold;
            growthFront = threshold;
          } else {
            float threshold = uGrowth;
            shouldDiscard = vProgress > threshold;
            growthFront = threshold;
          }

          if (shouldDiscard) discard;

          vec3 viewDir = vec3(0.0, 0.0, 1.0);
          float fresnel = 1.0 - abs(dot(vNormal, viewDir));
          fresnel = pow(fresnel, 2.0);

          float pulseDir = uDirection > 0.0 ? 1.0 : -1.0;
          vec2 flowUV = vec2(vProgress * 3.0, vUv.x * 1.5);

          float noise1 = snoise(flowUV * 6.0 + vec2(time * 0.2 * pulseDir, time * 0.08));
          float noise2 = snoise(flowUV * 12.0 + vec2(-time * 0.15, time * 0.12) - noise1 * 0.2);
          float noise3 = snoise(flowUV * 24.0 + vec2(time * 0.1, -time * 0.06));

          float wisp = (noise1 * 0.5 + noise2 * 0.35 + noise3 * 0.15) * 0.5 + 0.5;
          wisp = smoothstep(0.15, 0.85, wisp);

          float shimmer = snoise(flowUV * 40.0 + vec2(uTime * 0.8, uTime * 0.6));
          shimmer = pow(max(shimmer, 0.0), 3.0) * 0.3;

          float pulse = 0.85 + 0.15 * sin(uTime * 0.3 + vProgress * 4.0 * pulseDir);

          float tipDist = abs(vProgress - growthFront);
          float tipGlow = 1.0 - smoothstep(0.0, 0.2, tipDist);
          tipGlow = pow(tipGlow, 1.5);

          float edgeFade = 1.0 - fresnel * 0.6;
          float taperBrightness = 0.6 + vTaper * 0.4;

          float intensity = wisp * pulse * taperBrightness * edgeFade;
          intensity += shimmer * wisp;
          intensity += tipGlow * 0.6;
          intensity *= 1.2; // Increased from 0.7 for better visibility
          intensity *= uFade;
          intensity *= uOpacity;

          // Use the provided color with subtle tinting
          vec3 color = uColor;
          color += vec3(0.1, 0.08, 0.06) * fresnel;
          color += uColor * 0.3 * tipGlow;
          color *= intensity;

          float alpha = intensity * 0.75;

          gl_FragColor = vec4(color, alpha);
        }
      `,
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
    });
  }
}

// Helper to get Vector3 from either direct value or ref
const getPosition = (pos: THREE.Vector3 | React.MutableRefObject<THREE.Vector3>): THREE.Vector3 => {
  return 'current' in pos ? pos.current : pos;
};

const ChatTendril: React.FC<ChatTendrilProps> = ({
  startPos,
  endPos,
  direction,
  color,
  onComplete,
  onReachTarget,
  duration = 2.0,
}) => {
  const meshRef = useRef<THREE.Mesh>(null);
  const stateRef = useRef<{
    state: 'growing' | 'holding' | 'fading' | 'complete';
    growth: number;
    fade: number;
    elapsed: number;
  }>({
    state: 'growing',
    growth: 0,
    fade: 1,
    elapsed: 0,
  });

  const tendrilColor = useMemo(() => {
    if (color) return color;
    return direction === 'outgoing'
      ? new THREE.Color(0x4a9eff)  // Steel blue for user messages
      : new THREE.Color(0xffffff); // Bright white for HECATE responses
  }, [color, direction]);

  // Create material once
  const material = useMemo(() => new TendrilMaterial(), []);

  // Create unit-length geometry once - we'll scale it each frame
  const geometry = useMemo(() => {
    // Unit cylinder along Y axis, will be scaled to match distance
    // Thin tendril (0.03 radius) for elegant beam effect
    return new THREE.CylinderGeometry(0.03, 0.03, 1, 8, 32, true);
  }, []);

  // Reusable objects for frame updates (avoid GC)
  const tempVec = useMemo(() => new THREE.Vector3(), []);
  const tempQuat = useMemo(() => new THREE.Quaternion(), []);
  const upVec = useMemo(() => new THREE.Vector3(0, 1, 0), []);

  // Initialize material with direction and color
  useEffect(() => {
    // Both directions grow from startPos to endPos
    // VoidExperience.tsx already swaps positions based on direction
    material.uniforms.uDirection.value = -1.0;
    material.uniforms.uColor.value = tendrilColor;
  }, [material, direction, tendrilColor]);

  useFrame((_, delta) => {
    if (!meshRef.current) return;

    const state = stateRef.current;
    state.elapsed += delta;

    // Get current positions (supports both static and ref-based positions)
    const currentStart = getPosition(startPos);
    const currentEnd = getPosition(endPos);

    // Update mesh transform to track moving targets
    tempVec.subVectors(currentEnd, currentStart);
    const length = tempVec.length();

    if (length > 0.01) {
      // Update position to midpoint
      meshRef.current.position.lerpVectors(currentStart, currentEnd, 0.5);

      // Update rotation to face direction
      tempVec.normalize();
      tempQuat.setFromUnitVectors(upVec, tempVec);
      meshRef.current.quaternion.copy(tempQuat);

      // Update scale to match distance (geometry is unit length)
      meshRef.current.scale.set(1, length, 1);
    }

    // Update time uniform for wave animation
    material.uniforms.uTime.value = state.elapsed;

    const growSpeed = 1 / duration; // Complete in 'duration' seconds
    const fadeSpeed = 2.0; // Faster fade out

    switch (state.state) {
      case 'growing':
        state.growth = Math.min(1.0, state.growth + delta * growSpeed);
        material.uniforms.uGrowth.value = state.growth;

        if (state.growth >= 1.0) {
          state.state = 'holding';
          state.elapsed = 0; // Reset for hold timer
          onReachTarget?.(); // Trigger when tendril reaches target
        }
        break;

      case 'holding':
        // Hold at full growth (0.8s)
        if (state.elapsed > 0.8) {
          state.state = 'fading';
        }
        break;

      case 'fading':
        state.fade = Math.max(0, state.fade - delta * fadeSpeed * 0.6);
        material.uniforms.uFade.value = state.fade;

        if (state.fade <= 0) {
          state.state = 'complete';
          onComplete?.();
        }
        break;

      case 'complete':
        // Do nothing, waiting to be unmounted
        break;
    }
  });

  if (stateRef.current.state === 'complete') {
    return null;
  }

  return (
    <mesh ref={meshRef} geometry={geometry} material={material} />
  );
};

export default ChatTendril;
