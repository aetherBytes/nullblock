import React, { Suspense, useState, useCallback, useRef, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Preload } from '@react-three/drei';
import * as THREE from 'three';
import VoidScene from './scene/VoidScene';
import CameraController from './scene/CameraController';
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
  isLoggedIn?: boolean; // Controls interactivity and camera position
}

// Camera target for traversal animation
interface CameraTarget {
  position: THREE.Vector3;
  lookAt: THREE.Vector3;
}

// Camera positions
const PRE_LOGIN_CAMERA = new THREE.Vector3(4, 3, 12); // Far back, offset to the side
const POST_LOGIN_CAMERA = new THREE.Vector3(0, 0.5, 6); // Centered on Crossroads

const VoidExperience: React.FC<VoidExperienceProps> = ({
  publicKey,
  theme: _theme = 'null',
  onClusterClick,
  onTabSelect,
  loginAnimationPhase,
  isLoggedIn = false,
}) => {
  const [hoveredCluster, setHoveredCluster] = useState<string | null>(null);
  const [selectedCluster, setSelectedCluster] = useState<ClusterData | null>(null);
  const [isInteracting, setIsInteracting] = useState(false);
  const [cameraTarget, setCameraTarget] = useState<CameraTarget | null>(null);
  const [hasArrivedAtCluster, setHasArrivedAtCluster] = useState(false);
  const [hasZoomedToHecate, setHasZoomedToHecate] = useState(false);
  const [isCanvasReady, setIsCanvasReady] = useState(false);
  const orbitControlsRef = useRef<any>(null);
  const wasLoggedIn = useRef(false);

  // Detect if this is a page refresh with existing session (publicKey exists at mount)
  const isReturningUser = useRef(!!publicKey);
  const hasTriggeredInitialZoom = useRef(false);

  // Wait for canvas to stabilize before enabling animations
  useEffect(() => {
    const timer = setTimeout(() => {
      setIsCanvasReady(true);
    }, 100);
    return () => clearTimeout(timer);
  }, []);

  // Handle zoom animation
  useEffect(() => {
    if (!isCanvasReady) return;
    if (hasTriggeredInitialZoom.current) return;

    // Returning user (page refresh with session) - zoom when animation completes
    if (isReturningUser.current && isLoggedIn && !hasZoomedToHecate) {
      hasTriggeredInitialZoom.current = true;
      wasLoggedIn.current = true;
      setCameraTarget({
        position: POST_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
      return;
    }

    // Fresh login (was not logged in, now is)
    if (!isReturningUser.current && isLoggedIn && !wasLoggedIn.current && !hasZoomedToHecate) {
      hasTriggeredInitialZoom.current = true;
      wasLoggedIn.current = true;
      setCameraTarget({
        position: POST_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
      return;
    }
  }, [isLoggedIn, hasZoomedToHecate, isCanvasReady]);

  // Reset state on logout and zoom back out
  useEffect(() => {
    if (!isLoggedIn && wasLoggedIn.current) {
      wasLoggedIn.current = false;
      setHasZoomedToHecate(false);
      hasTriggeredInitialZoom.current = false;
      isReturningUser.current = false;

      // Animate camera back to pre-login position
      setCameraTarget({
        position: PRE_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
    }
  }, [isLoggedIn]);

  const handleClusterHover = useCallback((clusterId: string | null) => {
    // Only allow hover interaction when logged in
    if (!isLoggedIn) return;
    setHoveredCluster(clusterId);
    document.body.style.cursor = clusterId ? 'pointer' : 'auto';
  }, [isLoggedIn]);

  const handleClusterClick = useCallback((cluster: ClusterData, position: THREE.Vector3) => {
    // Only allow click interaction when logged in
    if (!isLoggedIn) return;

    setSelectedCluster(cluster);
    setHasArrivedAtCluster(false);

    // Set camera target to fly to the cluster
    setCameraTarget({
      position: position.clone(),
      lookAt: position.clone(),
    });

    onClusterClick?.(cluster);
  }, [onClusterClick, isLoggedIn]);

  const handleCameraArrival = useCallback(() => {
    if (!hasZoomedToHecate && isLoggedIn) {
      // Just finished zooming to Crossroads on login
      setHasZoomedToHecate(true);
      setCameraTarget(null); // Clear target
    } else if (!isLoggedIn) {
      // Finished zooming out on logout
      setCameraTarget(null); // Clear target
    } else {
      // Arrived at a cluster
      setHasArrivedAtCluster(true);
    }
  }, [hasZoomedToHecate, isLoggedIn]);

  const handleInteractionStart = useCallback(() => {
    setIsInteracting(true);
  }, []);

  const handleInteractionEnd = useCallback(() => {
    setIsInteracting(false);
  }, []);

  const handleCloseClusterPanel = useCallback(() => {
    setSelectedCluster(null);
    setCameraTarget(null); // Return camera to home
    setHasArrivedAtCluster(false);
  }, []);

  const handleDiveToCrossroads = useCallback((_cluster: ClusterData) => {
    setSelectedCluster(null);
    onTabSelect?.('crossroads');
  }, [onTabSelect]);

  // Determine home position based on login state
  const homePosition = isLoggedIn ? POST_LOGIN_CAMERA : PRE_LOGIN_CAMERA;

  // Use stable orbit settings - only change after zoom completes to avoid jarring transitions
  const isZooming = cameraTarget !== null;
  const isFullyLoggedIn = isLoggedIn && hasZoomedToHecate;

  // Keep distance limits wide during zoom to prevent OrbitControls from clamping the camera
  // PRE_LOGIN_CAMERA is at z=12, so maxDistance must stay >= 15 until zoom completes
  const minDist = isFullyLoggedIn ? 3 : 1;
  const maxDist = isFullyLoggedIn ? 15 : 20;

  return (
    <div className={styles.voidContainer}>
      <Canvas
        camera={{ position: [PRE_LOGIN_CAMERA.x, PRE_LOGIN_CAMERA.y, PRE_LOGIN_CAMERA.z], fov: 60 }}
        gl={{ antialias: true, alpha: false }}
        dpr={[1, 2]}
        style={{ touchAction: 'none' }}
      >
        <color attach="background" args={['#000000']} />
        <fog attach="fog" args={['#000000', 10, 35]} />

        <Suspense fallback={null}>
          <VoidScene
            hoveredCluster={hoveredCluster}
            selectedClusterId={selectedCluster?.id || null}
            onClusterHover={handleClusterHover}
            onClusterClick={handleClusterClick}
            isInteractive={isLoggedIn}
          />
        </Suspense>

        {/* Camera controller for smooth traversal */}
        <CameraController
          target={cameraTarget}
          onArrival={handleCameraArrival}
          orbitControlsRef={orbitControlsRef}
          duration={2.5}
          homePosition={homePosition}
        />

        <OrbitControls
          ref={orbitControlsRef}
          enableDamping={!isFullyLoggedIn}
          dampingFactor={0.2}
          rotateSpeed={0.5}
          zoomSpeed={0.8}
          minDistance={minDist}
          maxDistance={maxDist}
          enablePan={false}
          maxPolarAngle={Math.PI * 0.85}
          minPolarAngle={Math.PI * 0.15}
          autoRotate={isCanvasReady && !isInteracting && !isZooming && !isFullyLoggedIn}
          autoRotateSpeed={0.05}
          onStart={handleInteractionStart}
          onEnd={handleInteractionEnd}
        />

        <Preload all />
      </Canvas>

      {/* Chat and cluster panel only shown when logged in */}
      {isLoggedIn && (
        <>
          <VoidChatHUD
            publicKey={publicKey}
            isActive={loginAnimationPhase === 'complete'}
          />

          {/* Cluster detail panel - shows after camera arrives */}
          {selectedCluster && hasArrivedAtCluster && (
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
