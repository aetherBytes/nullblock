import React from 'react';
import HecateOrb from './HecateOrb';
import ParticleField from './ParticleField';
import NeuralLines from './NeuralLines';
import AgentClusters from './AgentClusters';
import Dendrites from './Dendrites';
import type { ClusterData } from '../VoidExperience';

interface VoidSceneProps {
  hoveredCluster: string | null;
  onClusterHover: (clusterId: string | null) => void;
  onClusterClick: (cluster: ClusterData) => void;
  showClusters?: boolean;
}

const VoidScene: React.FC<VoidSceneProps> = ({
  hoveredCluster,
  onClusterHover,
  onClusterClick,
  showClusters = true,
}) => {
  return (
    <group>
      {/* Ambient lighting */}
      <ambientLight intensity={0.15} />
      <pointLight position={[0, 0, 0]} intensity={0.5} color="#e6c200" distance={10} />

      {/* Background layers */}
      <ParticleField count={1500} />
      <NeuralLines count={30} />

      {/* Central Hecate Orb */}
      <HecateOrb position={[0, 0, 0]} />

      {/* Floating agent clusters - always rendered, visibility controlled via prop */}
      <AgentClusters
        hoveredCluster={hoveredCluster}
        onClusterHover={onClusterHover}
        onClusterClick={onClusterClick}
        isVisible={showClusters}
      />

      {/* Dynamic connection lines - only shown when clusters visible */}
      {showClusters && <Dendrites />}
    </group>
  );
};

export default VoidScene;
