import React from 'react';
import styles from '../crossroads.module.scss';

// Animation phase type (matches home/index.tsx)
type AnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

interface CrossroadsLandingProps {
  onConnectWallet: () => void;
  animationPhase?: AnimationPhase;
}

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({
  onConnectWallet,
  animationPhase = 'complete',
}) => {
  // Determine CSS classes based on animation phase
  const getMissionPrimaryClass = () => {
    // "Silos are dead. The agent economy is open." fades in with background
    if (
      animationPhase === 'background' ||
      animationPhase === 'navbar' ||
      animationPhase === 'complete'
    ) {
      return `${styles.missionPrimary} ${styles.fadeIn}`;
    }

    return `${styles.missionPrimary} ${styles.hidden}`;
  };

  const getMissionSecondaryClass = () => {
    // "The first living bazaar. Web3 Native." flickers in with navbar
    if (animationPhase === 'navbar' || animationPhase === 'complete') {
      return `${styles.missionSecondary} ${styles.neonFlickerIn}`;
    }

    return `${styles.missionSecondary} ${styles.hidden}`;
  };

  const getCtaClass = () => {
    // Connect button and tagline flicker in with navbar
    if (animationPhase === 'navbar' || animationPhase === 'complete') {
      return styles.neonFlickerIn;
    }

    return styles.hidden;
  };

  return (
    <div className={styles.landingView}>
      <div className={styles.missionStatement}>
        <h2 className={styles.missionText}>
          <span className={getMissionPrimaryClass()}>
            Silos are dead.
            <br />
            The agent economy is open.
          </span>
          <br />
          <span className={getMissionSecondaryClass()}>The first living bazaar. Web3 Native.</span>
        </h2>
      </div>

      <div className={styles.hero}>
        <div className={styles.initialViewport}>
          <div className={styles.heroContent}>
            <div className={`${styles.buttonContainer} ${getCtaClass()}`}>
              <button className={styles.connectButton} onClick={onConnectWallet}>
                <span>🚀 Connect Wallet & Explore</span>
              </button>
              <p className={styles.missionTagline}>Picks and shovels for the new age.</p>
            </div>
          </div>

          <div className={`${styles.communityLinks} ${getCtaClass()}`}>
            <a
              href="https://aetherbytes.github.io/nullblock-sdk/"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.communityLink}
            >
              📚 Documentation
            </a>
            <a
              href="https://x.com/Nullblock_io"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.communityLink}
            >
              𝕏 Follow Updates
            </a>
            <a
              href="https://discord.gg/nullblock"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.communityLink}
            >
              💬 Join Discord
            </a>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CrossroadsLanding;
