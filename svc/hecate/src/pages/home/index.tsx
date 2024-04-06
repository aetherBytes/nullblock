import React, { useEffect } from 'react';
import styles from './index.module.scss';

const Home = () => {
  useEffect(() => {
    const updateFogEffect = (e) => {
      const fogOverlay = document.getElementById('fogOverlay');
      const x = e.clientX;
      const y = e.clientY;
      // Making the fog effect darker around the mouse cursor
      fogOverlay.style.background = `radial-gradient(circle at ${x}px ${y}px, transparent 100px, rgba(0, 0, 0, 0.9) 150px)`;
    };

    window.addEventListener('mousemove', updateFogEffect);

    return () => {
      window.removeEventListener('mousemove', updateFogEffect);
    };
  }, []);

  return (
    <>
      <div className={styles.backgroundImage}>
        {/* Your existing content */}
      </div>
      <div className={styles.fogOverlay} id="fogOverlay"></div>
    </>
  );
};

export default Home;

