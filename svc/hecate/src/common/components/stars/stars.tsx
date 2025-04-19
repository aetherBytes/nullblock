import { useState, useRef, Suspense } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { Points, PointMaterial, Preload } from "@react-three/drei";
import * as random from "maath/random";
import styles from './stars.module.scss';
import { Group, Points as ThreePoints } from 'three';

interface StarsProps {
  theme?: 'null' | 'matrix' | 'cyber' | 'light';
}

const Stars = ({ theme = 'null' }: StarsProps) => {
  const ref = useRef<ThreePoints>(null);
  const [sphere] = useState<Float32Array>(() => {
    const positions = new Float32Array(5000);
    random.inSphere(positions, { radius: 1.2 });
    return positions;
  });

  useFrame((state, delta) => {
    if (ref.current) {
      ref.current.rotation.x -= delta / 120;
      ref.current.rotation.y -= delta / 180;
    }
  });

  const getStarColor = () => {
    switch (theme) {
      case 'light':
        return '#000000';
      case 'matrix':
        return '#00ff00';
      case 'cyber':
        return '#00ffff';
      default:
        return '#f272c8';
    }
  };

  return (
    <group rotation={[0, 0, Math.PI / 4]}>
      <Points ref={ref} positions={sphere} stride={3} frustumCulled>
        <PointMaterial
          transparent
          color={getStarColor()}
          size={theme === 'light' ? 0.003 : 0.002}
          sizeAttenuation={true}
          depthWrite={false}
        />
      </Points>
    </group>
  );
};

interface StarsCanvasProps {
  theme?: 'null' | 'matrix' | 'cyber' | 'light';
}

const StarsCanvas = ({ theme = 'null' }: StarsCanvasProps) => {
  return (
    <div className={`${styles.starsCanvas} ${styles[theme]}`}>
      <Canvas camera={{ position: [0, 0, 1] }}>
        <Suspense fallback={null}>
          <Stars theme={theme} />
        </Suspense>
        <Preload all />
      </Canvas>
    </div>
  );
};

export default StarsCanvas;

