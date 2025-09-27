import { Points, PointMaterial, Preload } from '@react-three/drei';
import { Canvas, useFrame } from '@react-three/fiber';
import * as random from 'maath/random';
import { useState, useRef, Suspense } from 'react';
import type { Points as ThreePoints } from 'three';
import styles from './stars.module.scss';

interface StarsProps {
  theme?: 'null' | 'matrix' | 'light';
}

const Stars = ({ theme = 'light' }: StarsProps) => {
  const ref = useRef<ThreePoints>(null);
  const [sphere] = useState<Float32Array>(() => {
    // Each point needs 3 coordinates (x,y,z), so we need positions.length to be divisible by 3
    const numPoints = 1600; // Reduced from 5000/3 to avoid potential issues
    const positions = new Float32Array(numPoints * 3);

    console.log('Generating star positions, array length:', positions.length);

    try {
      random.inSphere(positions, { radius: 1.2 });
      
      // Validate and fix any NaN values
      let nanCount = 0;
      for (let i = 0; i < positions.length; i++) {
        if (isNaN(positions[i]) || !isFinite(positions[i])) {
          nanCount++;
          // Replace NaN/Infinity with a random value between -1.2 and 1.2
          positions[i] = (Math.random() - 0.5) * 2.4;
        }
      }
      
      if (nanCount > 0) {
        console.warn(`Fixed ${nanCount} NaN values in star positions`);
      }
      
      console.log('Star positions generated successfully, sample values:', positions.slice(0, 9));
      
    } catch (error) {
      console.warn('Failed to generate star positions with random.inSphere, using fallback:', error);
      
      // Fallback: manually generate positions
      for (let i = 0; i < positions.length; i += 3) {
        const radius = Math.random() * 1.2;
        const theta = Math.random() * Math.PI * 2;
        const phi = Math.acos(2 * Math.random() - 1);
        
        positions[i] = radius * Math.sin(phi) * Math.cos(theta);     // x
        positions[i + 1] = radius * Math.sin(phi) * Math.sin(theta); // y
        positions[i + 2] = radius * Math.cos(phi);                   // z
      }
      
      console.log('Used fallback star generation, sample values:', positions.slice(0, 9));
    }

    // Final validation to ensure no NaN values exist
    const finalCheck = Array.from(positions).every(val => isFinite(val));
    console.log('Final validation - all positions are finite:', finalCheck);
    
    if (!finalCheck) {
      console.error('Still have invalid positions after validation!');
      // Force regenerate with simple method
      for (let i = 0; i < positions.length; i += 3) {
        positions[i] = (Math.random() - 0.5) * 2;     // x
        positions[i + 1] = (Math.random() - 0.5) * 2; // y  
        positions[i + 2] = (Math.random() - 0.5) * 2; // z
      }
    }

    return positions;
  });

  useFrame((_, delta) => {
    if (ref.current && isFinite(delta)) {
      const rotationX = delta / 120;
      const rotationY = delta / 180;
      
      // Ensure rotation values are valid before applying
      if (isFinite(rotationX)) {
        ref.current.rotation.x -= rotationX;
      }
      if (isFinite(rotationY)) {
        ref.current.rotation.y -= rotationY;
      }
    }
  });

  const getStarColor = () => {
    switch (theme) {
      case 'light':
        // Use black/dark colors that will be visible against white background
        const blackColors = [
          '#000000',  // Pure black
          '#1a1a1a',  // Very dark gray
          '#2a2a2a',  // Dark gray
          '#333333',  // Charcoal
          '#000000',  // Pure black
          '#1f1f1f',  // Dark gray
        ];
        return blackColors[Math.floor(Math.random() * blackColors.length)];
      case 'null':
        // Matrix-style green colors for null theme
        const matrixColors = [
          '#00ff00',  // Pure green
          '#00ff41',  // Bright green
          '#00ff9d',  // Echo green
          '#00ff88',  // Light green
          '#00ff00',  // Pure green
          '#00ff44',  // Cyan-green
        ];
        return matrixColors[Math.floor(Math.random() * matrixColors.length)];
      default:
        // Use white colors that will be visible against black background
        const defaultWhiteColors = [
          '#ffffff',  // Pure white
          '#f8f8f8',  // Off-white
          '#e8e8e8',  // Light gray
          '#f0f0f0',  // Very light gray
          '#ffffff',  // Pure white
          '#f5f5f5',  // White smoke
        ];
        return defaultWhiteColors[Math.floor(Math.random() * defaultWhiteColors.length)];
    }
  };

  // Add safety check before rendering
  if (!sphere || sphere.length === 0) {
    console.warn('No valid star positions available, skipping render');
    return null;
  }

  return (
    <group rotation={[0, 0, Math.PI / 4]}>
      <Points ref={ref} positions={sphere} stride={3} frustumCulled={false}>
        <PointMaterial
          transparent
          color={getStarColor()}
          size={theme === 'light' ? 0.004 : (theme === 'null' ? 0.003 : 0.002)}
          sizeAttenuation
          depthWrite={false}
        />
      </Points>
    </group>
  );
};

interface StarsCanvasProps {
  theme?: 'null' | 'matrix' | 'light';
}

const StarsCanvas = ({ theme = 'light' }: StarsCanvasProps) => (
  <div className={`${styles.starsCanvas} ${styles[theme]}`}>
    <Canvas camera={{ position: [0, 0, 1] }}>
      <Suspense fallback={null}>
        <Stars theme={theme} />
      </Suspense>
      <Preload all />
    </Canvas>
  </div>
);

export default StarsCanvas;
