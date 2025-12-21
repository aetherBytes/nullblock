import React, { useRef, useState, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import { Html, useGLTF } from '@react-three/drei';
import * as THREE from 'three';
import type { ClusterData } from '../VoidExperience';

const HECATE_MODEL_PATH = '/models/hecate-orb.glb';

interface HecateModelProps {
  isInteractive: boolean;
  onPointerOver?: (e: { stopPropagation: () => void }) => void;
  onPointerOut?: (e: { stopPropagation: () => void }) => void;
  onClick?: (e: { stopPropagation: () => void }) => void;
}

const HecateModel: React.FC<HecateModelProps> = ({
  isInteractive,
  onPointerOver,
  onPointerOut,
  onClick,
}) => {
  const { scene } = useGLTF(HECATE_MODEL_PATH);
  const modelRef = useRef<THREE.Group>(null);

  const clonedScene = useMemo(() => {
    const clone = scene.clone();
    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        mesh.castShadow = true;
      }
    });
    return clone;
  }, [scene]);

  useFrame((state) => {
    if (modelRef.current) {
      modelRef.current.rotation.y = state.clock.elapsedTime * 0.3;
    }
  });

  return (
    <primitive
      ref={modelRef}
      object={clonedScene}
      scale={0.00225}
      rotation={[-0.3, 0, 0]}
      onPointerOver={isInteractive ? onPointerOver : undefined}
      onPointerOut={isInteractive ? onPointerOut : undefined}
      onClick={isInteractive ? onClick : undefined}
    />
  );
};

useGLTF.preload(HECATE_MODEL_PATH);

/**
 * AgentCluster - Major nodes representing AI Agents
 *
 * These are the primary interactive elements in the Void Experience:
 * - HECATE (vessel AI) - Gold glow, largest - your MK1 exploration companion
 * - Siren (marketing) - Purple accent
 * - Erebus (router) - Blue accent
 * - Other agents as discovered via /api/discovery/agents
 *
 * Agents orbit around the central CrossroadsOrb (the marketplace bazaar).
 * Clicking an agent freezes it in place and opens the ClusterPanel.
 */
interface AgentClusterProps {
  cluster: ClusterData;
  basePosition: [number, number, number];
  isHovered: boolean;
  isSelected?: boolean; // When selected, freeze orbital motion
  onHover: (clusterId: string | null) => void;
  onClick: (cluster: ClusterData, position: THREE.Vector3) => void;
  orbitPhase: number;
  orbitRadius: number;
  isInteractive?: boolean; // Controls whether cluster can be clicked/hovered
  fadeDelay?: number;
}

const AgentCluster: React.FC<AgentClusterProps> = ({
  cluster,
  basePosition,
  isHovered,
  isSelected = false,
  onHover,
  onClick,
  orbitPhase,
  orbitRadius,
  isInteractive = true,
  fadeDelay = 0,
}) => {
  const groupRef = useRef<THREE.Group>(null);
  const meshRef = useRef<THREE.Mesh>(null);
  const particlesRef = useRef<THREE.Points>(null);
  const [showTooltip, setShowTooltip] = useState(false);
  const fadeStartTime = useRef<number | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  // Store frozen position when selected
  const frozenPosition = useRef<THREE.Vector3 | null>(null);

  // Check if this is HECATE for special rendering
  const isHecate = cluster.name.toLowerCase().includes('hecate');

  // Determine cluster size based on type (Hecate is larger)
  const baseSize = isHecate ? 0.25 : 0.18;

  // No hover scale effect - just show at normal scale
  const targetScale = 1.0;

  // Create particle positions for nebula effect
  const particlePositions = useMemo(() => {
    const count = 50;
    const positions = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      const i3 = i * 3;
      const r = Math.random() * 0.3;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      positions[i3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i3 + 2] = r * Math.cos(phi);
    }
    return positions;
  }, []);

  // Animation
  useFrame((state) => {
    const time = state.clock.elapsedTime;

    // Track when component mounts for staggered fade-in
    if (fadeStartTime.current === null) {
      fadeStartTime.current = time;
    }

    if (groupRef.current) {
      // When selected, freeze at current position
      if (isSelected) {
        if (!frozenPosition.current) {
          // Capture current position when first selected
          frozenPosition.current = groupRef.current.position.clone();
        }
        // Stay at frozen position
        groupRef.current.position.copy(frozenPosition.current);
      } else {
        // Clear frozen position when deselected
        frozenPosition.current = null;

        // Normal orbital motion
        // Siren has a tilted orbit plane for extreme angle from HECATE
        const isSiren = cluster.name.toLowerCase().includes('siren');
        const animPhase = time * 0.15 + orbitPhase;

        let x, y, z;
        if (isSiren) {
          // Tilted orbit - more vertical
          x = Math.cos(animPhase) * orbitRadius * 0.3;
          z = Math.sin(animPhase) * orbitRadius;
          y = Math.cos(animPhase) * orbitRadius * 0.8;
        } else {
          x = Math.cos(animPhase) * orbitRadius;
          z = Math.sin(animPhase) * orbitRadius;
          y = basePosition[1] + Math.sin(time * 0.3 + orbitPhase) * 0.2;
        }

        groupRef.current.position.set(x, y, z);
      }

      // Calculate effective target scale with fade delay for initial appearance
      let effectiveTargetScale = targetScale;
      const timeSinceMount = time - fadeStartTime.current;
      if (timeSinceMount < fadeDelay) {
        effectiveTargetScale = 0; // Still waiting for delay
      } else if (!isVisible) {
        setIsVisible(true);
      }

      // Smooth scale transition
      const currentScale = groupRef.current.scale.x;
      const lerpSpeed = 0.12;
      const newScale = THREE.MathUtils.lerp(currentScale, effectiveTargetScale, lerpSpeed);

      // Set minimum scale after fade delay to ensure visibility
      const minScale = timeSinceMount >= fadeDelay ? 0.4 : 0;
      const finalScale = Math.max(newScale, minScale);
      groupRef.current.scale.setScalar(finalScale);
    }

    if (meshRef.current) {
      // Self-rotation
      meshRef.current.rotation.y = time * 0.3;
    }

    if (particlesRef.current) {
      // Particle swirl
      particlesRef.current.rotation.y = time * 0.2;
      particlesRef.current.rotation.x = Math.sin(time * 0.5) * 0.1;
    }
  });

  // Status-based glow intensity
  const glowIntensity = cluster.status === 'healthy' ? 0.8 :
                        cluster.status === 'unhealthy' ? 0.4 : 0.5;

  return (
    <group
      ref={groupRef}
      position={basePosition}
    >
      {/* Core orb - HECATE uses custom GLB model, others use sphere */}
      {isHecate ? (
        <HecateModel
          isInteractive={isInteractive}
          onPointerOver={(e) => {
            e.stopPropagation();
            onHover(cluster.id);
            setShowTooltip(true);
          }}
          onPointerOut={(e) => {
            e.stopPropagation();
            onHover(null);
            setShowTooltip(false);
          }}
          onClick={(e) => {
            e.stopPropagation();
            const worldPos = new THREE.Vector3();
            if (groupRef.current) {
              groupRef.current.getWorldPosition(worldPos);
            }
            onClick(cluster, worldPos);
          }}
        />
      ) : (
        <mesh
          ref={meshRef}
          castShadow
          onPointerOver={isInteractive ? (e) => {
            e.stopPropagation();
            onHover(cluster.id);
            setShowTooltip(true);
          } : undefined}
          onPointerOut={isInteractive ? (e) => {
            e.stopPropagation();
            onHover(null);
            setShowTooltip(false);
          } : undefined}
          onClick={isInteractive ? (e) => {
            e.stopPropagation();
            const worldPos = new THREE.Vector3();
            if (groupRef.current) {
              groupRef.current.getWorldPosition(worldPos);
            }
            onClick(cluster, worldPos);
          } : undefined}
        >
          <sphereGeometry args={[baseSize, 32, 32]} />
          <meshStandardMaterial
            color={cluster.color}
            emissive={cluster.color}
            emissiveIntensity={glowIntensity}
            metalness={0.5}
            roughness={0.3}
          />
        </mesh>
      )}

      {/* Outer glow */}
      <mesh>
        <sphereGeometry args={[baseSize * 1.4, 16, 16]} />
        <meshBasicMaterial
          color={cluster.color}
          transparent
          opacity={0.1}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Particle nebula */}
      <points ref={particlesRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={particlePositions.length / 3}
            array={particlePositions}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          color={cluster.color}
          size={0.02}
          transparent
          opacity={0.6}
          sizeAttenuation
          blending={THREE.AdditiveBlending}
          depthWrite={false}
        />
      </points>

      {/* Point light */}
      <pointLight
        color={cluster.color}
        intensity={0.8}
        distance={3}
        decay={2}
      />

      {/* Tooltip */}
      {showTooltip && (
        <Html
          position={[0, baseSize + 0.3, 0]}
          center
          style={{ pointerEvents: 'none' }}
        >
          <div style={{
            background: 'rgba(10, 10, 20, 0.9)',
            backdropFilter: 'blur(10px)',
            border: `1px solid ${cluster.color}40`,
            borderRadius: '8px',
            padding: '8px 12px',
            color: '#e8e8e8',
            fontSize: '12px',
            whiteSpace: 'nowrap',
            boxShadow: `0 0 20px ${cluster.color}30`,
          }}>
            <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>
              {cluster.name}
            </div>
            <div style={{
              color: cluster.status === 'healthy' ? '#00ff9d' :
                     cluster.status === 'unhealthy' ? '#ff3333' : '#e8e8e8',
              fontSize: '10px',
              textTransform: 'uppercase'
            }}>
              {cluster.status}
            </div>
          </div>
        </Html>
      )}
    </group>
  );
};

export default AgentCluster;
