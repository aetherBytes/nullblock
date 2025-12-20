import React, { Suspense, useState, useCallback } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Preload } from '@react-three/drei';
import VoidScene from './scene/VoidScene';
import VoidChatHUD from './chat/VoidChatHUD';
import ClusterPanel from '../hud/ClusterPanel';
import styles from './VoidExperience.module.scss';

export interface ClusterData {
  id: string;
  name: string;
  type: 'agent' | 'protocol' | 'service' | 'tool';
  status: 'healthy' | 'unhealthy' | 'unknown';
  description?: string;
  color: string;
  metrics?: {
    tasksProcessed?: number;
    uptime?: string;
    lastActive?: string;
  };
}

interface VoidExperienceProps {
  publicKey: string | null;
  theme?: 'null' | 'light' | 'dark';
  onClusterClick?: (cluster: ClusterData) => void;
  onTabSelect?: (tab: 'crossroads' | 'hecate') => void;
  loginAnimationPhase?: string;
  minimal?: boolean; // Pre-login mode: no clusters, dendrites, or chat
}

const VoidExperience: React.FC<VoidExperienceProps> = ({
  publicKey,
  theme: _theme = 'null',
  onClusterClick,
  onTabSelect,
  loginAnimationPhase,
  minimal = false,
}) => {
  const [hoveredCluster, setHoveredCluster] = useState<string | null>(null);
  const [selectedCluster, setSelectedCluster] = useState<ClusterData | null>(null);
  const [isInteracting, setIsInteracting] = useState(false);

  const handleClusterHover = useCallback((clusterId: string | null) => {
    setHoveredCluster(clusterId);
    document.body.style.cursor = clusterId ? 'pointer' : 'auto';
  }, []);

  const handleClusterClick = useCallback((cluster: ClusterData) => {
    setSelectedCluster(cluster);
    onClusterClick?.(cluster);
  }, [onClusterClick]);

  const handleInteractionStart = useCallback(() => {
    setIsInteracting(true);
  }, []);

  const handleInteractionEnd = useCallback(() => {
    setIsInteracting(false);
  }, []);

  const handleCloseClusterPanel = useCallback(() => {
    setSelectedCluster(null);
  }, []);

  const handleDiveToCrossroads = useCallback((_cluster: ClusterData) => {
    setSelectedCluster(null);
    onTabSelect?.('crossroads');
  }, [onTabSelect]);

  return (
    <div className={styles.voidContainer}>
      <Canvas
        camera={{ position: [0, 0, 5], fov: 60 }}
        gl={{ antialias: true, alpha: false }}
        dpr={[1, 2]}
      >
        <color attach="background" args={['#000000']} />
        <fog attach="fog" args={['#000000', 8, 30]} />

        <Suspense fallback={null}>
          <VoidScene
            hoveredCluster={hoveredCluster}
            onClusterHover={handleClusterHover}
            onClusterClick={handleClusterClick}
            showClusters={!minimal}
          />
        </Suspense>

        <OrbitControls
          enableDamping
          dampingFactor={0.05}
          rotateSpeed={0.5}
          zoomSpeed={0.8}
          minDistance={3}
          maxDistance={15}
          enablePan={false}
          maxPolarAngle={Math.PI * 0.85}
          minPolarAngle={Math.PI * 0.15}
          autoRotate={!isInteracting}
          autoRotateSpeed={0.3}
          onStart={handleInteractionStart}
          onEnd={handleInteractionEnd}
        />

        <Preload all />
      </Canvas>

      {/* Chat and cluster panel only shown when logged in (not minimal mode) */}
      {!minimal && (
        <>
          <VoidChatHUD
            publicKey={publicKey}
            isActive={loginAnimationPhase === 'complete'}
          />

          {/* Cluster detail panel */}
          {selectedCluster && (
            <ClusterPanel
              cluster={selectedCluster}
              onClose={handleCloseClusterPanel}
              onDiveToCrossroads={handleDiveToCrossroads}
            />
          )}
        </>
      )}
    </div>
  );
};

export default VoidExperience;
