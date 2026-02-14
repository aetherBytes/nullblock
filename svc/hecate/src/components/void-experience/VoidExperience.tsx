import { OrbitControls, Preload } from '@react-three/drei';
import { Canvas, useThree, useFrame } from '@react-three/fiber';
import React, { Suspense, useState, useCallback, useRef, useEffect } from 'react';
import * as THREE from 'three';
import styles from './VoidExperience.module.scss';
import VoidScene from './scene/VoidScene';

export interface ClusterData {
  id: string;
  name: string;
  type: 'agent' | 'protocol' | 'tool' | 'service';
  status: 'healthy' | 'unhealthy' | 'unknown';
  description: string;
  color: string;
  metrics?: {
    tasksProcessed?: number;
    uptime?: number;
    lastActive?: string;
  };
}

interface VoidExperienceProps {
  publicKey: string | null;
  theme?: 'null' | 'light' | 'dark';
  loginAnimationPhase?: string;
  isLoggedIn?: boolean;
  triggerAlignment?: boolean;
  onAlignmentComplete?: () => void;
  keepAligned?: boolean;
}

const PRE_LOGIN_POSITION = new THREE.Vector3(6, 4.5, 20);
const POST_LOGIN_POSITION = new THREE.Vector3(6, 4.5, 18);
const LOOK_AT_TARGET = new THREE.Vector3(0, 0, 0);

interface CameraAnimatorProps {
  isLoggedIn: boolean;
  orbitControlsRef: React.RefObject<any>;
}

const CameraAnimator: React.FC<CameraAnimatorProps> = ({ isLoggedIn, orbitControlsRef }) => {
  const { camera } = useThree();
  const isAnimating = useRef(false);
  const animationProgress = useRef(0);
  const startPosition = useRef(new THREE.Vector3());
  const targetPosition = useRef(new THREE.Vector3());
  const prevLoggedIn = useRef(isLoggedIn);
  const duration = 2.0;

  useEffect(() => {
    if (prevLoggedIn.current !== isLoggedIn) {
      prevLoggedIn.current = isLoggedIn;

      startPosition.current.copy(camera.position);
      targetPosition.current.copy(isLoggedIn ? POST_LOGIN_POSITION : PRE_LOGIN_POSITION);

      animationProgress.current = 0;
      isAnimating.current = true;

      if (orbitControlsRef.current) {
        orbitControlsRef.current.enabled = false;
      }
    }
  }, [isLoggedIn, camera, orbitControlsRef]);

  const easeInOutQuint = (t: number): number =>
    t < 0.5 ? 16 * t * t * t * t * t : 1 - Math.pow(-2 * t + 2, 5) / 2;

  useFrame((_, delta) => {
    if (!isAnimating.current) {
      return;
    }

    animationProgress.current += delta / duration;

    if (animationProgress.current >= 1) {
      animationProgress.current = 1;
      isAnimating.current = false;

      camera.position.copy(targetPosition.current);

      if (orbitControlsRef.current) {
        orbitControlsRef.current.target.copy(LOOK_AT_TARGET);
        orbitControlsRef.current.enabled = true;
        orbitControlsRef.current.update();
      }

      return;
    }

    const easedProgress = easeInOutQuint(animationProgress.current);

    camera.position.lerpVectors(startPosition.current, targetPosition.current, easedProgress);

    if (orbitControlsRef.current) {
      orbitControlsRef.current.target.copy(LOOK_AT_TARGET);
      orbitControlsRef.current.update();
    }
  });

  return null;
};

const VoidExperience: React.FC<VoidExperienceProps> = ({
  publicKey: _publicKey,
  theme: _theme = 'null',
  loginAnimationPhase: _loginAnimationPhase,
  isLoggedIn = false,
  triggerAlignment = false,
  onAlignmentComplete,
  keepAligned = false,
}) => {
  const [isInteracting, setIsInteracting] = useState(false);
  const orbitControlsRef = useRef<any>(null);

  const handleInteractionStart = useCallback(() => {
    setIsInteracting(true);
  }, []);

  const handleInteractionEnd = useCallback(() => {
    setIsInteracting(false);
  }, []);

  return (
    <div className={styles.voidContainer}>
      <Canvas
        camera={{
          position: [PRE_LOGIN_POSITION.x, PRE_LOGIN_POSITION.y, PRE_LOGIN_POSITION.z],
          fov: 60,
        }}
        gl={{ antialias: true, alpha: false }}
        dpr={[1, 2]}
        style={{ touchAction: 'none' }}
      >
        <color attach="background" args={['#000000']} />
        <fog attach="fog" args={['#000000', 15, 55]} />

        <Suspense fallback={null}>
          <VoidScene
            triggerAlignment={triggerAlignment}
            onAlignmentComplete={onAlignmentComplete}
            keepAligned={keepAligned}
          />
        </Suspense>

        <CameraAnimator isLoggedIn={isLoggedIn} orbitControlsRef={orbitControlsRef} />

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
    </div>
  );
};

export default VoidExperience;
