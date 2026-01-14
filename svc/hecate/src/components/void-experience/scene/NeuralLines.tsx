import { useFrame, extend } from '@react-three/fiber';
import React, { useRef, useMemo } from 'react';
import * as THREE from 'three';
import type { ConstellationNode, ClusterOrbit } from './VoidScene';

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

        // FBM - 4 octaves (reduced from 5 for GPU optimization)
        float fbm3D(vec3 p) {
          float value = 0.0;
          float amplitude = 0.5;
          float frequency = 1.0;
          for (int i = 0; i < 4; i++) {
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
          float time = uTime * 0.3;

          vec3 pos = normalize(vPosition);

          // Multiple noise layers for gaseous, flowing effect
          float noise1 = fbm3D(pos * 2.0 + vec3(time * 0.3, -time * 0.2, time * 0.25));
          float noise2 = fbm3D(pos * 4.0 + vec3(-time * 0.4, time * 0.2, -time * 0.15));
          float noise3 = snoise3D(pos * 6.0 + vec3(time * 0.5, -time * 0.3, time * 0.35));
          float noise4 = snoise3D(pos * 10.0 + vec3(-time * 0.6, time * 0.4, -time * 0.2));

          // Combine for wispy, flowing patterns
          float gasFlow = noise1 * 0.4 + noise2 * 0.3 + noise3 * 0.2 + noise4 * 0.1;
          float wispyNoise = smoothstep(-0.3, 0.6, gasFlow);

          // Fresnel for edge softness
          float fresnel = 1.0 - abs(dot(vNormal, vec3(0.0, 0.0, 1.0)));
          float center = 1.0 - fresnel;

          // Softer core gradients for gaseous look
          float coreIntensity = pow(center, 2.0);
          float innerCore = pow(center, 4.0);
          float deepCore = pow(center, 6.0);

          // Pulse
          float pulse = sin(uTime * 0.5 + gasFlow * 2.0) * 0.5 + 0.5;

          // Blue color palette - more variation for gas clouds
          vec3 voidBlue = hsv2rgb(vec3(uHue + 0.02, 0.9, 0.05));
          vec3 darkBlue = hsv2rgb(vec3(uHue, 0.8, 0.15 * uBrightness));
          vec3 midBlue = hsv2rgb(vec3(uHue - 0.02, 0.6, 0.4 * uBrightness));
          vec3 paleBlue = hsv2rgb(vec3(uHue - 0.05, 0.4, 0.7 * uBrightness));
          vec3 brightBlue = hsv2rgb(vec3(uHue - 0.08, 0.3, 0.9 * uBrightness));
          vec3 coreWhite = vec3(0.85, 0.92, 1.0) * uBrightness;

          // Build gaseous color with noise-driven layers
          vec3 color = voidBlue;

          // Wispy gas layers driven by noise
          color = mix(color, darkBlue, wispyNoise * 0.7);
          color = mix(color, midBlue, wispyNoise * innerCore * 0.8);
          color = mix(color, paleBlue, smoothstep(0.3, 0.8, gasFlow) * coreIntensity * 0.6);

          // Bright wisps flowing through
          float brightWisps = smoothstep(0.4, 0.9, noise2 + noise3 * 0.5);
          color = mix(color, brightBlue, brightWisps * deepCore * 0.5);

          // Pulsing core glow
          color = mix(color, coreWhite, deepCore * pulse * 0.4);

          // Soft diffuse rim - not sharp edge
          float softRim = pow(fresnel, 1.5);
          vec3 rimColor = hsv2rgb(vec3(uHue + 0.03, 0.4, 0.5 * uBrightness));
          color += rimColor * softRim * wispyNoise * 0.5;

          // Calculate alpha for gaseous transparency
          // More transparent at edges, denser toward center
          float coreAlpha = smoothstep(0.0, 0.5, center);
          float noiseAlpha = 0.3 + wispyNoise * 0.4;
          float edgeFade = 1.0 - pow(fresnel, 1.2);

          float alpha = coreAlpha * noiseAlpha * edgeFade + deepCore * 0.5;
          alpha = clamp(alpha * 1.5, 0.0, 0.95);

          // Boost color intensity to compensate for transparency
          color *= 1.3;

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

  const gradient = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);

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
  clusterOrbits: ClusterOrbit[];
  animatedPositionsRef: React.MutableRefObject<THREE.Vector3[]>;
}

const NeuralLines: React.FC<NeuralLinesProps> = ({
  nodes,
  activeNodes = new Set(),
  clusterOrbits,
  animatedPositionsRef,
}) => {
  const groupRef = useRef<THREE.Group>(null);
  const linesRef = useRef<THREE.LineSegments>(null);
  const nodeMaterialRefs = useRef<(THREE.ShaderMaterial | null)[]>([]);
  const nodeGroupRefs = useRef<(THREE.Group | null)[]>([]);

  // Pre-allocated objects to avoid GC pressure in useFrame
  const tempMatrix = useRef(new THREE.Matrix4());
  const tempEuler = useRef(new THREE.Euler());
  const tempVec = useRef(new THREE.Vector3());
  const processedPairsSet = useRef(new Set<string>());

  // Create glow texture once
  const glowTexture = useMemo(() => createNodeGlowTexture(), []);

  // Create a soft blue glow texture for nodes (always visible, subtle)
  const nodeGlowTexture = useMemo(() => {
    const size = 128;
    const canvas = document.createElement('canvas');

    canvas.width = size;
    canvas.height = size;
    const ctx = canvas.getContext('2d')!;

    const gradient = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);

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
  const nodeVariations = useMemo(
    () =>
      nodes.map(() => ({
        hue: 0.55 + Math.random() * 0.1, // 0.55-0.65 for blue range
        brightness: 0.4 + Math.random() * 0.6, // 0.4-1.0 brightness variation
      })),
    [nodes],
  );

  // Build line geometry from nodes
  const { linePositions, lineOpacities } = useMemo(() => {
    const linePositions: number[] = [];
    const lineOpacities: number[] = [];
    const processedPairs = new Set<string>();

    for (let i = 0; i < nodes.length; i++) {
      for (const j of nodes[i].connections) {
        const pairKey = i < j ? `${i}-${j}` : `${j}-${i}`;

        if (processedPairs.has(pairKey)) {
          continue;
        }

        processedPairs.add(pairKey);

        linePositions.push(
          nodes[i].position.x,
          nodes[i].position.y,
          nodes[i].position.z,
          nodes[j].position.x,
          nodes[j].position.y,
          nodes[j].position.z,
        );

        // Random base opacity for each line
        const baseOpacity = 0.1 + Math.random() * 0.12;

        lineOpacities.push(baseOpacity, baseOpacity);
      }
    }

    return {
      linePositions: new Float32Array(linePositions),
      lineOpacities: new Float32Array(lineOpacities),
    };
  }, [nodes]);

  // Create animated opacity attribute
  const opacityAttr = useMemo(
    () => new THREE.BufferAttribute(lineOpacities.slice(), 1),
    [lineOpacities],
  );

  // Store base positions for orbital calculations
  const basePositions = useMemo(() => nodes.map((n) => n.position.clone()), [nodes]);

  useFrame((state) => {
    if (!linesRef.current) {
      return;
    }

    const time = state.clock.elapsedTime;

    // Animate each node based on its cluster's orbital parameters
    // Reuse pre-allocated objects to avoid GC pressure
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      const orbit = clusterOrbits[node.clusterId];

      if (!orbit) {
        continue;
      }

      const basePos = basePositions[i];
      const angle = orbit.phase + time * orbit.speed;

      // Reuse pre-allocated euler and matrix
      tempEuler.current.set(orbit.tiltX, angle, orbit.tiltZ, 'XYZ');
      tempMatrix.current.makeRotationFromEuler(tempEuler.current);

      // Apply rotation to base position (reuse tempVec instead of clone)
      tempVec.current.copy(basePos).applyMatrix4(tempMatrix.current);

      // Update the shared animated positions ref
      animatedPositionsRef.current[i].copy(tempVec.current);

      // Update the node group position
      const nodeGroup = nodeGroupRefs.current[i];

      if (nodeGroup) {
        nodeGroup.position.copy(tempVec.current);
      }
    }

    // Update line geometry with new animated positions
    const lineGeom = linesRef.current.geometry;
    const positions = lineGeom.attributes.position.array as Float32Array;
    let lineIdx = 0;

    // Reuse pre-allocated Set, clear instead of creating new
    processedPairsSet.current.clear();

    for (let i = 0; i < nodes.length; i++) {
      for (const j of nodes[i].connections) {
        const pairKey = i < j ? `${i}-${j}` : `${j}-${i}`;

        if (processedPairsSet.current.has(pairKey)) {
          continue;
        }

        processedPairsSet.current.add(pairKey);

        const posA = animatedPositionsRef.current[i];
        const posB = animatedPositionsRef.current[j];

        positions[lineIdx * 6 + 0] = posA.x;
        positions[lineIdx * 6 + 1] = posA.y;
        positions[lineIdx * 6 + 2] = posA.z;
        positions[lineIdx * 6 + 3] = posB.x;
        positions[lineIdx * 6 + 4] = posB.y;
        positions[lineIdx * 6 + 5] = posB.z;
        lineIdx++;
      }
    }
    lineGeom.attributes.position.needsUpdate = true;

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
          <bufferAttribute attach="attributes-opacity" {...opacityAttr} />
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
          <group
            key={i}
            ref={(el) => {
              nodeGroupRefs.current[i] = el;
            }}
            position={[node.position.x, node.position.y, node.position.z]}
          >
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
              color={isActive ? '#ffffff' : '#4488ff'}
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
