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

// Create a soft glow texture for HECATE
const createHecateGlowTexture = () => {
  const size = 256;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  const gradient = ctx.createRadialGradient(
    size / 2, size / 2, 0,
    size / 2, size / 2, size / 2
  );
  // Brighter steel-blue glow for visibility
  gradient.addColorStop(0, 'rgba(180, 210, 255, 1.0)');
  gradient.addColorStop(0.1, 'rgba(150, 190, 240, 0.8)');
  gradient.addColorStop(0.2, 'rgba(130, 170, 220, 0.5)');
  gradient.addColorStop(0.35, 'rgba(110, 150, 200, 0.3)');
  gradient.addColorStop(0.5, 'rgba(90, 130, 185, 0.15)');
  gradient.addColorStop(0.7, 'rgba(70, 110, 165, 0.05)');
  gradient.addColorStop(1, 'rgba(60, 100, 150, 0)');

  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

// Create a ping/beacon ring texture
const createPingTexture = () => {
  const size = 256;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  // Ring shape - bright at a specific radius, fading in and out
  const centerX = size / 2;
  const centerY = size / 2;
  const ringRadius = size * 0.35;
  const ringWidth = size * 0.08;

  const gradient = ctx.createRadialGradient(
    centerX, centerY, ringRadius - ringWidth,
    centerX, centerY, ringRadius + ringWidth
  );
  gradient.addColorStop(0, 'rgba(150, 200, 255, 0)');
  gradient.addColorStop(0.3, 'rgba(180, 220, 255, 0.8)');
  gradient.addColorStop(0.5, 'rgba(200, 230, 255, 1.0)');
  gradient.addColorStop(0.7, 'rgba(180, 220, 255, 0.8)');
  gradient.addColorStop(1, 'rgba(150, 200, 255, 0)');

  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
};

const HecateModel: React.FC<HecateModelProps> = ({
  isInteractive,
  onPointerOver,
  onPointerOut,
  onClick,
}) => {
  const { scene } = useGLTF(HECATE_MODEL_PATH);
  const modelRef = useRef<THREE.Group>(null);
  const glowRef = useRef<THREE.Sprite>(null);
  const outerGlowRef = useRef<THREE.Sprite>(null);
  const ping1Ref = useRef<THREE.Sprite>(null);
  const ping2Ref = useRef<THREE.Sprite>(null);
  const orbitalRef = useRef<THREE.Group>(null);
  const lightRef = useRef<THREE.PointLight>(null);

  const glowTexture = useMemo(() => createHecateGlowTexture(), []);
  const pingTexture = useMemo(() => createPingTexture(), []);

  // Ping animation state - triggers periodically
  const pingState = useRef({
    ping1Phase: -1, // -1 = inactive
    ping2Phase: -1,
    nextPingTime: 4 + Math.random() * 3, // First ping in 4-7 seconds
    isPinging: false,
  });

  const clonedScene = useMemo(() => {
    const clone = scene.clone();
    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        mesh.castShadow = true;
        // Enhance material for better reflections from Crossroads light
        if (mesh.material) {
          const mat = mesh.material as THREE.MeshStandardMaterial;
          if (mat.isMeshStandardMaterial) {
            mat.envMapIntensity = 2.0;
            mat.metalness = Math.min(mat.metalness + 0.3, 1.0);
            mat.roughness = Math.max(mat.roughness - 0.15, 0.05);
            mat.needsUpdate = true;
          }
        }
      }
    });
    return clone;
  }, [scene]);

  // Orbital particle positions
  const orbitalParticles = useMemo(() => {
    const particles: { angle: number; radius: number; speed: number; yOffset: number }[] = [];
    for (let i = 0; i < 6; i++) {
      particles.push({
        angle: (i / 6) * Math.PI * 2,
        radius: 0.35 + Math.random() * 0.1,
        speed: 0.8 + Math.random() * 0.4,
        yOffset: (Math.random() - 0.5) * 0.15,
      });
    }
    return particles;
  }, []);

  useFrame((state, delta) => {
    const time = state.clock.elapsedTime;

    if (modelRef.current) {
      modelRef.current.rotation.y = time * 0.3;
    }

    // Inner glow pulse
    if (glowRef.current) {
      const pulse = 2.2 + Math.sin(time * 1.5) * 0.2;
      glowRef.current.scale.set(pulse, pulse, 1);
    }

    // Outer atmospheric glow - slower, larger pulse
    if (outerGlowRef.current) {
      const outerPulse = 4.0 + Math.sin(time * 0.8) * 0.4;
      outerGlowRef.current.scale.set(outerPulse, outerPulse, 1);
    }

    // Ping beacon animation - triggers every 8-15 seconds
    const ps = pingState.current;
    ps.nextPingTime -= delta;

    // Start a new ping sequence
    if (!ps.isPinging && ps.nextPingTime <= 0) {
      ps.isPinging = true;
      ps.ping1Phase = 0;
      ps.ping2Phase = -0.3; // Second ping starts slightly after first
    }

    // Animate active pings
    if (ps.isPinging) {
      ps.ping1Phase += delta * 0.8;
      ps.ping2Phase += delta * 0.8;

      // End ping sequence when both are done
      if (ps.ping1Phase > 1.2 && ps.ping2Phase > 1.2) {
        ps.isPinging = false;
        ps.ping1Phase = -1;
        ps.ping2Phase = -1;
        ps.nextPingTime = 8 + Math.random() * 7; // Next ping in 8-15 seconds
      }
    }

    // Ping 1 - expands and fades
    if (ping1Ref.current) {
      if (ps.ping1Phase >= 0 && ps.ping1Phase <= 1.2) {
        const scale = 0.5 + ps.ping1Phase * 3.5;
        const opacity = Math.max(0, 1 - ps.ping1Phase * 0.9);
        ping1Ref.current.scale.set(scale, scale, 1);
        (ping1Ref.current.material as THREE.SpriteMaterial).opacity = opacity * 0.8;
        ping1Ref.current.visible = true;
      } else {
        ping1Ref.current.visible = false;
      }
    }

    // Ping 2 - offset timing
    if (ping2Ref.current) {
      if (ps.ping2Phase >= 0 && ps.ping2Phase <= 1.2) {
        const scale = 0.5 + ps.ping2Phase * 3.5;
        const opacity = Math.max(0, 1 - ps.ping2Phase * 0.9);
        ping2Ref.current.scale.set(scale, scale, 1);
        (ping2Ref.current.material as THREE.SpriteMaterial).opacity = opacity * 0.6;
        ping2Ref.current.visible = true;
      } else {
        ping2Ref.current.visible = false;
      }
    }

    // Orbital particles
    if (orbitalRef.current) {
      orbitalRef.current.children.forEach((child, i) => {
        const p = orbitalParticles[i];
        const angle = p.angle + time * p.speed;
        child.position.x = Math.cos(angle) * p.radius;
        child.position.z = Math.sin(angle) * p.radius;
        child.position.y = p.yOffset + Math.sin(time * 2 + i) * 0.05;
      });
    }

    // Pulsing light
    if (lightRef.current) {
      lightRef.current.intensity = 1.5 + Math.sin(time * 2) * 0.5;
    }
  });

  return (
    <group>
      {/* Outer atmospheric glow - large, soft */}
      <sprite ref={outerGlowRef} scale={[4, 4, 1]} position={[0, 0, -0.1]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={0.4}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Ping beacon ring 1 */}
      <sprite ref={ping1Ref} scale={[1, 1, 1]} position={[0, 0, 0]}>
        <spriteMaterial
          map={pingTexture}
          transparent
          opacity={0.7}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Ping beacon ring 2 - offset */}
      <sprite ref={ping2Ref} scale={[1, 1, 1]} position={[0, 0, 0]}>
        <spriteMaterial
          map={pingTexture}
          transparent
          opacity={0.5}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Inner core glow */}
      <sprite ref={glowRef} scale={[2.2, 2.2, 1]} position={[0, 0, -0.05]}>
        <spriteMaterial
          map={glowTexture}
          transparent
          opacity={1.0}
          depthWrite={false}
          blending={THREE.AdditiveBlending}
        />
      </sprite>

      {/* Orbiting particles */}
      <group ref={orbitalRef}>
        {orbitalParticles.map((_, i) => (
          <mesh key={i}>
            <sphereGeometry args={[0.025, 8, 8]} />
            <meshBasicMaterial
              color="#a0d0ff"
              transparent
              opacity={0.8}
            />
          </mesh>
        ))}
      </group>

      {/* The MK1 model */}
      <primitive
        ref={modelRef}
        object={clonedScene}
        scale={0.00291}
        rotation={[-0.3, 0, 0]}
        onPointerOver={isInteractive ? onPointerOver : undefined}
        onPointerOut={isInteractive ? onPointerOut : undefined}
        onClick={isInteractive ? onClick : undefined}
      />

      {/* Point light for visibility and reflections */}
      <pointLight
        ref={lightRef}
        color="#88bbff"
        intensity={1.5}
        distance={8}
        decay={2}
      />
    </group>
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

        // HECATE gets slower, more dramatic orbit
        const orbitSpeed = isHecate ? 0.06 : 0.15;
        const animPhase = time * orbitSpeed + orbitPhase;

        const x = Math.cos(animPhase) * orbitRadius;
        const z = Math.sin(animPhase) * orbitRadius;

        // HECATE has more dramatic vertical movement
        const yAmplitude = isHecate ? 0.8 : 0.2;
        const ySpeed = isHecate ? 0.12 : 0.3;
        const y = basePosition[1] + Math.sin(time * ySpeed + orbitPhase) * yAmplitude;

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

      // Depth-based scaling for HECATE - smaller when on far side of Crossroads
      let depthScale = 1.0;
      if (isHecate && !isSelected) {
        const z = groupRef.current.position.z;
        // z > 0 = behind Crossroads (far), z < 0 = in front (near)
        // Scale from 0.6 (far) to 1.2 (near) for dramatic perspective
        depthScale = THREE.MathUtils.mapLinear(z, orbitRadius, -orbitRadius, 0.55, 1.15);
        depthScale = THREE.MathUtils.clamp(depthScale, 0.55, 1.15);
      }

      // Smooth scale transition
      const currentScale = groupRef.current.scale.x;
      const lerpSpeed = 0.12;
      const targetWithDepth = effectiveTargetScale * depthScale;
      const newScale = THREE.MathUtils.lerp(currentScale, targetWithDepth, lerpSpeed);

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

      {/* Outer glow - not for HECATE */}
      {!isHecate && (
        <mesh>
          <sphereGeometry args={[baseSize * 1.4, 16, 16]} />
          <meshBasicMaterial
            color={cluster.color}
            transparent
            opacity={0.1}
            side={THREE.BackSide}
          />
        </mesh>
      )}

      {/* Particle nebula - not for HECATE */}
      {!isHecate && (
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
      )}

      {/* Point light - not for HECATE */}
      {!isHecate && (
        <pointLight
          color={cluster.color}
          intensity={0.8}
          distance={3}
          decay={2}
        />
      )}

      {/* Tooltip - HECATE gets special ethereal styling */}
      {showTooltip && (
        <Html
          position={[0, isHecate ? 0.5 : baseSize + 0.3, 0]}
          center
          style={{ pointerEvents: 'none' }}
        >
          {isHecate ? (
            <div style={{
              background: 'linear-gradient(135deg, rgba(10, 12, 20, 0.95) 0%, rgba(15, 20, 35, 0.95) 100%)',
              backdropFilter: 'blur(12px)',
              border: '1px solid rgba(74, 158, 255, 0.4)',
              borderRadius: '12px',
              padding: '12px 16px',
              color: '#e8e8e8',
              fontSize: '12px',
              whiteSpace: 'nowrap',
              boxShadow: '0 0 30px rgba(74, 158, 255, 0.25), inset 0 0 20px rgba(74, 158, 255, 0.05)',
              minWidth: '140px',
              textAlign: 'center',
            }}>
              <div style={{
                fontWeight: 700,
                marginBottom: '6px',
                fontSize: '14px',
                letterSpacing: '2px',
                color: '#fff',
                textShadow: '0 0 15px rgba(74, 158, 255, 0.5)',
              }}>
                H.E.C.A.T.E
              </div>
              <div style={{
                fontSize: '9px',
                color: 'rgba(74, 158, 255, 0.8)',
                marginBottom: '8px',
                letterSpacing: '0.5px',
              }}>
                Vessel: MK1 | AI: HECATE
              </div>
              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '6px',
                padding: '4px 10px',
                background: 'rgba(74, 158, 255, 0.1)',
                border: '1px solid rgba(74, 158, 255, 0.25)',
                borderRadius: '12px',
                fontSize: '10px',
                fontWeight: 600,
                color: '#4a9eff',
                textTransform: 'uppercase',
                letterSpacing: '0.5px',
              }}>
                <span style={{
                  width: '6px',
                  height: '6px',
                  borderRadius: '50%',
                  background: cluster.status === 'healthy' ? '#4a9eff' : '#ff3333',
                  boxShadow: cluster.status === 'healthy' ? '0 0 8px #4a9eff' : '0 0 8px #ff3333',
                }} />
                {cluster.status === 'healthy' ? 'Online' : cluster.status === 'unhealthy' ? 'Degraded' : 'Unknown'}
              </div>
            </div>
          ) : (
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
          )}
        </Html>
      )}
    </group>
  );
};

export default AgentCluster;
