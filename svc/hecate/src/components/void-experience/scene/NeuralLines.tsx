import React, { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';
import type { ConstellationNode } from './VoidScene';

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
}

const NeuralLines: React.FC<NeuralLinesProps> = ({ nodes }) => {
  const groupRef = useRef<THREE.Group>(null);
  const linesRef = useRef<THREE.LineSegments>(null);

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

      {/* Service nodes - small glowing points representing tools/services/servers */}
      {nodes.map((node, i) => (
        <mesh key={i} position={[node.position.x, node.position.y, node.position.z]}>
          <sphereGeometry args={[0.08, 8, 8]} />
          <meshBasicMaterial
            color="#00d4ff"
            transparent
            opacity={0.5}
          />
        </mesh>
      ))}
    </group>
  );
};

export default NeuralLines;
