import React from 'react';
import styles from '../crossroads.module.scss';

// Animation phase type (matches home/index.tsx)
type AnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

interface CrossroadsLandingProps {
  animationPhase?: AnimationPhase;
}

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({
  animationPhase = 'complete',
}) => {
  return (
    <div className={styles.landingView}>
      <p className={`${styles.bottomTagline} ${animationPhase === 'navbar' || animationPhase === 'complete' ? styles.neonFlickerIn : styles.hidden}`}>
        Discover agents, tools, and workflows. Own the tools that own the future.
      </p>
    </div>
  );
};

export default CrossroadsLanding;
