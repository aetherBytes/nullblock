import React, { useRef, useMemo, useCallback } from 'react';
import { useFrame, useThree, extend } from '@react-three/fiber';
import * as THREE from 'three';

// Animated sun surface shader on SPHERE geometry - no edges!
class SunSurfaceMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uHover: { value: 0 },
      },
      vertexShader: `
        varying vec3 vNormal;
        varying vec3 vPosition;
        varying vec2 vUv;

        void main() {
          vNormal = normalize(normalMatrix * normal);
          vPosition = position;
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform float uTime;
        uniform float uHover;
        varying vec3 vNormal;
        varying vec3 vPosition;
        varying vec2 vUv;

        // Simplex-like noise
        vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }

        float snoise(vec2 v) {
          const vec4 C = vec4(0.211324865405187, 0.366025403784439,
                             -0.577350269189626, 0.024390243902439);
          vec2 i  = floor(v + dot(v, C.yy));
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

        // FBM (Fractal Brownian Motion) for layered noise
        float fbm(vec2 p) {
          float value = 0.0;
          float amplitude = 0.5;
          float frequency = 1.0;
          for (int i = 0; i < 6; i++) {
            value += amplitude * snoise(p * frequency);
            amplitude *= 0.5;
            frequency *= 2.0;
          }
          return value;
        }

        void main() {
          float time = uTime * 0.15;

          // Use spherical coordinates from the vertex position
          float theta = atan(vPosition.z, vPosition.x);
          float phi = acos(vPosition.y / length(vPosition));
          vec2 sphereUV = vec2(theta / 6.28318 + 0.5, phi / 3.14159);

          // Multiple noise layers
          float noise1 = fbm(sphereUV * 4.0 + vec2(time * 0.3, -time * 0.1));
          float noise2 = fbm(sphereUV * 8.0 + vec2(-time * 0.5, time * 0.2));
          float noise3 = snoise(sphereUV * 16.0 + vec2(time * 0.8, -time * 0.4));
          float noise4 = fbm(sphereUV * 2.0 + vec2(time * 0.1, time * 0.15));

          float combinedNoise = noise1 * 0.4 + noise2 * 0.3 + noise3 * 0.2 + noise4 * 0.1;

          // === BLACK HOLE VOID EFFECT ===
          // Fresnel for rim detection
          float fresnel = 1.0 - abs(dot(vNormal, vec3(0.0, 0.0, 1.0)));

          // DEEP VOID CENTER - almost pure black with subtle turbulence
          // The center should feel like an abyss
          float voidDepth = 1.0 - fresnel; // 1 at center, 0 at edge
          voidDepth = pow(voidDepth, 0.8);

          // Very subtle dark turbulence in the void
          vec3 voidColor = vec3(0.0); // Pure black base
          float voidNoise = (noise1 * 0.5 + noise3 * 0.3) * 0.5 + 0.5;
          voidColor += vec3(0.015, 0.008, 0.01) * voidNoise * voidDepth;

          // === ESCAPING LIGHT RIM ===
          // Multiple rim layers for depth
          float innerRim = pow(fresnel, 4.0); // Tight bright edge
          float midRim = pow(fresnel, 2.0);   // Broader glow
          float outerRim = pow(fresnel, 1.2); // Soft outer halo

          // Noise variation on the rim - like turbulent escaping light
          float rimNoise = 0.6 + noise1 * 0.3 + noise2 * 0.2;
          float flareNoise = smoothstep(0.3, 0.8, noise1 + noise3 * 0.5);

          // Colors - bright white/warm escaping light
          vec3 brightWhite = vec3(1.0, 0.98, 0.95);
          vec3 warmLight = vec3(1.0, 0.9, 0.75);
          vec3 hotCore = vec3(1.0, 0.85, 0.7);

          // Build the rim glow
          vec3 rimColor = vec3(0.0);

          // Inner burning edge - very bright
          rimColor += hotCore * innerRim * rimNoise * 2.0;

          // Mid corona
          rimColor += warmLight * midRim * rimNoise * 0.8;

          // Outer soft glow
          rimColor += brightWhite * outerRim * 0.3;

          // Add flare bursts - brighter spots on the rim
          rimColor += brightWhite * flareNoise * innerRim * 1.5;

          // === COMBINE ===
          // Void in center, escaping light at edges
          vec3 color = voidColor + rimColor;

          // Subtle pulsing
          float pulse = 1.0 + sin(uTime * 0.5) * 0.08;
          color *= pulse;

          // Occasional bright flash
          float flash = pow(sin(uTime * 0.3 + noise1 * 3.0) * 0.5 + 0.5, 8.0);
          color += brightWhite * flash * innerRim * 0.3;

          gl_FragColor = vec4(color, 1.0);
        }
      `,
      transparent: false,
      depthWrite: true,
      side: THREE.FrontSide,
    });
  }
}

// Corona RING shader - for torus geometry
class CoronaRingMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uIntensity: { value: 1.0 },
      },
      vertexShader: `
        varying vec3 vNormal;
        varying vec2 vUv;
        varying vec3 vViewDir;

        void main() {
          vNormal = normalize(normalMatrix * normal);
          vUv = uv;
          vec4 worldPos = modelMatrix * vec4(position, 1.0);
          vViewDir = normalize(cameraPosition - worldPos.xyz);
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform float uTime;
        uniform float uIntensity;
        varying vec3 vNormal;
        varying vec2 vUv;
        varying vec3 vViewDir;

        // Noise
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
          float time = uTime * 0.1;

          // Fresnel for glow at edges
          float fresnel = 1.0 - abs(dot(vNormal, vViewDir));
          fresnel = pow(fresnel, 1.5);

          // Flowing noise along the ring
          vec2 flowUV = vUv + vec2(time * 0.3, time * 0.1);
          float noise1 = snoise(flowUV * 8.0);
          float noise2 = snoise(flowUV * 16.0 - noise1 * 0.3);

          // Combine
          float wisp = (noise1 * 0.6 + noise2 * 0.4) * 0.5 + 0.5;
          wisp = smoothstep(0.3, 0.7, wisp);

          // Pulse
          float pulse = 0.8 + 0.2 * sin(uTime * 0.5 + vUv.x * 6.28);

          // Final intensity
          float intensity = (0.5 + fresnel * 0.5) * wisp * pulse * uIntensity;

          // Color - bright white/warm
          vec3 color = vec3(1.0, 0.95, 0.9) * intensity;

          // Alpha
          float alpha = intensity * 0.8;

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

// Solar flare shader - elongated rays
class SolarFlareMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uFlarePhase: { value: 0 },
      },
      vertexShader: `
        varying vec2 vUv;
        void main() {
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform float uTime;
        uniform float uFlarePhase;
        varying vec2 vUv;

        void main() {
          // Flare shape - bright at base, fading toward tip
          float distFromBase = vUv.y;
          float distFromCenter = abs(vUv.x - 0.5) * 2.0;

          // Taper toward tip
          float taper = 1.0 - distFromBase;
          float width = 0.3 * taper + 0.05;

          // Core brightness
          float core = smoothstep(width, 0.0, distFromCenter);
          core *= (1.0 - distFromBase * 0.8);

          // Animated intensity
          float flicker = 0.7 + 0.3 * sin(uTime * 3.0 + uFlarePhase * 10.0);
          float pulse = 0.8 + 0.2 * sin(uTime * 1.5 + uFlarePhase * 5.0);

          float intensity = core * flicker * pulse;

          // Color - bright white/yellow
          vec3 color = vec3(1.0, 0.95, 0.85) * intensity;

          gl_FragColor = vec4(color, intensity * 0.9);
        }
      `,
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
    });
  }
}

extend({ SunSurfaceMaterial, CoronaRingMaterial, SolarFlareMaterial });

interface CrossroadsOrbProps {
  position?: [number, number, number];
}

// Outer glow texture - very smooth exponential falloff
const createGlowTexture = () => {
  const size = 512;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  // Use more gradient stops for smoother falloff
  const gradient = ctx.createRadialGradient(size/2, size/2, 0, size/2, size/2, size/2);
  gradient.addColorStop(0, 'rgba(255, 255, 255, 0.8)');
  gradient.addColorStop(0.05, 'rgba(255, 255, 255, 0.6)');
  gradient.addColorStop(0.1, 'rgba(255, 255, 255, 0.4)');
  gradient.addColorStop(0.15, 'rgba(255, 252, 250, 0.25)');
  gradient.addColorStop(0.2, 'rgba(255, 250, 248, 0.15)');
  gradient.addColorStop(0.3, 'rgba(255, 248, 245, 0.08)');
  gradient.addColorStop(0.4, 'rgba(255, 245, 240, 0.04)');
  gradient.addColorStop(0.5, 'rgba(255, 242, 238, 0.02)');
  gradient.addColorStop(0.6, 'rgba(255, 240, 235, 0.01)');
  gradient.addColorStop(0.7, 'rgba(255, 238, 232, 0.005)');
  gradient.addColorStop(1, 'rgba(255, 235, 230, 0)');

  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

// Particle texture
const createParticleTexture = (color: [number, number, number]) => {
  const size = 64;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(size/2, size/2, 0, size/2, size/2, size/2);
  gradient.addColorStop(0, `rgba(${color[0]}, ${color[1]}, ${color[2]}, 1)`);
  gradient.addColorStop(0.3, `rgba(${color[0]}, ${color[1]}, ${color[2]}, 0.5)`);
  gradient.addColorStop(0.6, `rgba(${color[0]}, ${color[1]}, ${color[2]}, 0.15)`);
  gradient.addColorStop(1, `rgba(${color[0]}, ${color[1]}, ${color[2]}, 0)`);

  ctx.fillStyle = gradient;
  ctx.beginPath();
  ctx.arc(size/2, size/2, size/2, 0, Math.PI * 2);
  ctx.fill();

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

// Solar particle data
interface SolarParticle {
  position: THREE.Vector3;
  basePosition: THREE.Vector3;
  velocity: THREE.Vector3;
  phase: number;
  speed: number;
  radius: number;
}

const CrossroadsOrb: React.FC<CrossroadsOrbProps> = ({ position = [0, 0, 0] }) => {
  const groupRef = useRef<THREE.Group>(null);
  const sunPlaneRef = useRef<THREE.Mesh>(null);
  const coronaGroupRef = useRef<THREE.Group>(null);
  const corona2GroupRef = useRef<THREE.Group>(null);
  const corona3GroupRef = useRef<THREE.Group>(null);
  const glowRef = useRef<THREE.Sprite>(null);
  const glow2Ref = useRef<THREE.Sprite>(null);
  const brightParticlesRef = useRef<THREE.Points>(null);
  const streamParticlesRef = useRef<THREE.Points>(null);
  const flaresRef = useRef<THREE.Group>(null);

  const sunMaterialRef = useRef<SunSurfaceMaterial>(null);
  const coronaMaterialRef = useRef<CoronaRingMaterial>(null);
  const corona2MaterialRef = useRef<CoronaRingMaterial>(null);
  const corona3MaterialRef = useRef<CoronaRingMaterial>(null);
  const flareMaterialsRef = useRef<SolarFlareMaterial[]>([]);

  const { camera, pointer, raycaster } = useThree();
  const mouseWorld = useRef(new THREE.Vector3());
  const isHovered = useRef(false);
  const hoverAmount = useRef(0);

  const sunRadius = 1.6;
  const brightParticleCount = 250;
  const streamParticleCount = 200;
  const flareCount = 8;

  // Solar flare positions - distributed around the sphere
  const flareData = useMemo(() => {
    const flares = [];
    for (let i = 0; i < flareCount; i++) {
      const theta = (i / flareCount) * Math.PI * 2 + Math.random() * 0.3;
      const phi = Math.PI / 2 + (Math.random() - 0.5) * 0.8; // Mostly around equator
      const length = 0.8 + Math.random() * 1.2; // Flare length
      const width = 0.15 + Math.random() * 0.15;
      const phase = Math.random() * Math.PI * 2;

      flares.push({ theta, phi, length, width, phase });
    }
    return flares;
  }, [flareCount]);

  // Textures
  const glowTexture = useMemo(() => createGlowTexture(), []);
  const brightTexture = useMemo(() => createParticleTexture([255, 255, 255]), []);

  // Curl noise for fluid particle motion
  const curl3D = useCallback((x: number, y: number, z: number, time: number): THREE.Vector3 => {
    const eps = 0.01;
    const eps2 = 2 * eps;

    const noise2D = (px: number, py: number): number => {
      return Math.sin(px * 1.5 + time) * Math.sin(py * 1.5) * 0.5 +
             Math.sin(px * 3.7 - time * 0.7) * Math.sin(py * 2.3) * 0.25 +
             Math.sin(px * 7.1 + time * 0.3) * Math.sin(py * 5.9) * 0.125;
    };

    const curl = new THREE.Vector3();

    let n1 = noise2D(x, y + eps);
    let n2 = noise2D(x, y - eps);
    let a = (n1 - n2) / eps2;
    n1 = noise2D(x, z + eps);
    n2 = noise2D(x, z - eps);
    let b = (n1 - n2) / eps2;
    curl.x = a - b;

    n1 = noise2D(y, z + eps);
    n2 = noise2D(y, z - eps);
    a = (n1 - n2) / eps2;
    n1 = noise2D(x + eps, z);
    n2 = noise2D(x - eps, z);
    b = (n1 - n2) / eps2;
    curl.y = a - b;

    n1 = noise2D(x + eps, y);
    n2 = noise2D(x - eps, y);
    a = (n1 - n2) / eps2;
    n1 = noise2D(y + eps, z);
    n2 = noise2D(y - eps, z);
    b = (n1 - n2) / eps2;
    curl.z = a - b;

    return curl;
  }, []);

  // Bright corona particles streaming outward
  const brightParticles = useMemo((): SolarParticle[] => {
    const particles: SolarParticle[] = [];
    for (let i = 0; i < brightParticleCount; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = sunRadius + 0.3 + Math.random() * 2.0;

      const pos = new THREE.Vector3(
        r * Math.sin(phi) * Math.cos(theta),
        r * Math.sin(phi) * Math.sin(theta),
        r * Math.cos(phi)
      );

      particles.push({
        position: pos.clone(),
        basePosition: pos.clone(),
        velocity: pos.clone().normalize().multiplyScalar(0.3 + Math.random() * 0.6),
        phase: Math.random() * Math.PI * 2,
        speed: 0.15 + Math.random() * 0.35,
        radius: r,
      });
    }
    return particles;
  }, [sunRadius, brightParticleCount]);

  // Stream particles with curl motion
  const streamParticles = useMemo((): SolarParticle[] => {
    const particles: SolarParticle[] = [];
    for (let i = 0; i < streamParticleCount; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = sunRadius + 0.4 + Math.random() * 2.5;

      const pos = new THREE.Vector3(
        r * Math.sin(phi) * Math.cos(theta),
        r * Math.sin(phi) * Math.sin(theta),
        r * Math.cos(phi)
      );

      particles.push({
        position: pos.clone(),
        basePosition: pos.clone(),
        velocity: new THREE.Vector3(),
        phase: Math.random() * Math.PI * 2,
        speed: 0.08 + Math.random() * 0.15,
        radius: r,
      });
    }
    return particles;
  }, [sunRadius, streamParticleCount]);

  // Position arrays
  const brightPositions = useMemo(() => new Float32Array(brightParticleCount * 3), [brightParticleCount]);
  const streamPositions = useMemo(() => new Float32Array(streamParticleCount * 3), [streamParticleCount]);

  useFrame((state, delta) => {
    const time = state.clock.elapsedTime;

    // Update mouse position
    raycaster.setFromCamera(pointer, camera);
    const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
    raycaster.ray.intersectPlane(plane, mouseWorld.current);

    // Check hover
    if (groupRef.current) {
      const groupPos = new THREE.Vector3();
      groupRef.current.getWorldPosition(groupPos);
      isHovered.current = mouseWorld.current.distanceTo(groupPos) < sunRadius + 2;

      const targetHover = isHovered.current ? 1.0 : 0.0;
      hoverAmount.current += (targetHover - hoverAmount.current) * 0.08;

      // Gentle breathing
      const breathe = Math.sin(time * 0.2) * 0.015;
      groupRef.current.position.y = position[1] + breathe;
    }

    // Update sun shader
    if (sunMaterialRef.current) {
      sunMaterialRef.current.uniforms.uTime.value = time;
      sunMaterialRef.current.uniforms.uHover.value = hoverAmount.current;
    }

    // Update corona shaders
    if (coronaMaterialRef.current) {
      coronaMaterialRef.current.uniforms.uTime.value = time;
    }
    if (corona2MaterialRef.current) {
      corona2MaterialRef.current.uniforms.uTime.value = time;
    }
    if (corona3MaterialRef.current) {
      corona3MaterialRef.current.uniforms.uTime.value = time;
    }

    // Animate corona shells - TRUE GYROSCOPE EFFECT
    // Rotate the groups on different axes so the fresnel rings tilt in 3D
    if (coronaGroupRef.current) {
      // Inner ring: rotates around X axis (tilts forward/back)
      coronaGroupRef.current.rotation.x += delta * 0.3;
      coronaGroupRef.current.rotation.z += delta * 0.1;
    }
    if (corona2GroupRef.current) {
      // Middle ring: rotates around Z axis (tilts side to side)
      corona2GroupRef.current.rotation.z -= delta * 0.25;
      corona2GroupRef.current.rotation.y += delta * 0.12;
    }
    if (corona3GroupRef.current) {
      // Outer ring: rotates around Y axis with X tilt
      corona3GroupRef.current.rotation.y += delta * 0.18;
      corona3GroupRef.current.rotation.x -= delta * 0.15;
    }

    // Animate flares
    if (flaresRef.current) {
      flaresRef.current.rotation.y += delta * 0.02;
      flaresRef.current.children.forEach((flare, i) => {
        const material = flareMaterialsRef.current[i];
        if (material) {
          material.uniforms.uTime.value = time;
        }
        // Subtle scale pulsing per flare
        const pulseFactor = 0.9 + Math.sin(time * 2 + i * 1.5) * 0.15;
        flare.scale.y = pulseFactor;
      });
    }

    // Sphere rotation and subtle pulsing
    if (sunPlaneRef.current) {
      // Slow rotation for living surface feel
      sunPlaneRef.current.rotation.y += delta * 0.05;

      // Subtle pulsing scale
      const pulse = 1 + Math.sin(time * 0.4) * 0.02 + Math.sin(time * 0.7) * 0.01;
      const hoverScale = 1 + hoverAmount.current * 0.05;
      const scale = pulse * hoverScale;
      sunPlaneRef.current.scale.setScalar(scale);
    }

    // Glow pulsing - dramatic breathing
    const basePulse = Math.sin(time * 0.5) * 0.5;
    const breathePulse = Math.sin(time * 0.25) * 0.3;
    const quickPulse = Math.sin(time * 1.5) * 0.15;
    const hoverBoost = hoverAmount.current * 1.0;

    if (glowRef.current) {
      const scale = 8 + basePulse + breathePulse + quickPulse + hoverBoost;
      glowRef.current.scale.set(scale, scale, 1);
    }
    if (glow2Ref.current) {
      const scale = 16 + basePulse * 1.5 + breathePulse + hoverBoost;
      glow2Ref.current.scale.set(scale, scale, 1);
    }

    // Animate bright streaming particles
    for (let i = 0; i < brightParticles.length; i++) {
      const p = brightParticles[i];

      // Stream outward
      p.position.add(p.velocity.clone().multiplyScalar(delta * p.speed));

      // Reset when too far
      const dist = p.position.length();
      if (dist > sunRadius + 4) {
        const theta = Math.random() * Math.PI * 2;
        const phi = Math.acos(2 * Math.random() - 1);
        const r = sunRadius + 0.3;

        p.position.set(
          r * Math.sin(phi) * Math.cos(theta),
          r * Math.sin(phi) * Math.sin(theta),
          r * Math.cos(phi)
        );
        p.velocity.copy(p.position.clone().normalize().multiplyScalar(0.3 + Math.random() * 0.6));
      }

      brightPositions[i * 3] = p.position.x;
      brightPositions[i * 3 + 1] = p.position.y;
      brightPositions[i * 3 + 2] = p.position.z;
    }

    // Animate stream particles with curl
    for (let i = 0; i < streamParticles.length; i++) {
      const p = streamParticles[i];

      const curl = curl3D(p.position.x * 0.2, p.position.y * 0.2, p.position.z * 0.2, time * 0.5);
      const radial = p.position.clone().normalize();
      const blended = radial.lerp(curl.normalize(), 0.6);

      p.position.add(blended.multiplyScalar(delta * p.speed));

      const dist = p.position.length();
      if (dist > sunRadius + 3.5 || dist < sunRadius + 0.3) {
        const theta = Math.random() * Math.PI * 2;
        const phi = Math.acos(2 * Math.random() - 1);
        const r = sunRadius + 0.5 + Math.random() * 1.5;

        p.position.set(
          r * Math.sin(phi) * Math.cos(theta),
          r * Math.sin(phi) * Math.sin(theta),
          r * Math.cos(phi)
        );
      }

      streamPositions[i * 3] = p.position.x;
      streamPositions[i * 3 + 1] = p.position.y;
      streamPositions[i * 3 + 2] = p.position.z;
    }

    // Update GPU buffers
    if (brightParticlesRef.current) {
      brightParticlesRef.current.geometry.attributes.position.needsUpdate = true;
    }
    if (streamParticlesRef.current) {
      streamParticlesRef.current.geometry.attributes.position.needsUpdate = true;
    }
  });

  return (
    <group ref={groupRef} position={position}>
      {/* Outer atmospheric glow - dramatic halo */}
      <sprite ref={glow2Ref} scale={[16, 16, 1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.4}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Inner corona glow - bright ring */}
      <sprite ref={glowRef} scale={[8, 8, 1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.6}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Main sun sphere with animated noise shader */}
      <mesh ref={sunPlaneRef}>
        <sphereGeometry args={[sunRadius, 64, 64]} />
        {/* @ts-ignore */}
        <sunSurfaceMaterial ref={sunMaterialRef} />
      </mesh>

      {/* Inner corona ring - gyroscope ring 1 (horizontal) */}
      <group ref={coronaGroupRef}>
        <mesh>
          <torusGeometry args={[sunRadius * 1.3, 0.08, 16, 64]} />
          {/* @ts-ignore */}
          <coronaRingMaterial ref={coronaMaterialRef} uIntensity={1.0} />
        </mesh>
      </group>

      {/* Middle corona ring - gyroscope ring 2 (tilted 60°) */}
      <group ref={corona2GroupRef} rotation={[Math.PI / 3, 0, 0]}>
        <mesh>
          <torusGeometry args={[sunRadius * 1.5, 0.06, 16, 64]} />
          {/* @ts-ignore */}
          <coronaRingMaterial ref={corona2MaterialRef} uIntensity={0.8} />
        </mesh>
      </group>

      {/* Outer corona ring - gyroscope ring 3 (tilted 120°) */}
      <group ref={corona3GroupRef} rotation={[0, 0, Math.PI / 3]}>
        <mesh>
          <torusGeometry args={[sunRadius * 1.7, 0.05, 16, 64]} />
          {/* @ts-ignore */}
          <coronaRingMaterial ref={corona3MaterialRef} uIntensity={0.6} />
        </mesh>
      </group>

      {/* Solar flares - elongated rays shooting out */}
      <group ref={flaresRef}>
        {flareData.map((flare, i) => {
          // Direction on sphere surface
          const dirX = Math.sin(flare.phi) * Math.cos(flare.theta);
          const dirY = Math.cos(flare.phi);
          const dirZ = Math.sin(flare.phi) * Math.sin(flare.theta);

          // Position: start at surface, offset by half flare length outward
          const offsetDist = sunRadius + flare.length * 0.5;
          const x = dirX * offsetDist;
          const y = dirY * offsetDist;
          const z = dirZ * offsetDist;

          // Direction pointing outward
          const dir = new THREE.Vector3(dirX, dirY, dirZ);

          // Create rotation to point flare outward
          const quaternion = new THREE.Quaternion();
          quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), dir);
          const euler = new THREE.Euler().setFromQuaternion(quaternion);

          return (
            <mesh
              key={i}
              position={[x, y, z]}
              rotation={[euler.x, euler.y, euler.z]}
            >
              <planeGeometry args={[flare.width, flare.length]} />
              {/* @ts-ignore */}
              <solarFlareMaterial
                ref={(el: SolarFlareMaterial | null) => {
                  if (el) flareMaterialsRef.current[i] = el;
                }}
                uFlarePhase={flare.phase}
              />
            </mesh>
          );
        })}
      </group>

      {/* Curl stream particles - living fluid effect */}
      <points ref={streamParticlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={streamParticleCount}
            array={streamPositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={brightTexture}
          size={0.15}
          transparent
          opacity={0.7}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </points>

      {/* Bright streaming outward particles */}
      <points ref={brightParticlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={brightParticleCount}
            array={brightPositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          map={brightTexture}
          size={0.1}
          transparent
          opacity={0.6}
          sizeAttenuation
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </points>

      {/* Main white light */}
      <pointLight color="#ffffff" intensity={8} distance={18} decay={2} />

      {/* Warm accent light */}
      <pointLight color="#fff8f0" intensity={3} distance={10} decay={2} />
    </group>
  );
};

export default CrossroadsOrb;
