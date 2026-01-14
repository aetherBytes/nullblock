import { useFrame } from '@react-three/fiber';
import React, { useRef, useState, useEffect, useMemo } from 'react';
import * as THREE from 'three';

interface DendriteConnection {
  id: string;
  fromPosition: THREE.Vector3;
  toPosition: THREE.Vector3;
  color: string;
  active: boolean;
  progress: number;
  opacity: number;
  createdAt: number;
}

interface DendritesProps {
  maxConnections?: number;
}

const Dendrites: React.FC<DendritesProps> = ({ maxConnections = 8 }) => {
  const groupRef = useRef<THREE.Group>(null);
  const [connections, setConnections] = useState<DendriteConnection[]>([]);

  // Create demo connections that periodically activate
  useEffect(() => {
    const createRandomConnection = () => {
      const angle1 = Math.random() * Math.PI * 2;
      const angle2 = Math.random() * Math.PI * 2;
      const radius = 2.5;

      const from = new THREE.Vector3(
        Math.cos(angle1) * radius,
        Math.sin(angle1 * 2) * 0.3,
        Math.sin(angle1) * radius,
      );

      const to = new THREE.Vector3(
        Math.cos(angle2) * radius,
        Math.sin(angle2 * 2) * 0.3,
        Math.sin(angle2) * radius,
      );

      const colors = ['#00ff9d', '#00d4ff', '#b967ff', '#e6c200'];
      const color = colors[Math.floor(Math.random() * colors.length)];

      return {
        id: `dendrite-${Date.now()}-${Math.random()}`,
        fromPosition: from,
        toPosition: to,
        color,
        active: true,
        progress: 0,
        opacity: 0,
        createdAt: Date.now(),
      };
    };

    // Periodically create new connections
    const interval = setInterval(() => {
      setConnections((prev) => {
        // Remove old connections
        const now = Date.now();
        const filtered = prev.filter((c) => now - c.createdAt < 5000);

        // Add new connection if under limit
        if (filtered.length < maxConnections && Math.random() > 0.5) {
          return [...filtered, createRandomConnection()];
        }

        return filtered;
      });
    }, 2000);

    // Create a few initial connections
    const initial: DendriteConnection[] = [];
    for (let i = 0; i < 3; i++) {
      initial.push(createRandomConnection());
    }
    setConnections(initial);

    return () => clearInterval(interval);
  }, [maxConnections]);

  // Animate connections
  useFrame((_, delta) => {
    setConnections((prev) =>
      prev.map((conn) => {
        if (conn.active) {
          // Grow the connection
          const newProgress = Math.min(conn.progress + delta * 2, 1);
          const newOpacity = Math.min(conn.opacity + delta * 3, 0.8);

          // Start fading after fully grown
          if (newProgress >= 1) {
            const age = (Date.now() - conn.createdAt) / 1000;

            if (age > 2) {
              return {
                ...conn,
                progress: newProgress,
                opacity: Math.max(0, conn.opacity - delta * 0.5),
                active: conn.opacity > 0.1,
              };
            }
          }

          return {
            ...conn,
            progress: newProgress,
            opacity: newOpacity,
          };
        }

        return conn;
      }),
    );
  });

  return (
    <group ref={groupRef}>
      {connections.map((conn) => (
        <DendriteLine
          key={conn.id}
          from={conn.fromPosition}
          to={conn.toPosition}
          color={conn.color}
          progress={conn.progress}
          opacity={conn.opacity}
        />
      ))}
    </group>
  );
};

interface DendriteLineProps {
  from: THREE.Vector3;
  to: THREE.Vector3;
  color: string;
  progress: number;
  opacity: number;
}

const DendriteLine: React.FC<DendriteLineProps> = ({ from, to, color, progress, opacity }) => {
  // Create curved path between points
  const points = useMemo(() => {
    const midPoint = new THREE.Vector3().lerpVectors(from, to, 0.5);

    // Add some curve by offsetting the midpoint
    midPoint.y += 0.5;

    const curve = new THREE.QuadraticBezierCurve3(from, midPoint, to);

    return curve.getPoints(32);
  }, [from, to]);

  // Animated points based on progress
  const visiblePoints = useMemo(() => {
    const numVisible = Math.floor(points.length * progress);

    return points.slice(0, numVisible);
  }, [points, progress]);

  const geometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();

    if (visiblePoints.length > 1) {
      const positions = new Float32Array(visiblePoints.length * 3);

      visiblePoints.forEach((point, i) => {
        positions[i * 3] = point.x;
        positions[i * 3 + 1] = point.y;
        positions[i * 3 + 2] = point.z;
      });
      geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    }

    return geo;
  }, [visiblePoints]);

  if (visiblePoints.length < 2) {
    return null;
  }

  return (
    <group>
      {/* Main line */}
      <line>
        <primitive object={geometry} attach="geometry" />
        <lineBasicMaterial
          color={color}
          transparent
          opacity={opacity}
          linewidth={2}
          blending={THREE.AdditiveBlending}
        />
      </line>

      {/* Glow effect line */}
      <line>
        <primitive object={geometry.clone()} attach="geometry" />
        <lineBasicMaterial
          color={color}
          transparent
          opacity={opacity * 0.3}
          linewidth={4}
          blending={THREE.AdditiveBlending}
        />
      </line>

      {/* Pulse particle at the leading edge */}
      {progress > 0 && progress < 1 && (
        <mesh position={visiblePoints[visiblePoints.length - 1]}>
          <sphereGeometry args={[0.03, 8, 8]} />
          <meshBasicMaterial color={color} transparent opacity={opacity} />
        </mesh>
      )}

      {/* Burst effect at endpoints when connection completes */}
      {progress >= 1 && opacity > 0.5 && (
        <mesh position={to}>
          <sphereGeometry args={[0.05, 16, 16]} />
          <meshBasicMaterial color={color} transparent opacity={opacity * 0.5} />
        </mesh>
      )}
    </group>
  );
};

export default Dendrites;
