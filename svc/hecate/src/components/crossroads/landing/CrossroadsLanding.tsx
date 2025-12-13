import React from 'react';
import styles from '../crossroads.module.scss';

interface CrossroadsLandingProps {
  onConnectWallet: () => void;
}

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({ onConnectWallet }) => {
  return (
    <div className={styles.landingView}>
      <div className={styles.missionStatement}>
        <h2 className={styles.missionText}>
          <span className={styles.missionPrimary}>
            Silos are dead.
            <br />
            The agent economy is open.
          </span>
          <br />
          <span className={styles.missionSecondary}>The first living bazaar. Web3 Native.</span>
        </h2>
      </div>

      <div className={styles.hero}>
        <div className={styles.initialViewport}>
          <div className={styles.heroContent}>
            <div className={styles.buttonContainer}>
              <button className={styles.connectButton} onClick={onConnectWallet}>
                <span>🚀 Connect Wallet & Explore</span>
              </button>
              <p className={styles.missionTagline}>
                Picks and shovels for the new age.
              </p>
            </div>
          </div>

          <div className={styles.communityLinks}>
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

