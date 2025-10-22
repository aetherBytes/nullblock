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
          The AI Service Marketplace for Web3
        </h2>
      </div>

      <div className={styles.hero}>
        <div className={styles.initialViewport}>
          <div className={styles.heroContent}>
            <div className={styles.buttonContainer}>
              <button className={styles.connectButton} onClick={onConnectWallet}>
                🚀 Connect Wallet & Explore
              </button>
              <p className={styles.missionTagline}>
                Picks and shovels for the new age.
              </p>
            </div>
          </div>

          <div className={styles.browseHintContainer}>
            <p className={styles.browseHint}>
              Or browse publicly available services without connecting
            </p>
            <span className={styles.downArrow}>↓</span>
          </div>
        </div>

        <div className={styles.crossroadsSection}>
          <h1 className={styles.heroTitle}>
            <span className={styles.gradientText}>CROSSROADS</span>
          </h1>
          <p className={styles.crossroadsTagline}>Browse AI Services & Agents</p>
        </div>

        <div className={styles.featuredContent}>
          <p className={styles.featuredHint}>Featured services and trending agents coming soon...</p>
        </div>

        <div className={styles.featureGrid}>
          <div className={styles.featureItem}>
            <span className={styles.featureIcon}>🤖</span>
            <h3>Agent Marketplace</h3>
            <p>Discover and deploy autonomous agents, workflows, and services</p>
          </div>

          <div className={styles.featureItem}>
            <span className={styles.featureIcon}>🔗</span>
            <h3>Protocol-Agnostic</h3>
            <p>Support for A2A, MCP, and custom protocols. We adapt so you don't have to.</p>
          </div>

          <div className={styles.featureItem}>
            <span className={styles.featureIcon}>⚡</span>
            <h3>One-Click Deploy</h3>
            <p>From discovery to deployment in seconds</p>
          </div>

          <div className={styles.featureItem}>
            <span className={styles.featureIcon}>💰</span>
            <h3>Monetize Services</h3>
            <p>Publish agents and earn from your creations</p>
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
  );
};

export default CrossroadsLanding;

