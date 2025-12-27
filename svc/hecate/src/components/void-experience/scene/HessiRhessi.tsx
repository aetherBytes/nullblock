import React, { useRef, useMemo, useEffect } from 'react';
import { useFrame, useThree } from '@react-three/fiber';
import { useGLTF } from '@react-three/drei';
import * as THREE from 'three';

const HESSI_MODEL_PATH = '/models/HESSI-RHESSI.glb';

interface HessiRhessiProps {
  onPositionUpdate?: (position: THREE.Vector3) => void;
  onHoverChange?: (isHovered: boolean) => void;
  isActive?: boolean;
  isCharging?: boolean;
  isProcessing?: boolean;
  isReceiving?: boolean;
}

// Create a soft blue-white glow texture for HESSI-RHESSI
const createHessiGlowTexture = () => {
  const size = 256;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(
    size / 2, size / 2, 0,
    size / 2, size / 2, size / 2
  );
  // Blue-white ethereal glow
  gradient.addColorStop(0, 'rgba(200, 220, 255, 1.0)');
  gradient.addColorStop(0.15, 'rgba(150, 190, 250, 0.7)');
  gradient.addColorStop(0.3, 'rgba(100, 160, 230, 0.4)');
  gradient.addColorStop(0.5, 'rgba(74, 140, 210, 0.2)');
  gradient.addColorStop(0.7, 'rgba(50, 120, 190, 0.08)');
  gradient.addColorStop(1, 'rgba(40, 100, 170, 0)');

  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

const HessiRhessi: React.FC<HessiRhessiProps> = ({
  onPositionUpdate,
  onHoverChange,
  isActive = true,
  isCharging = false,
  isProcessing = false,
  isReceiving = false,
}) => {
  const { camera } = useThree();
  const { scene } = useGLTF(HESSI_MODEL_PATH);
  const groupRef = useRef<THREE.Group>(null);
  const modelRef = useRef<THREE.Group>(null);
  const glowRef = useRef<THREE.Sprite>(null);
  const outerGlowRef = useRef<THREE.Sprite>(null);
  const lightRef = useRef<THREE.PointLight>(null);

  const glowTexture = useMemo(() => createHessiGlowTexture(), []);

  // Clone the scene - preserve original textures
  const clonedScene = useMemo(() => {
    const clone = scene.clone();
    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        mesh.castShadow = true;
        // Clone material to avoid modifying the original
        if (mesh.material) {
          mesh.material = (mesh.material as THREE.Material).clone();
        }
      }
    });
    return clone;
  }, [scene]);

  // Position relative to camera - fixed in screen space
  // Left of chat input, floating in 3D space
  const targetPosition = useRef(new THREE.Vector3());
  const currentPosition = useRef(new THREE.Vector3());

  useEffect(() => {
    // Initialize position
    if (groupRef.current) {
      currentPosition.current.copy(groupRef.current.position);
    }
  }, []);

  useFrame((state, delta) => {
    const time = state.clock.elapsedTime;

    if (groupRef.current) {
      // Calculate world position based on camera
      // Position to the RIGHT of the screen, near the chat input
      // Responsive x position: adjust based on screen size
      const viewportWidth = state.size.width;
      // On large screens (>1024px): x=0.52 (close to chat on right)
      // On medium screens (768-1024px): x=0.62 (more padding)
      // On small screens (<768px, iPad/tablet): x=0.72 (significant padding)
      let ndcX = 0.52;
      if (viewportWidth < 768) {
        ndcX = 0.72;
      } else if (viewportWidth < 1024) {
        ndcX = 0.62;
      }
      const ndc = new THREE.Vector3(ndcX, -0.75, 0.5);
      ndc.unproject(camera);
      const dir = ndc.sub(camera.position).normalize();
      const distance = 4; // Closer to camera for smaller appearance
      targetPosition.current.copy(camera.position).add(dir.multiplyScalar(distance));

      // Smooth position follow
      currentPosition.current.lerp(targetPosition.current, delta * 3);
      groupRef.current.position.copy(currentPosition.current);

      // Add gentle float animation (subtle for small model)
      const floatY = Math.sin(time * 0.8) * 0.01;
      const floatX = Math.cos(time * 0.5) * 0.005;
      groupRef.current.position.y += floatY;
      groupRef.current.position.x += floatX;

      // Report position for tendril targeting
      if (onPositionUpdate) {
        const worldPos = new THREE.Vector3();
        groupRef.current.getWorldPosition(worldPos);
        onPositionUpdate(worldPos);
      }
    }

    // Model rotation - gentle spin
    if (modelRef.current) {
      modelRef.current.rotation.y += delta * 0.2;
      // Slight wobble
      modelRef.current.rotation.x = Math.sin(time * 0.3) * 0.05;
      modelRef.current.rotation.z = Math.cos(time * 0.4) * 0.03;
    }

    // Glow effects - pulse based on state
    const basePulse = Math.sin(time * 1.5) * 0.15;
    const chargingIntensity = isCharging ? 1.5 + Math.sin(time * 8) * 0.5 : 1.0;
    const processingPulse = isProcessing ? 1.2 + Math.sin(time * 3) * 0.3 : 1.0;
    // Receiving creates a bright flash that fades
    const receivingIntensity = isReceiving ? 2.5 + Math.sin(time * 12) * 0.8 : 1.0;
    const stateMultiplier = chargingIntensity * processingPulse * receivingIntensity;

    if (glowRef.current) {
      const scale = (0.045 + basePulse * 0.009) * stateMultiplier;
      glowRef.current.scale.set(scale, scale, 1);
      (glowRef.current.material as THREE.SpriteMaterial).opacity =
        isActive ? (0.7 * stateMultiplier) : 0.3;
    }

    if (outerGlowRef.current) {
      // When receiving, the outer glow expands significantly
      const receivingScale = isReceiving ? 1.8 : 1.0;
      const outerScale = (0.073 + basePulse * 0.014) * stateMultiplier * receivingScale;
      outerGlowRef.current.scale.set(outerScale, outerScale, 1);
      (outerGlowRef.current.material as THREE.SpriteMaterial).opacity =
        isActive ? (0.35 * stateMultiplier) : 0.15;
    }

    // Light intensity
    if (lightRef.current) {
      const baseIntensity = 1.5 + Math.sin(time * 2) * 0.3;
      lightRef.current.intensity = baseIntensity * stateMultiplier;
      // Color shift based on state
      if (isReceiving) {
        // Bright white-blue flash when receiving transmission
        lightRef.current.color.setHex(0xffffff);
      } else if (isCharging) {
        lightRef.current.color.setHex(0x6ab4ff);
      } else if (isProcessing) {
        lightRef.current.color.setHex(0x88ccff);
      } else {
        lightRef.current.color.setHex(0x4a9eff);
      }
    }
  });

  return (
    <group ref={groupRef}>
      {/* Invisible hit area for hover detection */}
      <mesh
        onPointerOver={() => onHoverChange?.(true)}
        onPointerOut={() => onHoverChange?.(false)}
      >
        <sphereGeometry args={[0.04, 16, 16]} />
        <meshBasicMaterial transparent opacity={0} depthWrite={false} />
      </mesh>

      {/* Outer atmospheric glow */}
      <sprite ref={outerGlowRef} scale={[0.073, 0.073, 1]} position={[0, 0, -0.001]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.35}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Inner core glow */}
      <sprite ref={glowRef} scale={[0.045, 0.045, 1]} position={[0, 0, -0.0005]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.7}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* The HESSI-RHESSI model */}
      <primitive
        ref={modelRef}
        object={clonedScene}
        scale={0.00056}
        rotation={[0, 0, 0]}
      />

      {/* Point light for visibility and reflections */}
      <pointLight
        ref={lightRef}
        color="#4a9eff"
        intensity={0.3}
        distance={0.5}
        decay={2}
      />
    </group>
  );
};

useGLTF.preload(HESSI_MODEL_PATH);

export default HessiRhessi;
