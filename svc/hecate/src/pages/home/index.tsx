import React, { useEffect } from 'react';
import Spline from '@splinetool/react-spline';
import styles from './index.module.scss'; // Adjust the import path as necessary

const Home = () => {
  useEffect(() => {
    const updateFogEffect = (e) => {
      const fogOverlay = document.getElementById('fogOverlay');
      const x = e.clientX;
      const y = e.clientY;
      fogOverlay.style.background = `radial-gradient(circle at ${x}px ${y}px, transparent 100px, rgba(0, 0, 0, 0.75) 150px)`;
    };

    const fogOverlay = document.getElementById('fogOverlay');
    fogOverlay.style.backgroundColor = 'rgba(0, 0, 0, 0.75)';

    window.addEventListener('mousemove', updateFogEffect);

    return () => window.removeEventListener('mousemove', updateFogEffect);
  }, []);

  return (
    <>
      <div className={styles.backgroundImage}></div>
      <div id="fogOverlay" className={styles.fogOverlay}></div>
      {/* Spline object is positioned absolutely and on top of all other content */}
      <Spline className={styles.splineObject} scene="https://prod.spline.design/1Q-qMj7C6kFIofgB/scene.splinecode" />
    </>
  );
};

export default Home;

