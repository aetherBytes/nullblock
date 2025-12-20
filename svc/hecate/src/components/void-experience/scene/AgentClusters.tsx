import React, { useMemo } from 'react';
import AgentCluster from './AgentCluster';
import { useAgentClusters } from '../hooks/useAgentClusters';
import type { ClusterData } from '../VoidExperience';

interface AgentClustersProps {
  hoveredCluster: string | null;
  onClusterHover: (clusterId: string | null) => void;
  onClusterClick: (cluster: ClusterData) => void;
  isVisible?: boolean;
}

const AgentClusters: React.FC<AgentClustersProps> = ({
  hoveredCluster,
  onClusterHover,
  onClusterClick,
  isVisible = true,
}) => {
  const { clusters } = useAgentClusters();

  // Calculate orbital positions for each cluster
  const clusterPositions = useMemo(() => {
    const baseRadius = 2.5;
    const radiusVariation = 0.5;

    return clusters.map((cluster, index) => {
      const count = clusters.length;
      const phase = (index / count) * Math.PI * 2;

      // Vary the radius slightly for each cluster
      const radius = baseRadius + (Math.sin(phase * 3) * radiusVariation);

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
      {clusterPositions.map(({ cluster, basePosition, orbitPhase, orbitRadius }, index) => (
        <AgentCluster
          key={cluster.id}
          cluster={cluster}
          basePosition={basePosition}
          isHovered={hoveredCluster === cluster.id}
          onHover={onClusterHover}
          onClick={onClusterClick}
          orbitPhase={orbitPhase}
          orbitRadius={orbitRadius}
          isVisible={isVisible}
          fadeDelay={index * 0.15} // Stagger fade-in by 150ms per cluster
        />
      ))}
    </group>
  );
};

export default AgentClusters;
