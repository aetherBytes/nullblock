import React, { useRef, useMemo, useCallback } from 'react';
import { useFrame, useThree, extend } from '@react-three/fiber';
import * as THREE from 'three';
import type { ConstellationNode } from './VoidScene';

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
        precision highp float;

        uniform float uTime;
        uniform float uHover;
        varying vec3 vNormal;
        varying vec3 vPosition;
        varying vec2 vUv;

        // 3D Simplex noise - no seams on sphere
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

        // FBM using 3D noise - more octaves for smoothness
        float fbm3D(vec3 p) {
          float value = 0.0;
          float amplitude = 0.5;
          float frequency = 1.0;
          for (int i = 0; i < 7; i++) {
            value += amplitude * snoise3D(p * frequency);
            amplitude *= 0.48;
            frequency *= 2.1;
          }
          return value;
        }

        // Smooth hash for film grain
        float hash(vec2 p) {
          vec3 p3 = fract(vec3(p.xyx) * 0.1031);
          p3 += dot(p3, p3.yzx + 33.33);
          return fract((p3.x + p3.y) * p3.z);
        }

        // Smooth value noise for grain
        float valueNoise(vec2 p) {
          vec2 i = floor(p);
          vec2 f = fract(p);
          f = f * f * (3.0 - 2.0 * f); // Smooth interpolation

          float a = hash(i);
          float b = hash(i + vec2(1.0, 0.0));
          float c = hash(i + vec2(0.0, 1.0));
          float d = hash(i + vec2(1.0, 1.0));

          return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
        }

        void main() {
          float time = uTime * 0.15;

          // Use 3D position directly - no seams
          vec3 pos = normalize(vPosition);

          // Multiple 3D noise layers - seamless on sphere
          float noise1 = fbm3D(pos * 2.0 + vec3(time * 0.3, -time * 0.1, time * 0.2));
          float noise2 = fbm3D(pos * 4.0 + vec3(-time * 0.5, time * 0.2, -time * 0.15));
          float noise3 = snoise3D(pos * 8.0 + vec3(time * 0.8, -time * 0.4, time * 0.3));
          float noise4 = fbm3D(pos * 1.5 + vec3(time * 0.1, time * 0.15, -time * 0.1));

          float combinedNoise = noise1 * 0.4 + noise2 * 0.3 + noise3 * 0.2 + noise4 * 0.1;

          // Fresnel for rim/center detection
          float fresnel = 1.0 - abs(dot(vNormal, vec3(0.0, 0.0, 1.0)));
          float center = 1.0 - fresnel; // 1 at center, 0 at edge

          // === PURE WHITE SINGULARITY ===
          // Blinding white void - pure energy with pale blue core
          vec3 voidBlack = vec3(0.01, 0.01, 0.01);
          vec3 deepBlack = vec3(0.02, 0.02, 0.025);
          vec3 electricWhite = vec3(1.0, 1.0, 1.0);      // Pure white
          vec3 electricSilver = vec3(0.85, 0.88, 0.92);  // Silver shimmer
          vec3 paleBlue = vec3(0.7, 0.85, 1.0);          // Faint pale blue for core
          vec3 darkGray = vec3(0.04, 0.04, 0.05);        // Very dark gray tint
          vec3 electricRed = electricWhite;              // Alias for compatibility
          vec3 electricCrimson = electricSilver;         // Alias for compatibility
          vec3 darkRed = darkGray;                       // Alias for compatibility

          // Core intensity zones - extremely tight
          float coreIntensity = pow(center, 6.0);
          float innerCore = pow(center, 10.0);
          float deepCore = pow(center, 15.0);
          float absoluteCore = pow(center, 22.0); // Pinpoint nuclear center

          // Slow ambient pulse
          float slowPulse = sin(uTime * 0.3) * 0.5 + 0.5;

          // === LIQUID ELECTRIC LAYERS ===
          // Flowing 3D layers - like liquid plasma
          vec3 flowOffset1 = vec3(time * 0.4, -time * 0.3, time * 0.25);
          vec3 flowOffset2 = vec3(-time * 0.35, time * 0.28, -time * 0.2);
          vec3 flowOffset3 = vec3(time * 0.22, time * 0.18, time * 0.3);

          float liquid1 = fbm3D(pos * 3.0 + flowOffset1);
          float liquid2 = fbm3D(pos * 4.0 + flowOffset2);
          float liquid3 = fbm3D(pos * 2.5 + flowOffset3);

          // Very wide smoothstep for liquid feel - no hard edges
          float flow1 = smoothstep(-0.2, 0.6, liquid1);
          float flow2 = smoothstep(-0.15, 0.65, liquid2);
          float flow3 = smoothstep(-0.1, 0.7, liquid3);

          // Blend into smooth flowing field
          float electricField = (flow1 * 0.35 + flow2 * 0.35 + flow3 * 0.3);
          electricField = smoothstep(0.2, 0.8, electricField); // Extra smoothing
          electricField *= innerCore;

          // Gentle shimmer layer
          float shimmer = fbm3D(pos * 5.0 + vec3(time * 0.8, -time * 0.6, time * 0.5));
          float shimmerIntensity = smoothstep(0.0, 0.8, shimmer) * deepCore * 0.3;

          // === NUCLEAR CENTER ===
          // Pulsing core reactor
          float reactorPulse = sin(uTime * 1.5) * 0.5 + 0.5;
          float reactorBeat = pow(reactorPulse, 4.0);

          // Spinning energy using 3D noise - seamless
          float spinNoise = snoise3D(pos * 4.0 + vec3(uTime * 0.3, uTime * 0.2, -uTime * 0.25));
          float coreSpinner = smoothstep(0.0, 0.7, spinNoise) * absoluteCore;

          // Smooth concentric waves from center
          float wave1 = sin(center * 20.0 - uTime * 0.8) * 0.5 + 0.5;
          float wave2 = sin(center * 35.0 + uTime * 0.6) * 0.5 + 0.5;
          float waves = smoothstep(0.3, 0.7, wave1 * wave2) * deepCore;

          // === BUILD COLOR - ALMOST PURE BLACK ===
          vec3 coreColor = voidBlack;

          // Barely visible dark red ambient
          coreColor = mix(coreColor, darkRed, deepCore * 0.1 * slowPulse);

          // LIQUID ELECTRIC FLOW - smooth gradients
          coreColor += electricRed * electricField * 0.35;
          coreColor += electricCrimson * flow2 * innerCore * 0.2;

          // Shimmer highlights - gentle
          coreColor += electricRed * shimmerIntensity * 0.4;

          // Smooth glow falloff
          float smoothGlow = smoothstep(0.0, 1.0, electricField) * 0.03 * coreIntensity;
          coreColor += electricCrimson * smoothGlow;

          // === NUCLEAR CORE CENTER ===
          // Smooth waves - faint
          coreColor += electricCrimson * waves * 0.2;

          // Spinning core energy - dim
          coreColor += electricRed * coreSpinner * 0.5;

          // Pulsing nuclear heart - small bright point
          coreColor += electricRed * absoluteCore * reactorBeat * 1.5;

          // Constant core visibility - pinpoint
          coreColor += electricCrimson * absoluteCore * 0.4;
          coreColor += darkRed * deepCore * 0.08;

          // Faint pale blue tint at the very center
          coreColor = mix(coreColor, paleBlue, absoluteCore * 0.3);
          coreColor += paleBlue * pow(center, 18.0) * 0.15;

          // === ESCAPING LIGHT RIM (black hole edge) ===
          float innerRim = pow(fresnel, 4.0);
          float midRim = pow(fresnel, 2.0);
          float outerRim = pow(fresnel, 1.2);

          float rimNoise = 0.6 + noise1 * 0.3 + noise2 * 0.2;
          float flareNoise = smoothstep(0.3, 0.8, noise1 + noise3 * 0.5);

          // Rim colors - pure blinding white
          vec3 brightWhite = vec3(1.0, 1.0, 1.0);
          vec3 coldLight = vec3(0.95, 0.97, 1.0);
          vec3 hotEdge = vec3(0.9, 0.92, 0.95);

          vec3 rimColor = vec3(0.0);
          rimColor += hotEdge * innerRim * rimNoise * 1.5;
          rimColor += coldLight * midRim * rimNoise * 0.6;
          rimColor += brightWhite * outerRim * 0.25;
          rimColor += brightWhite * flareNoise * innerRim * 1.0;

          // === COMBINE ALL ===
          vec3 color = vec3(0.0);

          // Dark energy core (center) - mostly black with lightning
          color += coreColor * smoothstep(0.4, 0.85, center);

          // Escaping light rim (edge) - the only bright part
          color += rimColor;

          // Subtle global pulse
          float globalPulse = 1.0 + sin(uTime * 0.25) * 0.02;
          color *= globalPulse;

          // Rare electric surge - faint
          float surgeCycle = pow(sin(uTime * 0.1 + noise1 * 2.0) * 0.5 + 0.5, 18.0);
          color += electricRed * surgeCycle * innerCore * 0.2;

          // Smooth film grain to break up any grid patterns
          float grain1 = valueNoise(gl_FragCoord.xy * 0.5 + uTime * 10.0);
          float grain2 = valueNoise(gl_FragCoord.xy * 0.8 - uTime * 8.0);
          float grain3 = valueNoise(gl_FragCoord.xy * 1.2 + uTime * 6.0);
          float smoothGrain = (grain1 + grain2 + grain3) / 3.0;

          // Apply subtle grain
          color += (smoothGrain - 0.5) * 0.015;

          // Extra smooth dither layer
          float microGrain = valueNoise(gl_FragCoord.xy * 2.0 + uTime * 20.0);
          color += (microGrain - 0.5) * 0.005;

          gl_FragColor = vec4(max(color, vec3(0.0)), 1.0);
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

// Dendrite/energy tendril shader - grows from surface outward like tentacles
class DendriteMaterial extends THREE.ShaderMaterial {
  constructor() {
    super({
      uniforms: {
        uTime: { value: 0 },
        uGrowth: { value: 0 }, // 0 = not visible, 1 = fully grown
        uFade: { value: 1 },   // 1 = visible, 0 = faded out
        uThickness: { value: 1.0 }, // Base thickness multiplier (randomized per tendril)
        uDirection: { value: 1.0 }, // 1.0 = outward (sending), -1.0 = inward (receiving)
        uOpacity: { value: 1.0 }, // Random opacity per tendril
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

          // With -Y to d.dir rotation: uv.y=1 is BASE (near orb), uv.y=0 is TIP (far)
          // Taper direction depends on growth direction:
          // - Outward (uDirection=1): thick at base (origin), thin at tip (destination)
          // - Inward (uDirection=-1): thick at tip (origin), thin at base (destination)
          float taperProgress = uDirection > 0.0 ? vProgress : 1.0 - vProgress;
          float taperCurve = pow(taperProgress, 0.6);
          vTaper = taperCurve;

          // Apply taper to XZ (radius) for cylindrical geometry
          float taperAmount = 0.08 + 0.92 * taperCurve;
          pos.x *= taperAmount * uThickness;
          pos.z *= taperAmount * uThickness;

          // Organic wave motion - in the middle section only
          // Pin both ends: origin (thick end) and destination (thin end/tip)
          float thinness = 1.0 - taperProgress;

          // Reduce wave at the very tip (last 15%) so it stays affixed to the node
          float tipPinning = smoothstep(0.0, 0.15, taperProgress); // 0 at tip, 1 elsewhere

          // Also reduce wave near the base (first 15%)
          float basePinning = smoothstep(0.0, 0.15, thinness); // 0 at base, 1 elsewhere

          // Wave is strongest in the middle, pinned at both ends
          float waveStrength = thinness * thinness * tipPinning * basePinning;

          float wave1 = sin(thinness * 5.0 + uTime * 2.5) * 0.06 * waveStrength;
          float wave2 = sin(thinness * 8.0 - uTime * 3.5) * 0.04 * waveStrength;
          pos.x += wave1;
          pos.z += wave2;

          // Slight spiral motion - also pinned at ends
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
        uniform float uDirection; // 1.0 = outward, -1.0 = inward
        uniform float uOpacity;   // Random opacity per tendril
        varying vec2 vUv;
        varying float vProgress;
        varying float vTaper;
        varying vec3 vNormal;

        // Simplex noise (matching corona ring shader)
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
          float time = uTime * 0.08; // Slower, more dreamlike

          // Growth direction
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

          // Soft fresnel for ethereal glow
          vec3 viewDir = vec3(0.0, 0.0, 1.0);
          float fresnel = 1.0 - abs(dot(vNormal, viewDir));
          fresnel = pow(fresnel, 2.0); // Softer falloff

          // Multi-layer flowing noise for ethereal wisps
          float pulseDir = uDirection > 0.0 ? 1.0 : -1.0;
          vec2 flowUV = vec2(vProgress * 3.0, vUv.x * 1.5);

          // Slow drifting layers
          float noise1 = snoise(flowUV * 6.0 + vec2(time * 0.2 * pulseDir, time * 0.08));
          float noise2 = snoise(flowUV * 12.0 + vec2(-time * 0.15, time * 0.12) - noise1 * 0.2);
          float noise3 = snoise(flowUV * 24.0 + vec2(time * 0.1, -time * 0.06)); // Fine detail

          // Create soft, wispy effect
          float wisp = (noise1 * 0.5 + noise2 * 0.35 + noise3 * 0.15) * 0.5 + 0.5;
          wisp = smoothstep(0.15, 0.85, wisp); // Very soft edges

          // Add shimmer - subtle twinkling
          float shimmer = snoise(flowUV * 40.0 + vec2(uTime * 0.8, uTime * 0.6));
          shimmer = pow(max(shimmer, 0.0), 3.0) * 0.3;

          // Very gentle pulse - almost breathing
          float pulse = 0.85 + 0.15 * sin(uTime * 0.3 + vProgress * 4.0 * pulseDir);

          // Soft tip glow at growth front
          float tipDist = abs(vProgress - growthFront);
          float tipGlow = 1.0 - smoothstep(0.0, 0.2, tipDist);
          tipGlow = pow(tipGlow, 1.5); // Softer glow

          // Ethereal edge fade - more transparent at edges
          float edgeFade = 1.0 - fresnel * 0.6;

          // Taper creates gentle brightness variation
          float taperBrightness = 0.6 + vTaper * 0.4;

          // Combine for ethereal intensity
          float intensity = wisp * pulse * taperBrightness * edgeFade;
          intensity += shimmer * wisp; // Shimmer only where visible
          intensity += tipGlow * 0.4;
          intensity *= 0.7; // Overall softer
          intensity *= uFade;
          intensity *= uOpacity;

          // Ethereal color - pale, ghostly white with subtle warmth
          vec3 baseColor = vec3(1.0, 0.98, 0.96);

          // Subtle direction tint - barely perceptible
          vec3 tintOut = vec3(0.97, 0.98, 1.0);  // Ghostly cool
          vec3 tintIn = vec3(1.0, 0.99, 0.97);   // Ghostly warm
          vec3 color = uDirection > 0.0 ? mix(baseColor, tintOut, 0.2) : mix(baseColor, tintIn, 0.2);

          // Soft fresnel glow
          color += vec3(0.1, 0.08, 0.06) * fresnel;

          // Ethereal tip glow
          color += vec3(0.2, 0.18, 0.16) * tipGlow;

          // Apply intensity
          color *= intensity;

          // Very soft alpha for ethereal transparency
          float alpha = intensity * 0.65;

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

extend({ SunSurfaceMaterial, CoronaRingMaterial, SolarFlareMaterial, DendriteMaterial });

interface CrossroadsOrbProps {
  position?: [number, number, number];
  constellationNodes?: ConstellationNode[];
  onActiveNodesChange?: (activeNodes: Set<number>) => void;
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

const CrossroadsOrb: React.FC<CrossroadsOrbProps> = ({ position = [0, 0, 0], constellationNodes = [], onActiveNodesChange }) => {
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
  const dendritesRef = useRef<THREE.Group>(null);
  const dendriteMaterialsRef = useRef<DendriteMaterial[]>([]);

  const { camera, pointer, raycaster } = useThree();
  const mouseWorld = useRef(new THREE.Vector3());
  const isHovered = useRef(false);
  const hoverAmount = useRef(0);

  const sunRadius = 1.6;
  const brightParticleCount = 250;
  const streamParticleCount = 200;
  const flareCount = 8;
  const dendriteCount = 12;

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

  // Dynamic dendrite state - each tendril has a lifecycle
  interface DendriteState {
    targetNodeIndex: number; // Index of constellation node this tendril connects to
    length: number;
    width: number;
    thickness: number;   // Random thickness multiplier for variety
    opacity: number;     // Random opacity per tendril
    dir: THREE.Vector3;
    quaternion: THREE.Quaternion;
    // Lifecycle
    state: 'growing' | 'holding' | 'fading' | 'waiting';
    growth: number;      // 0-1 how much has grown
    fade: number;        // 1-0 fade out
    lifetime: number;    // How long to hold
    elapsed: number;     // Time in current state
    growSpeed: number;   // How fast to grow
    growsOutward: boolean; // true = outward (sending), false = inward (receiving)
    lingers: boolean;    // true = stays connected much longer
  }

  const dendritesState = useRef<DendriteState[]>([]);
  const prevActiveNodes = useRef<Set<number>>(new Set());

  // Ring alignment state - rings periodically align then return to normal rotation
  const alignmentState = useRef({
    isAligning: false,
    alignProgress: 0, // 0 = normal rotation, 1 = fully aligned
    holdTime: 0,
    nextAlignTime: 8 + Math.random() * 12, // First alignment in 8-20 seconds
    // Store the rotation offsets to blend back to
    ring1BaseRot: { x: 0, z: 0 },
    ring2BaseRot: { z: 0, y: 0 },
    ring3BaseRot: { y: 0, x: 0 },
  });

  // Initialize dendrite slots with well-staggered timing
  useMemo(() => {
    dendritesState.current = [];
    for (let i = 0; i < dendriteCount; i++) {
      dendritesState.current.push({
        targetNodeIndex: -1,
        length: 2.0,
        width: 0.1,
        thickness: 1.0,
        opacity: 1.0,
        dir: new THREE.Vector3(0, 1, 0),
        quaternion: new THREE.Quaternion(),
        state: 'waiting',
        growth: 0,
        fade: 1,
        lifetime: 0,
        // Spread initial spawns over 0-10 seconds for natural staggering
        elapsed: (i / dendriteCount) * 6 + Math.random() * 4,
        growSpeed: 0.5,
        growsOutward: true,
        lingers: false,
      });
    }
  }, [dendriteCount]);

  // Function to spawn a new tendril targeting a constellation node
  const spawnDendrite = useCallback((d: DendriteState) => {
    // If no constellation nodes, fall back to random direction
    if (constellationNodes.length === 0) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.PI * 0.2 + Math.random() * Math.PI * 0.6;
      d.dir = new THREE.Vector3(
        Math.sin(phi) * Math.cos(theta),
        Math.cos(phi),
        Math.sin(phi) * Math.sin(theta)
      ).normalize();
      d.length = 2.0 + Math.random() * 3.0;
      d.targetNodeIndex = -1;
    } else {
      // Pick a random constellation node as target
      d.targetNodeIndex = Math.floor(Math.random() * constellationNodes.length);
      const targetNode = constellationNodes[d.targetNodeIndex];

      // Direction from orb center to constellation node
      d.dir = targetNode.position.clone().normalize();

      // Calculate exact length needed to reach the node from orb surface
      // Tip position = sunRadius + length (along direction)
      // We want tip to reach the node, so: sunRadius + length = distToNode
      // Therefore: length = distToNode - sunRadius
      const distToNode = targetNode.position.length();
      d.length = Math.max(1.5, distToNode - sunRadius);
    }

    d.width = 0.15 + Math.random() * 0.1; // Base width 0.15-0.25 (slightly thicker)
    d.thickness = 0.6 + Math.random() * 0.4; // Thickness variation 0.6-1.0
    d.opacity = 0.6 + Math.random() * 0.4; // Random opacity 0.6-1.0 (more visible)

    // Random direction: outward (sending) or inward (receiving)
    d.growsOutward = Math.random() > 0.4; // 60% outward, 40% inward

    // Some tendrils linger much longer (30% chance)
    d.lingers = Math.random() < 0.3;

    // Calculate rotation to point the tendril outward
    d.quaternion = new THREE.Quaternion();
    d.quaternion.setFromUnitVectors(new THREE.Vector3(0, -1, 0), d.dir);

    d.state = 'growing';
    d.growth = 0;
    d.fade = 1;

    // Lingering tendrils stay connected 3-8 seconds, normal ones 0.8-4.8 seconds
    if (d.lingers) {
      d.lifetime = 3.0 + Math.random() * 5.0;
    } else {
      d.lifetime = 0.8 + Math.random() * 4.0;
    }

    d.elapsed = 0;
    d.growSpeed = 0.3 + Math.random() * 0.9; // Growth speed 0.3-1.2
  }, [constellationNodes, sunRadius]);

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

    // Animate corona shells - TRUE GYROSCOPE EFFECT with periodic alignment
    const align = alignmentState.current;

    // Update alignment timer
    align.nextAlignTime -= delta;

    if (!align.isAligning && align.nextAlignTime <= 0) {
      // Start alignment sequence
      align.isAligning = true;
      align.alignProgress = 0;
      align.holdTime = 0;
      // Store current rotations as base to blend from
      if (coronaGroupRef.current) {
        align.ring1BaseRot.x = coronaGroupRef.current.rotation.x;
        align.ring1BaseRot.z = coronaGroupRef.current.rotation.z;
      }
      if (corona2GroupRef.current) {
        align.ring2BaseRot.z = corona2GroupRef.current.rotation.z;
        align.ring2BaseRot.y = corona2GroupRef.current.rotation.y;
      }
      if (corona3GroupRef.current) {
        align.ring3BaseRot.y = corona3GroupRef.current.rotation.y;
        align.ring3BaseRot.x = corona3GroupRef.current.rotation.x;
      }
    }

    if (align.isAligning) {
      if (align.alignProgress < 1) {
        // Ease into alignment
        align.alignProgress = Math.min(1, align.alignProgress + delta * 0.8);
        const ease = 1 - Math.pow(1 - align.alignProgress, 3); // Ease out cubic

        // Target alignment: all rings flat (rotation 0 on their tilt axes)
        // But keep spinning on Y axis together
        const sharedSpin = time * 0.15;

        if (coronaGroupRef.current) {
          coronaGroupRef.current.rotation.x = align.ring1BaseRot.x * (1 - ease);
          coronaGroupRef.current.rotation.z = align.ring1BaseRot.z * (1 - ease);
          coronaGroupRef.current.rotation.y = sharedSpin;
        }
        if (corona2GroupRef.current) {
          // Blend from tilted to flat
          const baseTiltX = Math.PI / 3;
          corona2GroupRef.current.rotation.x = baseTiltX * (1 - ease);
          corona2GroupRef.current.rotation.z = align.ring2BaseRot.z * (1 - ease);
          corona2GroupRef.current.rotation.y = sharedSpin;
        }
        if (corona3GroupRef.current) {
          // Blend from tilted to flat
          const baseTiltZ = Math.PI / 3;
          corona3GroupRef.current.rotation.z = baseTiltZ * (1 - ease);
          corona3GroupRef.current.rotation.x = align.ring3BaseRot.x * (1 - ease);
          corona3GroupRef.current.rotation.y = sharedSpin;
        }
      } else {
        // Hold alignment briefly (0.8-1.5 seconds)
        align.holdTime += delta;
        const sharedSpin = time * 0.15;

        // Keep spinning together while aligned
        if (coronaGroupRef.current) coronaGroupRef.current.rotation.y = sharedSpin;
        if (corona2GroupRef.current) corona2GroupRef.current.rotation.y = sharedSpin;
        if (corona3GroupRef.current) corona3GroupRef.current.rotation.y = sharedSpin;

        if (align.holdTime > 0.8 + Math.random() * 0.7) {
          // End alignment, schedule next one
          align.isAligning = false;
          align.alignProgress = 0;
          align.nextAlignTime = 15 + Math.random() * 25; // Next alignment in 15-40 seconds
        }
      }
    } else {
      // Normal gyroscope rotation
      if (coronaGroupRef.current) {
        coronaGroupRef.current.rotation.x += delta * 0.3;
        coronaGroupRef.current.rotation.z += delta * 0.1;
      }
      if (corona2GroupRef.current) {
        corona2GroupRef.current.rotation.z -= delta * 0.25;
        corona2GroupRef.current.rotation.y += delta * 0.12;
      }
      if (corona3GroupRef.current) {
        corona3GroupRef.current.rotation.y += delta * 0.18;
        corona3GroupRef.current.rotation.x -= delta * 0.15;
      }
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

    // Animate dendrites with dynamic lifecycle
    dendritesState.current.forEach((d, i) => {
      const material = dendriteMaterialsRef.current[i];

      switch (d.state) {
        case 'waiting':
          d.elapsed += delta;
          // Random spawn after waiting period - more staggered (2-8 seconds)
          if (d.elapsed > 2.0 + Math.random() * 6.0) {
            spawnDendrite(d);
          }
          if (material) {
            material.uniforms.uGrowth.value = 0;
            material.uniforms.uFade.value = 0;
          }
          break;

        case 'growing':
          d.elapsed += delta;
          d.growth = Math.min(1.0, d.growth + delta * d.growSpeed);
          if (material) {
            material.uniforms.uTime.value = time;
            material.uniforms.uGrowth.value = d.growth;
            material.uniforms.uFade.value = 1;
            material.uniforms.uDirection.value = d.growsOutward ? 1.0 : -1.0;
            material.uniforms.uOpacity.value = d.opacity;
          }
          // Transition to holding when fully grown
          if (d.growth >= 1.0) {
            d.state = 'holding';
            d.elapsed = 0;
          }
          break;

        case 'holding':
          d.elapsed += delta;
          if (material) {
            material.uniforms.uTime.value = time;
            material.uniforms.uGrowth.value = 1;
            material.uniforms.uFade.value = 1;
            material.uniforms.uDirection.value = d.growsOutward ? 1.0 : -1.0;
            material.uniforms.uOpacity.value = d.opacity;
          }
          // Transition to fading after hold time
          if (d.elapsed > d.lifetime) {
            d.state = 'fading';
            d.elapsed = 0;
          }
          break;

        case 'fading':
          d.elapsed += delta;
          d.fade = Math.max(0, d.fade - delta * 1.2); // Slower fade
          if (material) {
            material.uniforms.uTime.value = time;
            material.uniforms.uGrowth.value = 1;
            material.uniforms.uFade.value = d.fade;
            material.uniforms.uDirection.value = d.growsOutward ? 1.0 : -1.0;
            material.uniforms.uOpacity.value = d.opacity;
          }
          // Transition to waiting when fully faded
          if (d.fade <= 0) {
            d.state = 'waiting';
            d.elapsed = Math.random() * 4; // Random wait before next spawn
          }
          break;
      }
    });

    // Track which constellation nodes have active tendrils connected
    if (onActiveNodesChange) {
      const activeNodes = new Set<number>();
      dendritesState.current.forEach((d) => {
        // Node is active if tendril is growing, holding, or fading (still visible)
        if (d.targetNodeIndex >= 0 && d.state !== 'waiting') {
          activeNodes.add(d.targetNodeIndex);
        }
      });

      // Only call callback if the set changed
      const prevSet = prevActiveNodes.current;
      const hasChanged = activeNodes.size !== prevSet.size ||
        [...activeNodes].some(n => !prevSet.has(n));

      if (hasChanged) {
        prevActiveNodes.current = activeNodes;
        onActiveNodesChange(activeNodes);
      }
    }

    // Update mesh positions and rotations based on state
    if (dendritesRef.current) {
      dendritesRef.current.children.forEach((mesh, i) => {
        const d = dendritesState.current[i];
        const material = dendriteMaterialsRef.current[i];
        if (!d) return;

        // Control visibility based on state
        mesh.visible = d.state !== 'waiting';

        if (d.state !== 'waiting') {
          // Position: CENTER of tendril mesh, with base at surface
          // Plane geometry: center at origin, extends -height/2 to +height/2 in local Y
          // After scaling by d.length/2.0, actual height = d.length
          // To place BASE at surface: center = surface + halfLength along direction
          const meshCenter = sunRadius + d.length * 0.5;
          mesh.position.set(
            d.dir.x * meshCenter,
            d.dir.y * meshCenter,
            d.dir.z * meshCenter
          );

          // Apply stored quaternion to point local +Y toward d.dir (outward from sphere)
          mesh.quaternion.copy(d.quaternion);

          // Scale: width and length relative to base geometry (0.08 x 2.0)
          const widthScale = d.width / 0.08;
          const lengthScale = d.length / 2.0;
          mesh.scale.set(widthScale, lengthScale, 1);

          // Update thickness uniform for shader-based taper variation
          if (material) {
            material.uniforms.uThickness.value = d.thickness;
          }
        }
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
        <sphereGeometry args={[sunRadius, 128, 128]} />
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

      {/* Dendrites/wisps - flowing energy tendrils (dynamic lifecycle) */}
      <group ref={dendritesRef}>
        {Array.from({ length: dendriteCount }).map((_, i) => (
          <mesh
            key={`dendrite-${i}`}
            position={[0, sunRadius + 1, 0]}
          >
            {/* Cylinder: radiusTop, radiusBottom, height, radialSegments, heightSegments */}
            {/* Using same radius for both - shader handles the taper */}
            <cylinderGeometry args={[0.04, 0.04, 2.0, 8, 24]} />
            {/* @ts-ignore */}
            <dendriteMaterial
              ref={(el: DendriteMaterial | null) => {
                if (el) dendriteMaterialsRef.current[i] = el;
              }}
            />
          </mesh>
        ))}
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

      {/* Main pure white light */}
      <pointLight color="#ffffff" intensity={10} distance={18} decay={2} />

      {/* Silver accent light */}
      <pointLight color="#e8eef5" intensity={4} distance={10} decay={2} />
    </group>
  );
};

export default CrossroadsOrb;
