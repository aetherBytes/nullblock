import React from 'react';
import * as THREE from 'three';
import CrossroadsOrb from './CrossroadsOrb';
import ParticleField from './ParticleField';
import NeuralLines from './NeuralLines';
import AgentClusters from './AgentClusters';
import type { ClusterData } from '../VoidExperience';

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
  return (
    <group>
      {/* Ambient lighting */}
      <ambientLight intensity={0.15} />
      <pointLight position={[0, 0, 0]} intensity={0.5} color="#e6c200" distance={10} />

      {/* Background layers */}
      <ParticleField count={1500} />
      <NeuralLines count={30} />

      {/* Central Crossroads Orb - The marketplace bazaar hub */}
      <CrossroadsOrb position={[0, 0, 0]} />

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
