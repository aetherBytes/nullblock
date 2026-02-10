import React from 'react';
import styles from '../crossroads.module.scss';

// Animation phase type (matches home/index.tsx)
type AnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

interface CrossroadsLandingProps {
  onConnectWallet: () => void;
  onNavigateCrossroads: () => void;
  animationPhase?: AnimationPhase;
  pendingTransition?: boolean;
}

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({
  animationPhase: _animationPhase = 'complete',
}) => {
  return (
    <div className={styles.landingView}>
      <div className={styles.hero}>
        <div className={styles.initialViewport} />
      </div>
    </div>
  );
};

export default CrossroadsLanding;
