import React, { useMemo } from 'react';
import * as THREE from 'three';
import AgentCluster from './AgentCluster';
import { useAgentClusters } from '../hooks/useAgentClusters';
import type { ClusterData } from '../VoidExperience';

interface AgentClustersProps {
  hoveredCluster: string | null;
  selectedClusterId: string | null;
  onClusterHover: (clusterId: string | null) => void;
  onClusterClick: (cluster: ClusterData, position: THREE.Vector3) => void;
  isInteractive?: boolean; // Controls whether clusters can be clicked/hovered
}

const AgentClusters: React.FC<AgentClustersProps> = ({
  hoveredCluster,
  selectedClusterId,
  onClusterHover,
  onClusterClick,
  isInteractive = true,
}) => {
  const { clusters } = useAgentClusters();

  // Calculate orbital positions for each cluster
  const clusterPositions = useMemo(() => {
    const baseRadius = 2.5;
    const radiusVariation = 0.5;

    return clusters.map((cluster, index) => {
      const count = clusters.length;
      const isHecate = cluster.name.toLowerCase().includes('hecate');
      const isSiren = cluster.name.toLowerCase().includes('siren');

      // Give HECATE and Siren distinct orbital positions
      let phase = (index / count) * Math.PI * 2;
      if (isHecate) {
        phase = 0; // Front position
      } else if (isSiren) {
        phase = Math.PI * 0.5; // 90 degrees - tangent to HECATE
      }

      // Vary the radius slightly for each cluster
      // HECATE gets extra distance from Crossroads orb
      const radius = baseRadius + (Math.sin(phase * 3) * radiusVariation) + (isHecate ? 1.0 : 0);

      // Calculate base position (will be animated)
      // Siren orbits on a tilted plane for extreme angle from HECATE
      let x, y, z;
      if (isSiren) {
        // Tilted orbit - more vertical, crossing above/below
        x = Math.cos(phase) * radius * 0.3;
        z = Math.sin(phase) * radius;
        y = Math.cos(phase) * radius * 0.8; // High vertical component
      } else {
        x = Math.cos(phase) * radius;
        z = Math.sin(phase) * radius;
        y = Math.sin(phase * 2) * 0.3;
      }

      return {
        cluster,
        basePosition: [x, y, z] as [number, number, number],
        orbitPhase: phase,
        orbitRadius: radius,
      };
    });
  }, [clusters]);

  return (
    <group>
      {clusterPositions.map(({ cluster, basePosition, orbitPhase, orbitRadius }, index) => (
        <AgentCluster
          key={cluster.id}
          cluster={cluster}
          basePosition={basePosition}
          isHovered={isInteractive && hoveredCluster === cluster.id}
          isSelected={selectedClusterId === cluster.id}
          onHover={onClusterHover}
          onClick={onClusterClick}
          orbitPhase={orbitPhase}
          orbitRadius={orbitRadius}
          isInteractive={isInteractive}
          fadeDelay={index * 0.15} // Stagger fade-in by 150ms per cluster
        />
      ))}
    </group>
  );
};

export default AgentClusters;
