import { useThree, useFrame } from '@react-three/fiber';
import type React from 'react';
import { useRef, useEffect } from 'react';
import * as THREE from 'three';

interface CameraTarget {
  position: THREE.Vector3;
  lookAt: THREE.Vector3;
}

interface CameraControllerProps {
  target: CameraTarget | null;
  onArrival?: () => void;
  orbitControlsRef: React.RefObject<any>;
  duration?: number;
  homePosition?: THREE.Vector3;
}

const CameraController: React.FC<CameraControllerProps> = ({
  target,
  onArrival,
  orbitControlsRef,
  duration = 1.5,
  homePosition: homePositionProp,
}) => {
  const { camera } = useThree();

  // Animation state
  const isAnimating = useRef(false);
  const animationProgress = useRef(0);
  const startPosition = useRef(new THREE.Vector3());
  const startLookAt = useRef(new THREE.Vector3());
  const targetPosition = useRef(new THREE.Vector3());
  const targetLookAt = useRef(new THREE.Vector3());
  const midPoint = useRef(new THREE.Vector3());
  const hasArrived = useRef(false);

  // Store home position in ref to avoid re-triggering animation
  const homePositionRef = useRef(new THREE.Vector3(0, 0, 5));

  if (homePositionProp) {
    homePositionRef.current.copy(homePositionProp);
  }

  const homeLookAt = new THREE.Vector3(0, 0, 0);

  // Start animation when target changes
  useEffect(() => {
    if (target) {
      // Store start position
      startPosition.current.copy(camera.position);

      // Get current look-at from orbit controls or calculate
      if (orbitControlsRef.current) {
        startLookAt.current.copy(orbitControlsRef.current.target);
      } else {
        startLookAt.current.set(0, 0, 0);
      }

      // Check if this is a "zoom to center" target (position is the camera destination)
      // vs a "zoom to cluster" target (position is the cluster, we offset the camera)
      const isZoomToCenter = target.position.length() > 2; // Camera positions are far from origin

      if (isZoomToCenter) {
        // Direct camera position (e.g., login zoom to Hecate)
        targetPosition.current.copy(target.position);
      } else {
        // Cluster position - offset camera to view the cluster nicely
        const clusterPos = target.position;
        const direction = clusterPos.clone().normalize();

        targetPosition.current.copy(clusterPos).add(direction.multiplyScalar(1.5));
      }

      targetLookAt.current.copy(target.lookAt);

      // Calculate midpoint for curved path (arc above the direct line)
      midPoint.current.lerpVectors(startPosition.current, targetPosition.current, 0.5);
      midPoint.current.y += isZoomToCenter ? 2.0 : 1.5; // Higher arc for login zoom

      // Add some lateral curve based on cross product
      const travelDir = targetPosition.current.clone().sub(startPosition.current).normalize();
      const up = new THREE.Vector3(0, 1, 0);
      const lateral = travelDir.clone().cross(up).normalize();

      midPoint.current.add(lateral.multiplyScalar(isZoomToCenter ? 1.0 : 0.5));

      // Start animation
      animationProgress.current = 0;
      isAnimating.current = true;
      hasArrived.current = false;

      // Disable orbit controls during animation
      if (orbitControlsRef.current) {
        orbitControlsRef.current.enabled = false;
      }
    } else {
      // Return to home when target is null
      if (hasArrived.current || isAnimating.current) {
        // Check if we're already at home (within threshold) - don't animate if so
        const distanceToHome = camera.position.distanceTo(homePositionRef.current);

        if (distanceToHome < 0.5) {
          // Already at home, just stop
          hasArrived.current = false;
          isAnimating.current = false;

          return;
        }

        startPosition.current.copy(camera.position);

        if (orbitControlsRef.current) {
          startLookAt.current.copy(orbitControlsRef.current.target);
        }

        targetPosition.current.copy(homePositionRef.current);
        targetLookAt.current.copy(homeLookAt);

        // Curved path back home
        midPoint.current.lerpVectors(startPosition.current, targetPosition.current, 0.5);
        midPoint.current.y += 1;

        animationProgress.current = 0;
        isAnimating.current = true;
        hasArrived.current = false;
      }
    }
  }, [target, camera, orbitControlsRef]);

  // Easing function for smooth animation (quintic for extra smoothness)
  const easeInOutQuint = (t: number): number =>
    t < 0.5 ? 16 * t * t * t * t * t : 1 - Math.pow(-2 * t + 2, 5) / 2;

  // Quadratic Bezier curve interpolation
  const quadraticBezier = (
    start: THREE.Vector3,
    control: THREE.Vector3,
    end: THREE.Vector3,
    t: number,
  ): THREE.Vector3 => {
    const result = new THREE.Vector3();
    const mt = 1 - t;

    result.x = mt * mt * start.x + 2 * mt * t * control.x + t * t * end.x;
    result.y = mt * mt * start.y + 2 * mt * t * control.y + t * t * end.y;
    result.z = mt * mt * start.z + 2 * mt * t * control.z + t * t * end.z;

    return result;
  };

  useFrame((_, delta) => {
    if (!isAnimating.current) {
      return;
    }

    // Update progress
    animationProgress.current += delta / duration;

    if (animationProgress.current >= 1) {
      animationProgress.current = 1;
      isAnimating.current = false;

      // Snap to final position
      camera.position.copy(targetPosition.current);

      if (orbitControlsRef.current) {
        orbitControlsRef.current.target.copy(targetLookAt.current);
        orbitControlsRef.current.enabled = true;
        orbitControlsRef.current.update();
      }

      // Notify arrival
      if (target && !hasArrived.current) {
        hasArrived.current = true;
        onArrival?.();
      }

      return;
    }

    // Apply easing
    const easedProgress = easeInOutQuint(animationProgress.current);

    // Interpolate position along curved path
    const newPosition = quadraticBezier(
      startPosition.current,
      midPoint.current,
      targetPosition.current,
      easedProgress,
    );

    camera.position.copy(newPosition);

    // Interpolate look-at point (linear is fine for this)
    const newLookAt = new THREE.Vector3().lerpVectors(
      startLookAt.current,
      targetLookAt.current,
      easedProgress,
    );

    // Update orbit controls target
    if (orbitControlsRef.current) {
      orbitControlsRef.current.target.copy(newLookAt);
      orbitControlsRef.current.update();
    }
  });

  return null;
};

export default CameraController;
