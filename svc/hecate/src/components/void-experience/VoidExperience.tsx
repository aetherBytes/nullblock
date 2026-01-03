import React, { Suspense, useState, useCallback, useRef, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Preload } from '@react-three/drei';
import * as THREE from 'three';
import VoidScene from './scene/VoidScene';
import CameraController from './scene/CameraController';
import VoidHUD from './VoidHUD';
import styles from './VoidExperience.module.scss';


interface VoidExperienceProps {
  publicKey: string | null;
  theme?: 'null' | 'light' | 'dark';
  loginAnimationPhase?: string;
  isLoggedIn?: boolean; // Controls interactivity and camera position
  hecatePanelOpen?: boolean;
  onHecatePanelChange?: (open: boolean) => void;
}

// Camera target for traversal animation
interface CameraTarget {
  position: THREE.Vector3;
  lookAt: THREE.Vector3;
}

// Camera positions
const PRE_LOGIN_CAMERA = new THREE.Vector3(8, 6, 24); // Very far back, dramatic reveal
const POST_LOGIN_CAMERA = new THREE.Vector3(4, 3, 12); // Previous pre-login position, now the zoomed-in view
const PANEL_OPEN_CAMERA = new THREE.Vector3(7, 5, 22); // Zoomed out when Hecate panel is open

const VoidExperience: React.FC<VoidExperienceProps> = ({
  publicKey,
  theme: _theme = 'null',
  loginAnimationPhase,
  isLoggedIn = false,
  hecatePanelOpen = false,
  onHecatePanelChange,
}) => {
  const [isInteracting, setIsInteracting] = useState(false);
  const [cameraTarget, setCameraTarget] = useState<CameraTarget | null>(null);
  const [hasZoomedIn, setHasZoomedIn] = useState(false);
  const [isCanvasReady, setIsCanvasReady] = useState(false);
  const [glowActive, setGlowActive] = useState(false);
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
    if (isReturningUser.current && isLoggedIn && !hasZoomedIn) {
      hasTriggeredInitialZoom.current = true;
      wasLoggedIn.current = true;
      setCameraTarget({
        position: POST_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
      return;
    }

    // Fresh login (was not logged in, now is)
    if (!isReturningUser.current && isLoggedIn && !wasLoggedIn.current && !hasZoomedIn) {
      hasTriggeredInitialZoom.current = true;
      wasLoggedIn.current = true;
      setCameraTarget({
        position: POST_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
      return;
    }
  }, [isLoggedIn, hasZoomedIn, isCanvasReady]);

  // Reset state on logout and zoom back out
  useEffect(() => {
    if (!isLoggedIn && wasLoggedIn.current) {
      wasLoggedIn.current = false;
      setHasZoomedIn(false);
      hasTriggeredInitialZoom.current = false;
      isReturningUser.current = false;

      // Animate camera back to pre-login position
      setCameraTarget({
        position: PRE_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
    }
  }, [isLoggedIn]);

  // Zoom out when Hecate panel opens, zoom back in when it closes
  useEffect(() => {
    // Only trigger after initial login zoom is complete and user is fully logged in
    if (!isCanvasReady || !isLoggedIn || !hasZoomedIn) return;

    if (hecatePanelOpen) {
      // Zoom out to give room for the panel
      setCameraTarget({
        position: PANEL_OPEN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
    } else {
      // Zoom back in to the normal view
      setCameraTarget({
        position: POST_LOGIN_CAMERA.clone(),
        lookAt: new THREE.Vector3(0, 0, 0),
      });
    }
  }, [hecatePanelOpen, isCanvasReady, isLoggedIn, hasZoomedIn]);

  const handleCameraArrival = useCallback(() => {
    if (!hasZoomedIn && isLoggedIn) {
      // Just finished zooming to Crossroads on login
      setHasZoomedIn(true);
      setCameraTarget(null); // Clear target
    } else if (!isLoggedIn) {
      // Finished zooming out on logout
      setCameraTarget(null); // Clear target
    }
  }, [hasZoomedIn, isLoggedIn]);

  const handleInteractionStart = useCallback(() => {
    setIsInteracting(true);
  }, []);

  const handleInteractionEnd = useCallback(() => {
    setIsInteracting(false);
  }, []);

  // Chat glow effect - triggered when agent responds
  const handleAgentResponseReceived = useCallback((_messageId: string) => {
    // Trigger glow effect on chat box when response arrives
    setGlowActive(true);
    setTimeout(() => setGlowActive(false), 800);
  }, []);

  // Determine home position based on login state
  const homePosition = isLoggedIn ? POST_LOGIN_CAMERA : PRE_LOGIN_CAMERA;

  // Use stable orbit settings - only change after zoom completes to avoid jarring transitions
  const isZooming = cameraTarget !== null;
  const isFullyLoggedIn = isLoggedIn && hasZoomedIn;

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
          <VoidScene />
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

      {/* VoidHUD only shown when logged in */}
      {isLoggedIn && (
        <VoidHUD
          publicKey={publicKey}
          isActive={true}
          loginAnimationPhase={loginAnimationPhase}
          onAgentResponseReceived={handleAgentResponseReceived}
          glowActive={glowActive}
          hecatePanelOpen={hecatePanelOpen}
          onHecatePanelChange={onHecatePanelChange}
        />
      )}
    </div>
  );
};

export default VoidExperience;
