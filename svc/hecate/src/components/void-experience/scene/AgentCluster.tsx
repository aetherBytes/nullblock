import React, { useRef, useState, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import { Html } from '@react-three/drei';
import * as THREE from 'three';
import type { ClusterData } from '../VoidExperience';

interface AgentClusterProps {
  cluster: ClusterData;
  basePosition: [number, number, number];
  isHovered: boolean;
  onHover: (clusterId: string | null) => void;
  onClick: (cluster: ClusterData) => void;
  orbitPhase: number;
  orbitRadius: number;
  isVisible?: boolean;
  fadeDelay?: number;
}

const AgentCluster: React.FC<AgentClusterProps> = ({
  cluster,
  basePosition,
  isHovered,
  onHover,
  onClick,
  orbitPhase,
  orbitRadius,
  isVisible = true,
  fadeDelay = 0,
}) => {
  const groupRef = useRef<THREE.Group>(null);
  const meshRef = useRef<THREE.Mesh>(null);
  const particlesRef = useRef<THREE.Points>(null);
  const [showTooltip, setShowTooltip] = useState(false);
  const fadeStartTime = useRef<number | null>(null);

  // Determine cluster size based on type (Hecate is larger)
  const baseSize = cluster.name.toLowerCase().includes('hecate') ? 0.25 : 0.18;

  // When not visible, scale to 0. When visible, apply hover scale.
  const targetScale = isVisible ? (isHovered ? 1.3 : 1.0) : 0;

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

    // Track when visibility becomes true for staggered fade-in
    if (isVisible && fadeStartTime.current === null) {
      fadeStartTime.current = time;
    }
    if (!isVisible) {
      fadeStartTime.current = null;
    }

    if (groupRef.current) {
      // Orbital motion
      const x = Math.cos(time * 0.15 + orbitPhase) * orbitRadius;
      const z = Math.sin(time * 0.15 + orbitPhase) * orbitRadius;
      const y = basePosition[1] + Math.sin(time * 0.3 + orbitPhase) * 0.2;

      groupRef.current.position.set(x, y, z);

      // Calculate effective target scale with fade delay
      let effectiveTargetScale = targetScale;
      if (isVisible && fadeStartTime.current !== null) {
        const timeSinceVisible = time - fadeStartTime.current;
        if (timeSinceVisible < fadeDelay) {
          effectiveTargetScale = 0; // Still waiting for delay
        }
      }

      // Smooth scale transition with slower lerp for fade-in effect
      const currentScale = groupRef.current.scale.x;
      const lerpSpeed = isVisible ? 0.05 : 0.1; // Slower fade-in, faster fade-out
      const newScale = THREE.MathUtils.lerp(currentScale, effectiveTargetScale, lerpSpeed);
      groupRef.current.scale.setScalar(newScale);
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
      onPointerEnter={(e) => {
        e.stopPropagation();
        onHover(cluster.id);
        setShowTooltip(true);
      }}
      onPointerLeave={(e) => {
        e.stopPropagation();
        onHover(null);
        setShowTooltip(false);
      }}
      onClick={(e) => {
        e.stopPropagation();
        onClick(cluster);
      }}
    >
      {/* Core orb */}
      <mesh ref={meshRef} castShadow>
        <sphereGeometry args={[baseSize, 32, 32]} />
        <meshStandardMaterial
          color={cluster.color}
          emissive={cluster.color}
          emissiveIntensity={glowIntensity}
          metalness={0.5}
          roughness={0.3}
        />
      </mesh>

      {/* Outer glow */}
      <mesh>
        <sphereGeometry args={[baseSize * 1.4, 16, 16]} />
        <meshBasicMaterial
          color={cluster.color}
          transparent
          opacity={isHovered ? 0.2 : 0.1}
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
        intensity={isHovered ? 1.5 : 0.8}
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
