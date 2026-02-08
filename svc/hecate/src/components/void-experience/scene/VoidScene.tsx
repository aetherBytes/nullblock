import React, { useMemo, useState, useCallback, useRef } from 'react';
import * as THREE from 'three';
import CrossroadsOrb from './CrossroadsOrb';
import NeuralLines from './NeuralLines';
import ParticleField from './ParticleField';

// Shared constellation node type
export interface ConstellationNode {
  position: THREE.Vector3;
  connections: number[];
  clusterId: number; // Which connected group this node belongs to
}

// Orbital parameters for each connected cluster
export interface ClusterOrbit {
  speed: number; // Radians per second
  tiltX: number; // Orbital plane tilt on X axis
  tiltZ: number; // Orbital plane tilt on Z axis
  phase: number; // Initial phase offset
}

interface VoidSceneProps {
  triggerAlignment?: boolean;
  onAlignmentComplete?: () => void;
  keepAligned?: boolean;
}

const VoidScene: React.FC<VoidSceneProps> = ({
  triggerAlignment = false,
  onAlignmentComplete,
  keepAligned = false,
}) => {
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
          r * Math.cos(phi),
        ),
        connections: [],
        clusterId: -1, // Will be assigned after connections are made
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
        if (
          nodes[i].connections.length < maxConnections &&
          nodes[conn.index].connections.length < maxConnections
        ) {
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
        speed: 0.02 + Math.random() * 0.04, // 0.02-0.06 rad/sec (very slow)
        tiltX: (Math.random() - 0.5) * 0.3, // Small tilt variation
        tiltZ: (Math.random() - 0.5) * 0.3,
        phase: Math.random() * Math.PI * 2, // Random starting phase
      });
    }

    return { constellationNodes: nodes, clusterOrbits: orbits };
  }, []);

  // Generate outer constellations - smaller networks at the edge of the void
  const { outerNodes, outerClusterOrbits } = useMemo(() => {
    const nodes: ConstellationNode[] = [];

    // Create 10-14 small clusters scattered at varied distances
    const numClusters = 10 + Math.floor(Math.random() * 5);
    const minRadius = 20; // Minimum distance from center
    const maxRadius = 42;

    for (let cluster = 0; cluster < numClusters; cluster++) {
      // Each outer cluster has 3-6 nodes
      const clusterSize = 3 + Math.floor(Math.random() * 4);

      // Vary the distance - some closer, some further
      // Use a distribution that spreads them out more evenly
      const distanceBand = Math.random();
      let clusterRadius;

      if (distanceBand < 0.35) {
        // Closer band (20-28)
        clusterRadius = minRadius + Math.random() * 8;
      } else if (distanceBand < 0.7) {
        // Middle band (26-35)
        clusterRadius = 26 + Math.random() * 9;
      } else {
        // Outer band (33-42)
        clusterRadius = 33 + Math.random() * (maxRadius - 33);
      }

      const clusterTheta = Math.random() * Math.PI * 2;
      const clusterPhi = Math.acos(2 * Math.random() - 1);

      const clusterCenter = new THREE.Vector3(
        clusterRadius * Math.sin(clusterPhi) * Math.cos(clusterTheta),
        clusterRadius * Math.sin(clusterPhi) * Math.sin(clusterTheta),
        clusterRadius * Math.cos(clusterPhi),
      );

      const clusterStartIdx = nodes.length;

      // Vary cluster shape: chain, arc, scattered, hub-spoke
      const shapeType = Math.random();

      if (shapeType < 0.25) {
        // CHAIN - nodes in a line/curve
        const direction = new THREE.Vector3(
          Math.random() - 0.5,
          Math.random() - 0.5,
          Math.random() - 0.5,
        ).normalize();
        const perpendicular = new THREE.Vector3(
          Math.random() - 0.5,
          Math.random() - 0.5,
          Math.random() - 0.5,
        )
          .cross(direction)
          .normalize();

        for (let i = 0; i < clusterSize; i++) {
          const t = i / (clusterSize - 1) - 0.5; // -0.5 to 0.5
          const curve = Math.sin(t * Math.PI) * 1.5; // Slight curve
          const offset = direction
            .clone()
            .multiplyScalar(t * 8)
            .add(perpendicular.clone().multiplyScalar(curve))
            .add(
              new THREE.Vector3(
                (Math.random() - 0.5) * 1,
                (Math.random() - 0.5) * 1,
                (Math.random() - 0.5) * 1,
              ),
            );

          nodes.push({
            position: clusterCenter.clone().add(offset),
            connections: [],
            clusterId: cluster,
          });
        }
      } else if (shapeType < 0.5) {
        // ARC - nodes in a curved arc
        const arcAngle = Math.PI * (0.4 + Math.random() * 0.5); // 70-160 degree arc
        const arcRadius = 2 + Math.random() * 3;
        const startAngle = Math.random() * Math.PI * 2;
        const tiltAxis = new THREE.Vector3(
          Math.random() - 0.5,
          Math.random() - 0.5,
          Math.random() - 0.5,
        ).normalize();

        for (let i = 0; i < clusterSize; i++) {
          const angle = startAngle + (i / (clusterSize - 1)) * arcAngle;
          const offset = new THREE.Vector3(
            Math.cos(angle) * arcRadius,
            Math.sin(angle) * arcRadius * 0.5,
            Math.sin(angle) * arcRadius,
          )
            .applyAxisAngle(tiltAxis, Math.random() * Math.PI)
            .add(
              new THREE.Vector3(
                (Math.random() - 0.5) * 0.8,
                (Math.random() - 0.5) * 0.8,
                (Math.random() - 0.5) * 0.8,
              ),
            );

          nodes.push({
            position: clusterCenter.clone().add(offset),
            connections: [],
            clusterId: cluster,
          });
        }
      } else if (shapeType < 0.75) {
        // HUB-SPOKE - central node with others radiating out
        // Central hub node
        nodes.push({
          position: clusterCenter.clone(),
          connections: [],
          clusterId: cluster,
        });

        // Spoke nodes around it
        for (let i = 1; i < clusterSize; i++) {
          const angle = (i / (clusterSize - 1)) * Math.PI * 2;
          const spokeLen = 2.5 + Math.random() * 2.5;
          const offset = new THREE.Vector3(
            Math.cos(angle) * spokeLen,
            (Math.random() - 0.5) * 2,
            Math.sin(angle) * spokeLen,
          );

          nodes.push({
            position: clusterCenter.clone().add(offset),
            connections: [],
            clusterId: cluster,
          });
        }
      } else {
        // SCATTERED - random positions (original behavior)
        for (let i = 0; i < clusterSize; i++) {
          const offset = new THREE.Vector3(
            (Math.random() - 0.5) * 7,
            (Math.random() - 0.5) * 5,
            (Math.random() - 0.5) * 7,
          );

          nodes.push({
            position: clusterCenter.clone().add(offset),
            connections: [],
            clusterId: cluster,
          });
        }
      }

      // Connect nodes within this cluster
      const clusterNodes = nodes.slice(clusterStartIdx);
      const maxConn = shapeType < 0.5 ? 2 : 3; // Chains/arcs get 2, others get 3

      for (let i = 0; i < clusterNodes.length; i++) {
        const nodeIdx = clusterStartIdx + i;
        const distances: { index: number; dist: number }[] = [];

        for (let j = i + 1; j < clusterNodes.length; j++) {
          const otherIdx = clusterStartIdx + j;
          const dist = nodes[nodeIdx].position.distanceTo(nodes[otherIdx].position);

          distances.push({ index: otherIdx, dist });
        }

        distances.sort((a, b) => a.dist - b.dist);
        const connectTo = distances.slice(0, maxConn);

        for (const conn of connectTo) {
          if (
            nodes[nodeIdx].connections.length < maxConn &&
            nodes[conn.index].connections.length < maxConn
          ) {
            nodes[nodeIdx].connections.push(conn.index);
            nodes[conn.index].connections.push(nodeIdx);
          }
        }
      }
    }

    // Generate orbital parameters for outer clusters (slower, gentler movement)
    const orbits: ClusterOrbit[] = [];
    for (let c = 0; c < numClusters; c++) {
      orbits.push({
        speed: 0.008 + Math.random() * 0.015, // Very slow
        tiltX: (Math.random() - 0.5) * 0.2,
        tiltZ: (Math.random() - 0.5) * 0.2,
        phase: Math.random() * Math.PI * 2,
      });
    }

    return { outerNodes: nodes, outerClusterOrbits: orbits };
  }, []);

  // Ref to hold animated node positions (updated by NeuralLines, read by CrossroadsOrb)
  const animatedPositionsRef = useRef<THREE.Vector3[]>(
    constellationNodes.map((n) => n.position.clone()),
  );

  // Ref for outer constellation positions
  const outerAnimatedPositionsRef = useRef<THREE.Vector3[]>(
    outerNodes.map((n) => n.position.clone()),
  );

  return (
    <group>
      {/* Ambient lighting */}
      <ambientLight intensity={0.15} />
      <pointLight position={[0, 0, 0]} intensity={0.5} color="#e6c200" distance={10} />

      {/* Background layers */}
      <ParticleField count={800} />

      {/* Inner constellations */}
      <NeuralLines
        nodes={constellationNodes}
        activeNodes={activeNodes}
        clusterOrbits={clusterOrbits}
        animatedPositionsRef={animatedPositionsRef}
      />

      {/* Outer constellations - visible when zoomed out */}
      <NeuralLines
        nodes={outerNodes}
        clusterOrbits={outerClusterOrbits}
        animatedPositionsRef={outerAnimatedPositionsRef}
      />

      {/* Central Crossroads Orb - The marketplace bazaar hub */}
      <CrossroadsOrb
        position={[0, 0, 0]}
        constellationNodes={constellationNodes}
        animatedPositionsRef={animatedPositionsRef}
        outerNodes={outerNodes}
        outerAnimatedPositionsRef={outerAnimatedPositionsRef}
        onActiveNodesChange={handleActiveNodesChange}
        triggerAlignment={triggerAlignment}
        onAlignmentComplete={onAlignmentComplete}
        keepAligned={keepAligned}
      />
    </group>
  );
};

export default VoidScene;
