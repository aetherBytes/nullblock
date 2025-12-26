import React, { Suspense, useState, useCallback, useRef, useEffect } from 'react';
import { Canvas, useThree } from '@react-three/fiber';
import { OrbitControls, Preload } from '@react-three/drei';
import * as THREE from 'three';
import VoidScene from './scene/VoidScene';
import CameraController from './scene/CameraController';
import ChatTendril from './scene/ChatTendril';
import VoidChatHUD from './chat/VoidChatHUD';
import ClusterPanel from '../hud/ClusterPanel';
import HecatePanel from '../hud/HecatePanel';
import styles from './VoidExperience.module.scss';

// Tendril animation state
interface TendrilAnimation {
  id: string;
  direction: 'outgoing' | 'incoming';
  startRef: React.MutableRefObject<THREE.Vector3>;
  endRef: React.MutableRefObject<THREE.Vector3>;
}

// Helper component to calculate chat world position using camera
const ChatPositionCalculator: React.FC<{
  onPositionUpdate: (pos: THREE.Vector3) => void;
}> = ({ onPositionUpdate }) => {
  const { camera } = useThree();

  useEffect(() => {
    // Chat input bar is at the very bottom of screen
    // Convert screen position to world position at a fixed distance from camera
    const updatePosition = () => {
      // NDC: -1 is bottom, +1 is top. Use -0.92 to target the input bar (not the floating messages)
      const ndc = new THREE.Vector3(0, -0.92, 0.5);
      ndc.unproject(camera);
      const dir = ndc.sub(camera.position).normalize();
      const worldPos = camera.position.clone().add(dir.multiplyScalar(5));
      onPositionUpdate(worldPos);
    };

    updatePosition();

    // Update when camera moves
    const interval = setInterval(updatePosition, 100);
    return () => clearInterval(interval);
  }, [camera, onPositionUpdate]);

  return null;
};

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
const PRE_LOGIN_CAMERA = new THREE.Vector3(8, 6, 24); // Very far back, dramatic reveal
const POST_LOGIN_CAMERA = new THREE.Vector3(4, 3, 12); // Previous pre-login position, now the zoomed-in view

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
  const [focusedPosition, setFocusedPosition] = useState<THREE.Vector3 | null>(null);
  const [tendrils, setTendrils] = useState<TendrilAnimation[]>([]);
  const [tendrilHit, setTendrilHit] = useState(false);
  const orbitControlsRef = useRef<any>(null);
  const wasLoggedIn = useRef(false);

  // Track HECATE and chat positions for tendril animations
  const hecatePositionRef = useRef<THREE.Vector3>(new THREE.Vector3(0, 0, 5));
  const chatPositionRef = useRef<THREE.Vector3>(new THREE.Vector3(0, -2, 8));

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

    // Check if this is HECATE - offset to center object in left portion of screen
    const isHecate = cluster.name.toLowerCase().includes('hecate');

    // Camera position: stay at current distance, just reframe
    // Position camera in front of the cluster at a comfortable viewing distance
    const cameraDistance = 7;

    // For HECATE: Panel takes ~1/3 of screen on right, so we need to center
    // the object in the remaining left 2/3. By looking at a point to the RIGHT
    // of the object, it shifts LEFT in the frame.
    const lookAtOffsetX = isHecate ? 3 : 0;

    const cameraPos = new THREE.Vector3(
      position.x + (isHecate ? -2 : 0), // Slight camera offset
      position.y + 1.5,
      position.z + cameraDistance
    );

    const lookAtPos = new THREE.Vector3(
      position.x + lookAtOffsetX,
      position.y,
      position.z
    );

    // Store the focused position for spotlight
    setFocusedPosition(position.clone());

    // Set camera target
    setCameraTarget({
      position: cameraPos,
      lookAt: lookAtPos,
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
    setFocusedPosition(null);
  }, []);

  const handleDiveToCrossroads = useCallback((_cluster: ClusterData) => {
    setSelectedCluster(null);
    onTabSelect?.('crossroads');
  }, [onTabSelect]);

  const handleNavigateToHecate = useCallback(() => {
    setSelectedCluster(null);
    setCameraTarget(null);
    setHasArrivedAtCluster(false);
    onTabSelect?.('hecate');
  }, [onTabSelect]);

  // Tendril animation handlers
  const handleHecatePositionUpdate = useCallback((position: THREE.Vector3) => {
    hecatePositionRef.current.copy(position);
  }, []);

  const handleChatPositionUpdate = useCallback((position: THREE.Vector3) => {
    chatPositionRef.current.copy(position);
  }, []);

  const handleUserMessageSent = useCallback((messageId: string) => {
    // Create outgoing tendril: chat → HECATE (uses refs for live tracking)
    const tendril: TendrilAnimation = {
      id: `tendril-out-${messageId}`,
      direction: 'outgoing',
      startRef: chatPositionRef,
      endRef: hecatePositionRef,
    };
    setTendrils(prev => [...prev, tendril]);
  }, []);

  const handleAgentResponseReceived = useCallback((messageId: string) => {
    // Create incoming tendril: HECATE → chat (uses refs for live tracking)
    const tendril: TendrilAnimation = {
      id: `tendril-in-${messageId}`,
      direction: 'incoming',
      startRef: hecatePositionRef,
      endRef: chatPositionRef,
    };
    setTendrils(prev => [...prev, tendril]);
  }, []);

  const handleTendrilComplete = useCallback((tendrilId: string) => {
    setTendrils(prev => prev.filter(t => t.id !== tendrilId));
  }, []);

  const handleTendrilReachTarget = useCallback((direction: 'outgoing' | 'incoming') => {
    // Trigger glow effect when incoming tendril reaches the chat box
    if (direction === 'incoming') {
      setTendrilHit(true);
      setTimeout(() => setTendrilHit(false), 1200);
    }
  }, []);

  // Determine home position based on login state
  const homePosition = isLoggedIn ? POST_LOGIN_CAMERA : PRE_LOGIN_CAMERA;

  // Use stable orbit settings - only change after zoom completes to avoid jarring transitions
  const isZooming = cameraTarget !== null;
  const isFullyLoggedIn = isLoggedIn && hasZoomedToHecate;

  // Keep distance limits wide during zoom to prevent OrbitControls from clamping the camera
  // PRE_LOGIN_CAMERA is at z=24, so maxDistance must stay >= 30 until zoom completes
  const minDist = isFullyLoggedIn ? 4 : 1;
  const maxDist = isFullyLoggedIn ? 40 : 50;

  return (
    <div className={styles.voidContainer}>
      <Canvas
        camera={{ position: [PRE_LOGIN_CAMERA.x, PRE_LOGIN_CAMERA.y, PRE_LOGIN_CAMERA.z], fov: 60 }}
        gl={{ antialias: true, alpha: false }}
        dpr={[1, 2]}
        style={{ touchAction: 'none' }}
      >
        <color attach="background" args={['#000000']} />
        <fog attach="fog" args={['#000000', 15, 55]} />

        <Suspense fallback={null}>
          <VoidScene
            hoveredCluster={hoveredCluster}
            selectedClusterId={selectedCluster?.id || null}
            onClusterHover={handleClusterHover}
            onClusterClick={handleClusterClick}
            isInteractive={isLoggedIn}
            onHecatePositionUpdate={handleHecatePositionUpdate}
          />
        </Suspense>

        {/* Track chat input world position */}
        <ChatPositionCalculator onPositionUpdate={handleChatPositionUpdate} />

        {/* Chat tendrils - animated connections between chat and HECATE */}
        {tendrils.map(tendril => (
          <ChatTendril
            key={tendril.id}
            startPos={tendril.startRef}
            endPos={tendril.endRef}
            direction={tendril.direction}
            onComplete={() => handleTendrilComplete(tendril.id)}
            onReachTarget={() => handleTendrilReachTarget(tendril.direction)}
          />
        ))}

        {/* Spotlight for focused cluster - illuminates the object when panel is open */}
        {hasArrivedAtCluster && focusedPosition && (
          <>
            {/* Key light - main illumination from front-right */}
            <spotLight
              position={[focusedPosition.x + 4, focusedPosition.y + 3, focusedPosition.z + 5]}
              target-position={[focusedPosition.x, focusedPosition.y, focusedPosition.z]}
              intensity={80}
              angle={0.5}
              penumbra={0.8}
              color="#b8d4ff"
              distance={20}
              castShadow={false}
            />
            {/* Fill light - softer from left */}
            <pointLight
              position={[focusedPosition.x - 3, focusedPosition.y + 1, focusedPosition.z + 3]}
              intensity={15}
              color="#8ab4ff"
              distance={12}
            />
            {/* Rim light - behind to create edge definition */}
            <pointLight
              position={[focusedPosition.x, focusedPosition.y + 2, focusedPosition.z - 4]}
              intensity={20}
              color="#a0c8ff"
              distance={10}
            />
          </>
        )}

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
            onUserMessageSent={handleUserMessageSent}
            onAgentResponseReceived={handleAgentResponseReceived}
            tendrilHit={tendrilHit}
          />

          {/* Cluster detail panel - shows after camera arrives */}
          {selectedCluster && hasArrivedAtCluster && (
            selectedCluster.name.toLowerCase().includes('hecate') ? (
              <HecatePanel
                onClose={handleCloseClusterPanel}
                onNavigateToHecate={handleNavigateToHecate}
                status={selectedCluster.status}
              />
            ) : (
              <ClusterPanel
                cluster={selectedCluster}
                onClose={handleCloseClusterPanel}
                onDiveToCrossroads={handleDiveToCrossroads}
              />
            )
          )}
        </>
      )}
    </div>
  );
};

export default VoidExperience;
