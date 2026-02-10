import { useThree, useFrame } from '@react-three/fiber';
import { forwardRef, useImperativeHandle, useRef, useEffect, useCallback } from 'react';
import * as THREE from 'three';

export interface CockpitControlsHandle {
  enabled: boolean;
  setLookAt: (target: THREE.Vector3) => void;
}

interface CockpitControlsProps {
  enabled?: boolean;
  autopilot?: boolean;
  rotateSpeed?: number;
  dampingFactor?: number;
  minPitch?: number;
  maxPitch?: number;
  thrustSpeed?: number;
}

const LOOK_AT_TARGET = new THREE.Vector3(0, 0, 0);

const CockpitControls = forwardRef<CockpitControlsHandle, CockpitControlsProps>(
  (
    {
      enabled: enabledProp = true,
      autopilot = true,
      rotateSpeed = 0.3,
      dampingFactor = 0.92,
      minPitch = -Math.PI / 3,
      maxPitch = Math.PI / 3,
      thrustSpeed = 8,
    },
    ref,
  ) => {
    const { camera, gl } = useThree();
    const enabledRef = useRef(enabledProp);
    const autopilotRef = useRef(autopilot);
    const isDragging = useRef(false);
    const prevPointer = useRef({ x: 0, y: 0 });

    const yaw = useRef(0);
    const pitch_ = useRef(0);
    const velocityYaw = useRef(0);
    const velocityPitch = useRef(0);

    const orbitTheta = useRef(0);
    const orbitPhi = useRef(Math.PI / 4);
    const orbitRadius = useRef(20);
    const orbitVelTheta = useRef(0);
    const orbitVelPhi = useRef(0);

    const thrustForwardVel = useRef(0);
    const thrustRightVel = useRef(0);
    const forwardDir = useRef(new THREE.Vector3());
    const rightDir = useRef(new THREE.Vector3());

    const initialized = useRef(false);
    const keysDown = useRef<Set<string>>(new Set());

    useEffect(() => {
      enabledRef.current = enabledProp;
    }, [enabledProp]);

    useEffect(() => {
      const wasAutopilot = autopilotRef.current;
      autopilotRef.current = autopilot;

      if (wasAutopilot !== autopilot) {
        if (autopilot) {
          const offset = camera.position.clone().sub(LOOK_AT_TARGET);
          const spherical = new THREE.Spherical().setFromVector3(offset);
          orbitTheta.current = spherical.theta;
          orbitPhi.current = spherical.phi;
          orbitRadius.current = spherical.radius;
          orbitVelTheta.current = 0;
          orbitVelPhi.current = 0;
          thrustForwardVel.current = 0;
          thrustRightVel.current = 0;
        } else {
          const dir = LOOK_AT_TARGET.clone().sub(camera.position).normalize();
          yaw.current = Math.atan2(-dir.x, -dir.z);
          pitch_.current = Math.asin(THREE.MathUtils.clamp(dir.y, -1, 1));
          pitch_.current = THREE.MathUtils.clamp(pitch_.current, minPitch, maxPitch);
          velocityYaw.current = 0;
          velocityPitch.current = 0;
        }
      }
    }, [autopilot, camera, minPitch, maxPitch]);

    useImperativeHandle(
      ref,
      () => ({
        get enabled() {
          return enabledRef.current;
        },
        set enabled(val: boolean) {
          enabledRef.current = val;
        },
        setLookAt(target: THREE.Vector3) {
          if (autopilotRef.current) {
            const offset = camera.position.clone().sub(target);
            const spherical = new THREE.Spherical().setFromVector3(offset);
            orbitTheta.current = spherical.theta;
            orbitPhi.current = spherical.phi;
            orbitRadius.current = spherical.radius;
            orbitVelTheta.current = 0;
            orbitVelPhi.current = 0;
          } else {
            const dir = target.clone().sub(camera.position).normalize();
            yaw.current = Math.atan2(-dir.x, -dir.z);
            pitch_.current = Math.asin(THREE.MathUtils.clamp(dir.y, -1, 1));
            pitch_.current = THREE.MathUtils.clamp(pitch_.current, minPitch, maxPitch);
            velocityYaw.current = 0;
            velocityPitch.current = 0;
          }
        },
      }),
      [camera, minPitch, maxPitch],
    );

    useEffect(() => {
      if (!initialized.current) {
        const offset = camera.position.clone().sub(LOOK_AT_TARGET);
        const spherical = new THREE.Spherical().setFromVector3(offset);
        orbitTheta.current = spherical.theta;
        orbitPhi.current = spherical.phi;
        orbitRadius.current = spherical.radius;

        const dir = LOOK_AT_TARGET.clone().sub(camera.position).normalize();
        yaw.current = Math.atan2(-dir.x, -dir.z);
        pitch_.current = Math.asin(THREE.MathUtils.clamp(dir.y, -1, 1));

        if (autopilotRef.current) {
          camera.lookAt(LOOK_AT_TARGET);
        } else {
          const euler = new THREE.Euler(pitch_.current, yaw.current, 0, 'YXZ');
          camera.quaternion.setFromEuler(euler);
        }

        initialized.current = true;
      }
    }, [camera]);

    const onPointerDown = useCallback((e: PointerEvent) => {
      if (!enabledRef.current) return;
      isDragging.current = true;
      prevPointer.current = { x: e.clientX, y: e.clientY };
      if (autopilotRef.current) {
        orbitVelTheta.current = 0;
        orbitVelPhi.current = 0;
      } else {
        velocityYaw.current = 0;
        velocityPitch.current = 0;
      }
    }, []);

    const onPointerMove = useCallback(
      (e: PointerEvent) => {
        if (!isDragging.current || !enabledRef.current) return;
        const dx = e.clientX - prevPointer.current.x;
        const dy = e.clientY - prevPointer.current.y;
        prevPointer.current = { x: e.clientX, y: e.clientY };

        if (autopilotRef.current) {
          const thetaDelta = -dx * rotateSpeed * 0.01;
          const phiDelta = -dy * rotateSpeed * 0.01;
          orbitTheta.current += thetaDelta;
          orbitPhi.current = THREE.MathUtils.clamp(
            orbitPhi.current + phiDelta,
            0.1,
            Math.PI - 0.1,
          );
          orbitVelTheta.current = thetaDelta;
          orbitVelPhi.current = phiDelta;
        } else {
          const yawDelta = -dx * rotateSpeed * 0.01;
          const pitchDelta = -dy * rotateSpeed * 0.01;
          yaw.current += yawDelta;
          pitch_.current = THREE.MathUtils.clamp(
            pitch_.current + pitchDelta,
            minPitch,
            maxPitch,
          );
          velocityYaw.current = yawDelta;
          velocityPitch.current = pitchDelta;
        }
      },
      [rotateSpeed, minPitch, maxPitch],
    );

    const onPointerUp = useCallback(() => {
      isDragging.current = false;
    }, []);

    const onWheel = useCallback(
      (e: WheelEvent) => {
        if (!enabledRef.current || !autopilotRef.current) return;
        e.preventDefault();
        const zoomDelta = e.deltaY * 0.01;
        orbitRadius.current = THREE.MathUtils.clamp(
          orbitRadius.current + zoomDelta,
          5,
          80,
        );
      },
      [],
    );

    useEffect(() => {
      const canvas = gl.domElement;
      canvas.addEventListener('pointerdown', onPointerDown);
      canvas.addEventListener('pointermove', onPointerMove);
      canvas.addEventListener('pointerup', onPointerUp);
      canvas.addEventListener('pointerleave', onPointerUp);
      canvas.addEventListener('wheel', onWheel, { passive: false });
      return () => {
        canvas.removeEventListener('pointerdown', onPointerDown);
        canvas.removeEventListener('pointermove', onPointerMove);
        canvas.removeEventListener('pointerup', onPointerUp);
        canvas.removeEventListener('pointerleave', onPointerUp);
        canvas.removeEventListener('wheel', onWheel);
      };
    }, [gl, onPointerDown, onPointerMove, onPointerUp, onWheel]);

    useEffect(() => {
      const isTyping = () => {
        const el = document.activeElement;
        if (!el) return false;
        const tag = el.tagName;
        return tag === 'INPUT' || tag === 'TEXTAREA' || (el as HTMLElement).isContentEditable;
      };
      const THRUST_KEYS = new Set(['w', 'W', 'a', 'A', 's', 'S', 'd', 'D']);
      const onKeyDown = (e: KeyboardEvent) => {
        if (!THRUST_KEYS.has(e.key) || isTyping()) return;
        if (!autopilotRef.current) {
          e.preventDefault();
        }
        keysDown.current.add(e.key.toLowerCase());
      };
      const onKeyUp = (e: KeyboardEvent) => {
        keysDown.current.delete(e.key.toLowerCase());
      };
      window.addEventListener('keydown', onKeyDown);
      window.addEventListener('keyup', onKeyUp);
      return () => {
        window.removeEventListener('keydown', onKeyDown);
        window.removeEventListener('keyup', onKeyUp);
      };
    }, []);

    useFrame((_, delta) => {
      if (!enabledRef.current) return;

      if (autopilotRef.current) {
        if (!isDragging.current) {
          orbitVelTheta.current *= dampingFactor;
          orbitVelPhi.current *= dampingFactor;
          orbitTheta.current += orbitVelTheta.current;
          orbitPhi.current = THREE.MathUtils.clamp(
            orbitPhi.current + orbitVelPhi.current,
            0.1,
            Math.PI - 0.1,
          );
        }

        const spherical = new THREE.Spherical(
          orbitRadius.current,
          orbitPhi.current,
          orbitTheta.current,
        );
        const offset = new THREE.Vector3().setFromSpherical(spherical);
        camera.position.copy(LOOK_AT_TARGET).add(offset);
        camera.lookAt(LOOK_AT_TARGET);
      } else {
        if (!isDragging.current) {
          velocityYaw.current *= dampingFactor;
          velocityPitch.current *= dampingFactor;
          yaw.current += velocityYaw.current;
          pitch_.current = THREE.MathUtils.clamp(
            pitch_.current + velocityPitch.current,
            minPitch,
            maxPitch,
          );
        }

        const euler = new THREE.Euler(pitch_.current, yaw.current, 0, 'YXZ');
        camera.quaternion.setFromEuler(euler);

        const keys = keysDown.current;
        let targetForward = 0;
        let targetRight = 0;

        if (keys.has('w')) targetForward += thrustSpeed;
        if (keys.has('s')) targetForward -= thrustSpeed;
        if (keys.has('a')) targetRight -= thrustSpeed;
        if (keys.has('d')) targetRight += thrustSpeed;

        if (targetForward !== 0 || targetRight !== 0) {
          thrustForwardVel.current = THREE.MathUtils.lerp(
            thrustForwardVel.current,
            targetForward,
            0.05,
          );
          thrustRightVel.current = THREE.MathUtils.lerp(
            thrustRightVel.current,
            targetRight,
            0.05,
          );
        } else {
          thrustForwardVel.current *= 0.94;
          thrustRightVel.current *= 0.94;
          if (Math.abs(thrustForwardVel.current) < 0.01) thrustForwardVel.current = 0;
          if (Math.abs(thrustRightVel.current) < 0.01) thrustRightVel.current = 0;
        }

        if (thrustForwardVel.current !== 0 || thrustRightVel.current !== 0) {
          camera.getWorldDirection(forwardDir.current);
          rightDir.current.crossVectors(forwardDir.current, camera.up).normalize();
          camera.position.addScaledVector(forwardDir.current, thrustForwardVel.current * delta);
          camera.position.addScaledVector(rightDir.current, thrustRightVel.current * delta);
        }
      }
    });

    return null;
  },
);

CockpitControls.displayName = 'CockpitControls';

export default CockpitControls;
