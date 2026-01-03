import React, { useMemo, useState, useCallback, useRef } from 'react';
import * as THREE from 'three';
import CrossroadsOrb from './CrossroadsOrb';
import ParticleField from './ParticleField';
import NeuralLines from './NeuralLines';

// Shared constellation node type
export interface ConstellationNode {
  position: THREE.Vector3;
  connections: number[];
  clusterId: number; // Which connected group this node belongs to
}

// Orbital parameters for each connected cluster
export interface ClusterOrbit {
  speed: number;       // Radians per second
  tiltX: number;       // Orbital plane tilt on X axis
  tiltZ: number;       // Orbital plane tilt on Z axis
  phase: number;       // Initial phase offset
}

const VoidScene: React.FC = () => {
  // Track which constellation nodes have active tendrils
  const [activeNodes, setActiveNodes] = useState<Set<number>>(new Set());

  const handleActiveNodesChange = useCallback((nodes: Set<number>) => {
    setActiveNodes(nodes);
  }, []);

  // Generate constellation nodes and identify connected clusters
  const { constellationNodes, clusterOrbits } = useMemo(() => {
    const count = 25;
    const radius = 16;
    const nodes: ConstellationNode[] = [];

    // Create nodes in 3D space - pushed further out with more spacing
    for (let i = 0; i < count; i++) {
      const r = radius * 0.6 + Math.random() * radius * 0.4;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);

      nodes.push({
        position: new THREE.Vector3(
          r * Math.sin(phi) * Math.cos(theta),
          r * Math.sin(phi) * Math.sin(theta),
          r * Math.cos(phi)
        ),
        connections: [],
        clusterId: -1 // Will be assigned after connections are made
      });
    }

    // Connect nearby nodes (max 3 connections per node)
    const maxConnections = 3;
    const connectionDistance = radius * 0.5;

    for (let i = 0; i < nodes.length; i++) {
      const distances: { index: number; dist: number }[] = [];

      for (let j = i + 1; j < nodes.length; j++) {
        const dist = nodes[i].position.distanceTo(nodes[j].position);
        if (dist < connectionDistance) {
          distances.push({ index: j, dist });
        }
      }

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

    // Identify connected clusters using BFS
    let currentCluster = 0;
    for (let i = 0; i < nodes.length; i++) {
      if (nodes[i].clusterId === -1) {
        // BFS to find all connected nodes
        const queue = [i];
        nodes[i].clusterId = currentCluster;

        while (queue.length > 0) {
          const nodeIdx = queue.shift()!;
          for (const connIdx of nodes[nodeIdx].connections) {
            if (nodes[connIdx].clusterId === -1) {
              nodes[connIdx].clusterId = currentCluster;
              queue.push(connIdx);
            }
          }
        }
        currentCluster++;
      }
    }

    // Generate orbital parameters for each cluster
    const orbits: ClusterOrbit[] = [];
    for (let c = 0; c < currentCluster; c++) {
      orbits.push({
        speed: 0.02 + Math.random() * 0.04,  // 0.02-0.06 rad/sec (very slow)
        tiltX: (Math.random() - 0.5) * 0.3,  // Small tilt variation
        tiltZ: (Math.random() - 0.5) * 0.3,
        phase: Math.random() * Math.PI * 2   // Random starting phase
      });
    }

    return { constellationNodes: nodes, clusterOrbits: orbits };
  }, []);

  // Ref to hold animated node positions (updated by NeuralLines, read by CrossroadsOrb)
  const animatedPositionsRef = useRef<THREE.Vector3[]>(
    constellationNodes.map(n => n.position.clone())
  );

  return (
    <group>
      {/* Ambient lighting */}
      <ambientLight intensity={0.15} />
      <pointLight position={[0, 0, 0]} intensity={0.5} color="#e6c200" distance={10} />

      {/* Background layers */}
      <ParticleField count={1500} />
      <NeuralLines
        nodes={constellationNodes}
        activeNodes={activeNodes}
        clusterOrbits={clusterOrbits}
        animatedPositionsRef={animatedPositionsRef}
      />

      {/* Central Crossroads Orb - The marketplace bazaar hub */}
      <CrossroadsOrb
        position={[0, 0, 0]}
        constellationNodes={constellationNodes}
        animatedPositionsRef={animatedPositionsRef}
        onActiveNodesChange={handleActiveNodesChange}
      />
    </group>
  );
};

export default VoidScene;
