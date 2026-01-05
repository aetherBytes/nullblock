import React, { Suspense, useState, useCallback, useRef } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Preload } from '@react-three/drei';
import * as THREE from 'three';
import VoidScene from './scene/VoidScene';
import VoidHUD from './VoidHUD';
import styles from './VoidExperience.module.scss';


interface VoidExperienceProps {
  publicKey: string | null;
  theme?: 'null' | 'light' | 'dark';
  loginAnimationPhase?: string;
  isLoggedIn?: boolean;
  hecatePanelOpen?: boolean;
  onHecatePanelChange?: (open: boolean) => void;
}

// Single fixed camera position - zoomed in close
const CAMERA_POSITION = new THREE.Vector3(3.5, 2.5, 9);

const VoidExperience: React.FC<VoidExperienceProps> = ({
  publicKey,
  theme: _theme = 'null',
  loginAnimationPhase,
  isLoggedIn = false,
  hecatePanelOpen = false,
  onHecatePanelChange,
}) => {
  const [isInteracting, setIsInteracting] = useState(false);
  const [glowActive, setGlowActive] = useState(false);
  const orbitControlsRef = useRef<any>(null);

  const handleInteractionStart = useCallback(() => {
    setIsInteracting(true);
  }, []);

  const handleInteractionEnd = useCallback(() => {
    setIsInteracting(false);
  }, []);

  // Chat glow effect - triggered when agent responds
  const handleAgentResponseReceived = useCallback((_messageId: string) => {
    setGlowActive(true);
    setTimeout(() => setGlowActive(false), 800);
  }, []);

  return (
    <div className={styles.voidContainer}>
      <Canvas
        camera={{ position: [CAMERA_POSITION.x, CAMERA_POSITION.y, CAMERA_POSITION.z], fov: 60 }}
        gl={{ antialias: true, alpha: false }}
        dpr={[1, 2]}
        style={{ touchAction: 'none' }}
      >
        <color attach="background" args={['#000000']} />
        <fog attach="fog" args={['#000000', 15, 55]} />

        <Suspense fallback={null}>
          <VoidScene />
        </Suspense>

        <OrbitControls
          ref={orbitControlsRef}
          enableDamping={false}
          rotateSpeed={0.5}
          zoomSpeed={0.8}
          minDistance={4}
          maxDistance={40}
          enablePan={false}
          maxPolarAngle={Math.PI * 0.85}
          minPolarAngle={Math.PI * 0.15}
          autoRotate={!isInteracting}
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
