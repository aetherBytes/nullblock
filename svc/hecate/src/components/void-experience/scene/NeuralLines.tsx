import React, { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

interface NeuralLinesProps {
  count?: number;
  radius?: number;
}

interface NeuralNode {
  position: THREE.Vector3;
  connections: number[];
}

const NeuralLines: React.FC<NeuralLinesProps> = ({
  count = 30,
  radius = 10
}) => {
  const groupRef = useRef<THREE.Group>(null);
  const linesRef = useRef<THREE.LineSegments>(null);

  // Generate neural network nodes and connections
  const { nodes, linePositions, lineOpacities } = useMemo(() => {
    const nodes: NeuralNode[] = [];

    // Create nodes in 3D space
    for (let i = 0; i < count; i++) {
      const r = radius * 0.5 + Math.random() * radius * 0.5;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      nodes.push({
        position: new THREE.Vector3(
          r * Math.sin(phi) * Math.cos(theta),
          r * Math.sin(phi) * Math.sin(theta),
          r * Math.cos(phi)
        ),
        connections: []
      });
    }

    // Connect nearby nodes (max 3 connections per node)
    const maxConnections = 3;
    const connectionDistance = radius * 0.6;

    for (let i = 0; i < nodes.length; i++) {
      const distances: { index: number; dist: number }[] = [];

      for (let j = i + 1; j < nodes.length; j++) {
        const dist = nodes[i].position.distanceTo(nodes[j].position);
        if (dist < connectionDistance) {
          distances.push({ index: j, dist });
        }
      }

      // Sort by distance and take closest
      distances.sort((a, b) => a.dist - b.dist);
      const connectTo = distances.slice(0, maxConnections);

      for (const conn of connectTo) {
        if (nodes[i].connections.length < maxConnections &&
            nodes[conn.index].connections.length < maxConnections) {
          nodes[i].connections.push(conn.index);
          nodes[conn.index].connections.push(i);
        }
      }
    }

    // Build line geometry
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
        const baseOpacity = 0.03 + Math.random() * 0.05;
        lineOpacities.push(baseOpacity, baseOpacity);
      }
    }

    return {
      nodes,
      linePositions: new Float32Array(linePositions),
      lineOpacities: new Float32Array(lineOpacities)
    };
  }, [count, radius]);

  // Create animated opacity attribute
  const opacityAttr = useMemo(() => {
    return new THREE.BufferAttribute(lineOpacities.slice(), 1);
  }, [lineOpacities]);

  useFrame((state) => {
    if (!linesRef.current || !groupRef.current) return;

    const time = state.clock.elapsedTime;

    // Slow rotation of the entire neural network
    groupRef.current.rotation.y = time * 0.015;
    groupRef.current.rotation.x = Math.sin(time * 0.03) * 0.1;

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
          opacity={0.05}
          blending={THREE.AdditiveBlending}
          depthWrite={false}
        />
      </lineSegments>

      {/* Small glowing nodes at intersections */}
      {nodes.map((node, i) => (
        <mesh key={i} position={node.position}>
          <sphereGeometry args={[0.03, 8, 8]} />
          <meshBasicMaterial
            color="#00d4ff"
            transparent
            opacity={0.1}
          />
        </mesh>
      ))}
    </group>
  );
};

export default NeuralLines;
