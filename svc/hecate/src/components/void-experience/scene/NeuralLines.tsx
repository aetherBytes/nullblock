import React, { useRef, useMemo } from 'react';
import { useFrame, extend } from '@react-three/fiber';
import * as THREE from 'three';
import type { ConstellationNode } from './VoidScene';

// Mini Crossroads-style shader for constellation nodes - blue themed
class ConstellationCoreMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uHue: { value: 0.6 }, // 0.55-0.65 for blue variations
        uBrightness: { value: 0.5 }, // Variation in brightness
      },
      vertexShader: `
        varying vec3 vNormal;
        varying vec3 vPosition;

        void main() {
          vNormal = normalize(normalMatrix * normal);
          vPosition = position;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        precision highp float;

        uniform float uTime;
        uniform float uHue;
        uniform float uBrightness;
        varying vec3 vNormal;
        varying vec3 vPosition;

        // Simplex noise functions
        vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec4 permute(vec4 x) { return mod289(((x*34.0)+1.0)*x); }
        vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

        float snoise3D(vec3 v) {
          const vec2 C = vec2(1.0/6.0, 1.0/3.0);
          const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);

          vec3 i = floor(v + dot(v, C.yyy));
          vec3 x0 = v - i + dot(i, C.xxx);

          vec3 g = step(x0.yzx, x0.xyz);
          vec3 l = 1.0 - g;
          vec3 i1 = min(g.xyz, l.zxy);
          vec3 i2 = max(g.xyz, l.zxy);

          vec3 x1 = x0 - i1 + C.xxx;
          vec3 x2 = x0 - i2 + C.yyy;
          vec3 x3 = x0 - D.yyy;

          i = mod289(i);
          vec4 p = permute(permute(permute(
                    i.z + vec4(0.0, i1.z, i2.z, 1.0))
                  + i.y + vec4(0.0, i1.y, i2.y, 1.0))
                  + i.x + vec4(0.0, i1.x, i2.x, 1.0));

          float n_ = 0.142857142857;
          vec3 ns = n_ * D.wyz - D.xzx;

          vec4 j = p - 49.0 * floor(p * ns.z * ns.z);

          vec4 x_ = floor(j * ns.z);
          vec4 y_ = floor(j - 7.0 * x_);

          vec4 x = x_ *ns.x + ns.yyyy;
          vec4 y = y_ *ns.x + ns.yyyy;
          vec4 h = 1.0 - abs(x) - abs(y);

          vec4 b0 = vec4(x.xy, y.xy);
          vec4 b1 = vec4(x.zw, y.zw);

          vec4 s0 = floor(b0)*2.0 + 1.0;
          vec4 s1 = floor(b1)*2.0 + 1.0;
          vec4 sh = -step(h, vec4(0.0));

          vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy;
          vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww;

          vec3 p0 = vec3(a0.xy, h.x);
          vec3 p1 = vec3(a0.zw, h.y);
          vec3 p2 = vec3(a1.xy, h.z);
          vec3 p3 = vec3(a1.zw, h.w);

          vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
          p0 *= norm.x;
          p1 *= norm.y;
          p2 *= norm.z;
          p3 *= norm.w;

          vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
          m = m * m;
          return 42.0 * dot(m*m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
        }

        // FBM
        float fbm3D(vec3 p) {
          float value = 0.0;
          float amplitude = 0.5;
          float frequency = 1.0;
          for (int i = 0; i < 5; i++) {
            value += amplitude * snoise3D(p * frequency);
            amplitude *= 0.5;
            frequency *= 2.0;
          }
          return value;
        }

        // HSV to RGB
        vec3 hsv2rgb(vec3 c) {
          vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
          vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
          return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
        }

        void main() {
          float time = uTime * 0.2;

          vec3 pos = normalize(vPosition);

          // Noise layers for surface detail
          float noise1 = fbm3D(pos * 3.0 + vec3(time * 0.2, -time * 0.1, time * 0.15));
          float noise2 = fbm3D(pos * 5.0 + vec3(-time * 0.3, time * 0.15, -time * 0.1));
          float noise3 = snoise3D(pos * 8.0 + vec3(time * 0.4, -time * 0.2, time * 0.25));

          float combinedNoise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;

          // Fresnel
          float fresnel = 1.0 - abs(dot(vNormal, vec3(0.0, 0.0, 1.0)));
          float center = 1.0 - fresnel;

          // Core intensities
          float coreIntensity = pow(center, 4.0);
          float innerCore = pow(center, 8.0);
          float deepCore = pow(center, 12.0);

          // Pulse
          float pulse = sin(uTime * 0.4) * 0.5 + 0.5;

          // Blue color palette based on hue uniform
          vec3 darkBlue = hsv2rgb(vec3(uHue, 0.8, 0.1));
          vec3 midBlue = hsv2rgb(vec3(uHue, 0.7, 0.3 * uBrightness));
          vec3 paleBlue = hsv2rgb(vec3(uHue - 0.05, 0.4, 0.7 * uBrightness));
          vec3 brightBlue = hsv2rgb(vec3(uHue - 0.02, 0.5, 0.9 * uBrightness));
          vec3 coreWhite = vec3(0.8, 0.9, 1.0) * uBrightness;

          // Build color
          vec3 color = darkBlue;

          // Inner glow
          color = mix(color, midBlue, innerCore * 0.6);
          color = mix(color, paleBlue, deepCore * 0.8);

          // Noise-based surface detail
          float surfaceDetail = smoothstep(-0.2, 0.5, combinedNoise);
          color = mix(color, brightBlue, surfaceDetail * coreIntensity * 0.4);

          // Core brightness
          color = mix(color, coreWhite, deepCore * pulse * 0.5);

          // Rim glow
          float rim = pow(fresnel, 2.0);
          vec3 rimColor = hsv2rgb(vec3(uHue + 0.05, 0.5, 0.6));
          color += rimColor * rim * 0.4;

          // Bright edge
          float edgeGlow = pow(fresnel, 4.0);
          color += paleBlue * edgeGlow * 0.3;

          gl_FragColor = vec4(color, 1.0);
        }
      `,
      transparent: false,
      depthWrite: true,
      side: THREE.FrontSide,
    });
  }
}

extend({ ConstellationCoreMaterial });

// TypeScript declaration for the custom material in JSX
declare global {
  namespace JSX {
    interface IntrinsicElements {
      constellationCoreMaterial: React.DetailedHTMLProps<
        React.HTMLAttributes<THREE.ShaderMaterial> & {
          ref?: React.Ref<THREE.ShaderMaterial>;
        },
        THREE.ShaderMaterial
      >;
    }
  }
}

// Create a soft radial glow texture for active nodes
const createNodeGlowTexture = () => {
  const size = 128;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(
    size / 2, size / 2, 0,
    size / 2, size / 2, size / 2
  );
  // Soft white glow that fades out
  gradient.addColorStop(0, 'rgba(255, 255, 255, 1.0)');
  gradient.addColorStop(0.1, 'rgba(255, 255, 255, 0.8)');
  gradient.addColorStop(0.25, 'rgba(255, 255, 255, 0.5)');
  gradient.addColorStop(0.4, 'rgba(255, 255, 255, 0.25)');
  gradient.addColorStop(0.6, 'rgba(255, 255, 255, 0.1)');
  gradient.addColorStop(0.8, 'rgba(255, 255, 255, 0.03)');
  gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');

  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

/**
 * NeuralLines - Background constellation network representing Tools, Services, and Servers
 *
 * These minor nodes and connecting lines visualize the infrastructure layer:
 * - Tools being invoked
 * - Services communicating
 * - MCP servers and protocols in action
 *
 * The pulsing animations represent active data flow between systems.
 * Unlike Agent nodes (major nodes), these are decorative background elements.
 */
interface NeuralLinesProps {
  nodes: ConstellationNode[];
  activeNodes?: Set<number>;
}

const NeuralLines: React.FC<NeuralLinesProps> = ({ nodes, activeNodes = new Set() }) => {
  const groupRef = useRef<THREE.Group>(null);
  const linesRef = useRef<THREE.LineSegments>(null);
  const nodeMaterialRefs = useRef<(THREE.ShaderMaterial | null)[]>([]);

  // Create glow texture once
  const glowTexture = useMemo(() => createNodeGlowTexture(), []);

  // Create a soft blue glow texture for nodes (always visible, subtle)
  const nodeGlowTexture = useMemo(() => {
    const size = 128;
    const canvas = document.createElement('canvas');
    canvas.width = size;
    canvas.height = size;
    const ctx = canvas.getContext('2d')!;

    const gradient = ctx.createRadialGradient(
      size / 2, size / 2, 0,
      size / 2, size / 2, size / 2
    );
    // Subtle blue glow
    gradient.addColorStop(0, 'rgba(100, 180, 255, 0.6)');
    gradient.addColorStop(0.2, 'rgba(80, 150, 230, 0.3)');
    gradient.addColorStop(0.4, 'rgba(60, 120, 200, 0.15)');
    gradient.addColorStop(0.6, 'rgba(40, 100, 180, 0.05)');
    gradient.addColorStop(1, 'rgba(30, 80, 150, 0)');

    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, size, size);

    const texture = new THREE.CanvasTexture(canvas);
    texture.needsUpdate = true;
    return texture;
  }, []);

  // Generate per-node variations (hue and brightness)
  const nodeVariations = useMemo(() => {
    return nodes.map(() => ({
      hue: 0.55 + Math.random() * 0.1, // 0.55-0.65 for blue range
      brightness: 0.4 + Math.random() * 0.6, // 0.4-1.0 brightness variation
    }));
  }, [nodes]);

  // Build line geometry from nodes
  const { linePositions, lineOpacities } = useMemo(() => {
    const linePositions: number[] = [];
    const lineOpacities: number[] = [];
    const processedPairs = new Set<string>();

    for (let i = 0; i < nodes.length; i++) {
      for (const j of nodes[i].connections) {
        const pairKey = i < j ? `${i}-${j}` : `${j}-${i}`;
        if (processedPairs.has(pairKey)) continue;
        processedPairs.add(pairKey);

        linePositions.push(
          nodes[i].position.x, nodes[i].position.y, nodes[i].position.z,
          nodes[j].position.x, nodes[j].position.y, nodes[j].position.z
        );

        // Random base opacity for each line
        const baseOpacity = 0.1 + Math.random() * 0.12;
        lineOpacities.push(baseOpacity, baseOpacity);
      }
    }

    return {
      linePositions: new Float32Array(linePositions),
      lineOpacities: new Float32Array(lineOpacities)
    };
  }, [nodes]);

  // Create animated opacity attribute
  const opacityAttr = useMemo(() => {
    return new THREE.BufferAttribute(lineOpacities.slice(), 1);
  }, [lineOpacities]);

  useFrame((state) => {
    if (!linesRef.current) return;

    const time = state.clock.elapsedTime;

    // Constellation nodes are now static - no rotation
    // This allows tendrils from CrossroadsOrb to accurately target them

    // Animate line opacities - subtle pulsing
    const opacities = opacityAttr.array as Float32Array;
    for (let i = 0; i < opacities.length; i += 2) {
      const baseOpacity = lineOpacities[i];
      const pulse = Math.sin(time * 0.5 + i * 0.1) * 0.02;
      const newOpacity = Math.max(0.01, baseOpacity + pulse);
      opacities[i] = newOpacity;
      opacities[i + 1] = newOpacity;
    }
    opacityAttr.needsUpdate = true;

    // Update node material time uniforms
    nodeMaterialRefs.current.forEach((mat) => {
      if (mat) {
        mat.uniforms.uTime.value = time;
      }
    });
  });

  return (
    <group ref={groupRef}>
      <lineSegments ref={linesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={linePositions.length / 3}
            array={linePositions}
            itemSize={3}
          />
          <bufferAttribute
            attach="attributes-opacity"
            {...opacityAttr}
          />
        </bufferGeometry>
        <lineBasicMaterial
          color="#ffffff"
          transparent
          opacity={0.2}
          blending={THREE.AdditiveBlending}
          depthWrite={false}
        />
      </lineSegments>

      {/* Service nodes - mini Crossroads-style orbs representing tools/services/servers */}
      {nodes.map((node, i) => {
        const isActive = activeNodes.has(i);
        const variation = nodeVariations[i] || { hue: 0.6, brightness: 0.5 };
        return (
          <group key={i} position={[node.position.x, node.position.y, node.position.z]}>
            {/* Ambient blue glow behind node (always visible) */}
            <sprite scale={[0.5, 0.5, 1]}>
              <spriteMaterial
                map={nodeGlowTexture}
                transparent
                opacity={0.6}
                depthWrite={false}
                blending={THREE.AdditiveBlending}
              />
            </sprite>

            {/* Bright white glow when tendril is connected */}
            {isActive && (
              <sprite scale={[1.0, 1.0, 1]}>
                <spriteMaterial
                  map={glowTexture}
                  transparent
                  opacity={0.9}
                  depthWrite={false}
                  blending={THREE.AdditiveBlending}
                />
              </sprite>
            )}
            {/* Larger outer glow halo when active */}
            {isActive && (
              <sprite scale={[1.8, 1.8, 1]}>
                <spriteMaterial
                  map={glowTexture}
                  transparent
                  opacity={0.4}
                  depthWrite={false}
                  blending={THREE.AdditiveBlending}
                />
              </sprite>
            )}

            {/* Core node sphere with Crossroads-style shader */}
            <mesh>
              <sphereGeometry args={[0.12, 24, 24]} />
              <constellationCoreMaterial
                ref={(el: THREE.ShaderMaterial | null) => {
                  nodeMaterialRefs.current[i] = el;
                  if (el) {
                    el.uniforms.uHue.value = variation.hue;
                    el.uniforms.uBrightness.value = variation.brightness;
                  }
                }}
              />
            </mesh>

            {/* Subtle point light for each node */}
            <pointLight
              color={isActive ? "#ffffff" : "#4488ff"}
              intensity={isActive ? 0.4 : 0.15}
              distance={isActive ? 2.0 : 1.0}
              decay={2}
            />
          </group>
        );
      })}
    </group>
  );
};

export default NeuralLines;
