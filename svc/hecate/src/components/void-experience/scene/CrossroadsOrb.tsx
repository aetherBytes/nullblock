import React, { useRef, useMemo, useCallback } from 'react';
import { useFrame, useThree, extend } from '@react-three/fiber';
import * as THREE from 'three';

// Animated sun surface shader with visible noise distortion
class SunSurfaceMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uHover: { value: 0 },
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
        uniform float uHover;
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
          vec2 uv = vUv;
          vec2 center = uv - 0.5;
          float dist = length(center) * 2.0; // 0 at center, 1 at edge
          float angle = atan(center.y, center.x);

          // Hard cutoff - discard pixels beyond effect radius
          if (dist > 0.35) {
            discard;
          }

          // Polar coordinates for swirling effect
          float time = uTime * 0.15;
          vec2 polarUV = vec2(angle / 6.28318 + 0.5, dist);

          // Multiple noise layers at different scales and speeds
          float noise1 = fbm(polarUV * 3.0 + vec2(time * 0.3, -time * 0.1));
          float noise2 = fbm(polarUV * 6.0 + vec2(-time * 0.5, time * 0.2));
          float noise3 = snoise(polarUV * 12.0 + vec2(time * 0.8, -time * 0.4));
          float radialNoise = fbm(vec2(dist * 4.0, angle * 2.0 + time * 0.2));

          // Combine noise layers
          float combinedNoise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.15 + radialNoise * 0.2;
          float surfaceDetail = combinedNoise * 0.5 + 0.5;

          // === AGGRESSIVE FADE: reaches zero by dist = 0.32 ===
          float masterFade = 1.0 - smoothstep(0.2, 0.32, dist);
          masterFade = pow(masterFade, 2.5); // Very aggressive curve

          // Large black core - takes up most of the center
          float coreRadius = 0.14 + combinedNoise * 0.02;
          float coreMask = 1.0 - smoothstep(coreRadius - 0.01, coreRadius + 0.03, dist);

          // Corona ring: thin bright ring around the black core
          float coronaPeak = 0.18 + noise1 * 0.015;
          float coronaWidth = 0.04 + noise2 * 0.01;
          float corona = exp(-pow((dist - coronaPeak) / coronaWidth, 2.0));
          corona *= (0.5 + noise1 * 0.3 + noise3 * 0.2); // Reduced intensity

          // Very subtle inner glow
          float innerGlow = smoothstep(0.1, 0.15, dist) * (1.0 - smoothstep(0.15, 0.22, dist));
          innerGlow *= (0.4 + surfaceDetail * 0.3);

          // Colors - reduced brightness
          vec3 darkSurface = vec3(0.03, 0.015, 0.015) * surfaceDetail;
          vec3 warmGray = vec3(0.3, 0.27, 0.24) * (0.5 + noise2 * 0.3);
          vec3 corona_color = vec3(0.7, 0.68, 0.65); // Dimmer corona

          // Build color - start with black
          vec3 color = vec3(0.0);

          // Only add brightness outside the core
          float outsideCore = 1.0 - coreMask;

          // Subtle surface detail at core edge
          color += darkSurface * outsideCore * 0.3;

          // Warm transition
          color += warmGray * innerGlow * outsideCore * 0.4;

          // Corona ring - the main visible bright element
          color += corona_color * corona * outsideCore * 0.6;

          // Subtle turbulence
          float turbulence = (noise1 * 0.3 + noise3 * 0.2) * corona * outsideCore;
          color += vec3(0.5, 0.48, 0.45) * turbulence * 0.2;

          // Apply aggressive master fade
          color *= masterFade;

          // Gentle pulsing
          float pulse = 1.0 + sin(uTime * 0.5) * 0.03;
          color *= pulse;

          // Output - with additive blending, black = invisible
          gl_FragColor = vec4(color, 1.0);
        }
      `,
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending, // Black = adds nothing = invisible edges
    });
  }
}

extend({ SunSurfaceMaterial });

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
  const glowRef = useRef<THREE.Sprite>(null);
  const glow2Ref = useRef<THREE.Sprite>(null);
  const brightParticlesRef = useRef<THREE.Points>(null);
  const streamParticlesRef = useRef<THREE.Points>(null);

  const sunMaterialRef = useRef<SunSurfaceMaterial>(null);

  const { camera, pointer, raycaster } = useThree();
  const mouseWorld = useRef(new THREE.Vector3());
  const isHovered = useRef(false);
  const hoverAmount = useRef(0);

  const sunRadius = 1.6;
  const brightParticleCount = 250;
  const streamParticleCount = 200;

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

    // Billboard the sun plane to face camera
    if (sunPlaneRef.current) {
      sunPlaneRef.current.lookAt(camera.position);

      // Pulsing scale - large plane, effect only uses inner 50%
      const pulse = 1 + Math.sin(time * 0.4) * 0.04 + Math.sin(time * 0.7) * 0.02;
      const hoverScale = 1 + hoverAmount.current * 0.1;
      const scale = sunRadius * 7.0 * pulse * hoverScale;
      sunPlaneRef.current.scale.set(scale, scale, 1);
    }

    // Glow pulsing
    const basePulse = Math.sin(time * 0.5) * 0.3;
    const breathePulse = Math.sin(time * 0.25) * 0.15;
    const hoverBoost = hoverAmount.current * 0.5;

    if (glowRef.current) {
      const scale = 9 + basePulse + breathePulse + hoverBoost;
      glowRef.current.scale.set(scale, scale, 1);
    }
    if (glow2Ref.current) {
      const scale = 14 + basePulse * 1.4 + breathePulse * 0.8 + hoverBoost;
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
      {/* Outer atmospheric glow - very subtle */}
      <sprite ref={glow2Ref} scale={[10, 10, 1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.25}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Inner corona glow - subtle */}
      <sprite ref={glowRef} scale={[6, 6, 1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.35}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Main sun disc with animated noise shader */}
      <mesh ref={sunPlaneRef}>
        <planeGeometry args={[1, 1, 1, 1]} />
        {/* @ts-ignore */}
        <sunSurfaceMaterial ref={sunMaterialRef} />
      </mesh>

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
