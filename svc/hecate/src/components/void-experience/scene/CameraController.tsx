import { useThree, useFrame } from '@react-three/fiber';
import type React from 'react';
import { useRef, useEffect } from 'react';
import * as THREE from 'three';
import type { CockpitControlsHandle } from './CockpitControls';

interface CameraTarget {
  position: THREE.Vector3;
  lookAt: THREE.Vector3;
}

interface CameraControllerProps {
  target: CameraTarget | null;
  onArrival?: () => void;
  cockpitRef: React.RefObject<CockpitControlsHandle | null>;
  duration?: number;
  homePosition?: THREE.Vector3;
}

const CameraController: React.FC<CameraControllerProps> = ({
  target,
  onArrival,
  cockpitRef,
  duration = 1.5,
  homePosition: homePositionProp,
}) => {
  const { camera } = useThree();

  const isAnimating = useRef(false);
  const animationProgress = useRef(0);
  const startQuaternion = useRef(new THREE.Quaternion());
  const targetQuaternion = useRef(new THREE.Quaternion());
  const hasArrived = useRef(false);

  const homePositionRef = useRef(new THREE.Vector3(0, 0, 5));

  if (homePositionProp) {
    homePositionRef.current.copy(homePositionProp);
  }

  const homeLookAt = new THREE.Vector3(0, 0, 0);

  const computeLookAtQuaternion = (from: THREE.Vector3, to: THREE.Vector3): THREE.Quaternion => {
    const tempCamera = camera.clone();
    tempCamera.position.copy(from);
    tempCamera.lookAt(to);
    return tempCamera.quaternion.clone();
  };

  useEffect(() => {
    if (target) {
      startQuaternion.current.copy(camera.quaternion);
      targetQuaternion.current.copy(computeLookAtQuaternion(camera.position, target.lookAt));

      animationProgress.current = 0;
      isAnimating.current = true;
      hasArrived.current = false;

      if (cockpitRef.current) {
        cockpitRef.current.enabled = false;
      }
    } else {
      if (hasArrived.current || isAnimating.current) {
        startQuaternion.current.copy(camera.quaternion);
        targetQuaternion.current.copy(computeLookAtQuaternion(camera.position, homeLookAt));

        animationProgress.current = 0;
        isAnimating.current = true;
        hasArrived.current = false;
      }
    }
  }, [target, camera, cockpitRef]);

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

      camera.quaternion.copy(targetQuaternion.current);

      if (cockpitRef.current) {
        const finalTarget = target ? target.lookAt : homeLookAt;
        cockpitRef.current.setLookAt(finalTarget);
        cockpitRef.current.enabled = true;
      }

      if (target && !hasArrived.current) {
        hasArrived.current = true;
        onArrival?.();
      }

      return;
    }

    const easedProgress = easeInOutQuint(animationProgress.current);
    camera.quaternion.slerpQuaternions(
      startQuaternion.current,
      targetQuaternion.current,
      easedProgress,
    );
  });

  return null;
};

export default CameraController;
