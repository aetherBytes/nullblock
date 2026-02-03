import React, { useMemo } from 'react';
import type * as THREE from 'three';
import { type ClusterData } from '../VoidExperience';
import { useAgentClusters } from '../hooks/useAgentClusters';
import AgentCluster from './AgentCluster';

interface AgentClustersProps {
  hoveredCluster: string | null;
  selectedClusterId: string | null;
  onClusterHover: (clusterId: string | null) => void;
  onClusterClick: (cluster: ClusterData, position: THREE.Vector3) => void;
  isInteractive?: boolean; // Controls whether clusters can be clicked/hovered
  onHecatePositionUpdate?: (position: THREE.Vector3) => void;
}

const AgentClusters: React.FC<AgentClustersProps> = ({
  hoveredCluster,
  selectedClusterId,
  onClusterHover,
  onClusterClick,
  isInteractive = true,
  onHecatePositionUpdate,
}) => {
  const { clusters } = useAgentClusters();

  // Calculate orbital positions for each cluster
  // Currently only HECATE orbits Crossroads
  const clusterPositions = useMemo(() => {
    const baseRadius = 5.0;

    return clusters.map((cluster, index) => {
      const count = clusters.length;
      const isHecate = cluster.name.toLowerCase().includes('hecate');

      // HECATE gets front position
      const phase = isHecate ? 0 : (index / count) * Math.PI * 2;

      // HECATE gets extra distance from Crossroads orb
      const radius = baseRadius + (isHecate ? 1.5 : 0);

      // Calculate base position (will be animated)
      const x = Math.cos(phase) * radius;
      const z = Math.sin(phase) * radius;
      const y = Math.sin(phase * 2) * 0.3;

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
      {clusterPositions.map(({ cluster, basePosition, orbitPhase, orbitRadius }, index) => {
        const isHecate = cluster.name.toLowerCase().includes('hecate');

        return (
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
            onPositionUpdate={isHecate ? onHecatePositionUpdate : undefined}
          />
        );
      })}
    </group>
  );
};

export default AgentClusters;
