import React from 'react';
import styles from '../crossroads.module.scss';

interface CrossroadsLandingProps {
  onConnectWallet: () => void;
}

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({ onConnectWallet }) => {
  return (
    <div className={styles.landingView}>
      <div className={styles.topRightText}>
        <p className={styles.heroSubtitle}>
          The AI Service Marketplace for Web3
        </p>
        <p className={styles.heroDescription}>
          Discover and deploy autonomous agents, workflows, and MCP servers.
          Built for builders. Powered by the community.
        </p>
      </div>

      <div className={styles.hero}>
        <div className={styles.heroContent}>
          <button className={styles.connectButton} onClick={onConnectWallet}>
            🚀 Connect Wallet & Explore
          </button>

          <p className={styles.browseHint}>
            Or browse publicly available services without connecting
          </p>
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
            <p>Deploy autonomous trading, social, and DeFi agents</p>
          </div>

          <div className={styles.featureItem}>
            <span className={styles.featureIcon}>🔗</span>
            <h3>MCP Protocol</h3>
            <p>Seamless AI agent communication and coordination</p>
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

