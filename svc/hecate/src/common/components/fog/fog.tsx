import React, { useRef, useMemo, useState, useEffect } from 'react'
import { Canvas, useFrame } from '@react-three/fiber'
import { shaderMaterial, MapControls } from '@react-three/drei'
import * as THREE from 'three'
import styles from './fog.module.scss';
import { extend } from '@react-three/fiber'

// Custom shader material for fog effect
const FogShaderMaterial = shaderMaterial(
  {
    uTime: 0,
    uColor: new THREE.Color(0x5C5C5C), // Dark gray, reminiscent of dungeon walls
    uOpacity: 0.1, // Reduced opacity for subtler effect
  },
  // Vertex Shader
  `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      vec4 modelPosition = modelMatrix * vec4(position, 1.0);
      vec4 viewPosition = viewMatrix * modelPosition;
      vec4 projectedPosition = projectionMatrix * viewPosition;
      gl_Position = projectedPosition;
    }
  `,
  // Fragment Shader
  `
    uniform float uTime;
    uniform vec3 uColor;
    uniform float uOpacity;
    varying vec2 vUv;

    void main() {
      // Smooth transition for fog density with time and position
      float fogFactor = smoothstep(0.5, 0.9, abs(sin(vUv.y * 5.0 + uTime * 0.3)));
      gl_FragColor = vec4(uColor, uOpacity * fogFactor);
    }
  `
)

extend({ FogShaderMaterial })

interface FogObjectProps {
    index: number;
}

function FogObject({ index }: FogObjectProps) {
    const meshRef = useRef<THREE.Mesh>(null!);
    const position = useMemo(() => [
        Math.random() * 1600 - 800,
        Math.random() * 20 - 10,  // Random height but close to ground
        Math.random() * 1600 - 800
    ], [index]);

    useFrame((state) => {
        if (meshRef.current.material instanceof THREE.ShaderMaterial) {
            meshRef.current.material.uniforms.uTime.value = state.clock.elapsedTime;
        }
    });

    return (
        <mesh ref={meshRef} position={position}>
            <boxGeometry args={[100, Math.random() * 20 + 5, 100]} />  // Wider fog patches for a more enveloping effect
            <fogShaderMaterial key={FogShaderMaterial.key} attach="material" />
        </mesh>
    )
}

const FogCanvas: React.FC = () => {
    const [fogObjects] = useState(() =>
        Array.from({ length: 500 }, (_, i) => <FogObject key={i} index={i} />)
    );

    return (
        <Canvas className={styles.fullScreenCanvas} camera={{ position: [400, 200, 0], fov: 75 }}>
            <ambientLight intensity={0.2} />  // Reduced light for dungeon atmosphere
            <pointLight position={[10, 10, 10]} intensity={0.5} />  // Dim point light
            {fogObjects}
            <MapControls
                enableDamping={true}
                dampingFactor={0.05}
                screenSpacePanning={false}
                minDistance={100}
                maxDistance={500}
                maxPolarAngle={Math.PI / 2}
            />
        </Canvas>
    )
}

export default FogCanvas;
