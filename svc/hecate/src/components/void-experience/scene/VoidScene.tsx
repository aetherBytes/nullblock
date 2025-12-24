import React, { useMemo, useState, useCallback } from 'react';
import * as THREE from 'three';
import CrossroadsOrb from './CrossroadsOrb';
import ParticleField from './ParticleField';
import NeuralLines from './NeuralLines';
import AgentClusters from './AgentClusters';
import type { ClusterData } from '../VoidExperience';

// Shared constellation node type
export interface ConstellationNode {
  position: THREE.Vector3;
  connections: number[];
}

interface VoidSceneProps {
  hoveredCluster: string | null;
  selectedClusterId: string | null;
  onClusterHover: (clusterId: string | null) => void;
  onClusterClick: (cluster: ClusterData, position: THREE.Vector3) => void;
  isInteractive?: boolean; // Controls whether clusters can be clicked/hovered
}

const VoidScene: React.FC<VoidSceneProps> = ({
  hoveredCluster,
  selectedClusterId,
  onClusterHover,
  onClusterClick,
  isInteractive = true,
}) => {
  // Track which constellation nodes have active tendrils
  const [activeNodes, setActiveNodes] = useState<Set<number>>(new Set());

  const handleActiveNodesChange = useCallback((nodes: Set<number>) => {
    setActiveNodes(nodes);
  }, []);

  // Generate constellation nodes - shared between NeuralLines and CrossroadsOrb
  const constellationNodes = useMemo(() => {
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
        connections: []
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

    return nodes;
  }, []);

  return (
    <group>
      {/* Ambient lighting */}
      <ambientLight intensity={0.15} />
      <pointLight position={[0, 0, 0]} intensity={0.5} color="#e6c200" distance={10} />

      {/* Background layers */}
      <ParticleField count={1500} />
      <NeuralLines nodes={constellationNodes} activeNodes={activeNodes} />

      {/* Central Crossroads Orb - The marketplace bazaar hub */}
      <CrossroadsOrb
        position={[0, 0, 0]}
        constellationNodes={constellationNodes}
        onActiveNodesChange={handleActiveNodesChange}
      />

      {/* Floating agent clusters - always visible, interactivity controlled via prop */}
      <AgentClusters
        hoveredCluster={hoveredCluster}
        selectedClusterId={selectedClusterId}
        onClusterHover={onClusterHover}
        onClusterClick={onClusterClick}
        isInteractive={isInteractive}
      />
    </group>
  );
};

export default VoidScene;
