import React from 'react';
import styles from '../crossroads.module.scss';

interface CrossroadsLandingProps {
  onConnectWallet: () => void;
}

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({ onConnectWallet }) => {
  return (
    <div className={styles.landingView}>
      <div className={styles.hero}>
        <h1>Welcome to Crossroads</h1>
        <p className={styles.subtitle}>
          Your gateway to autonomous agent workflows and Web3 automation
        </p>
        <div className={styles.stats}>
          <span>1,234</span> Services • <span>5,678</span> Active Users • <span>$2.3M</span> Volume
        </div>
      </div>

      <div className={styles.featureShowcase}>
        <div className={styles.featureCard}>
          <div className={styles.featureIcon}>🤖</div>
          <h3>Agent Marketplace</h3>
          <p>
            Discover and deploy autonomous agents for trading, social monitoring, and DeFi operations.
            Browse hundreds of pre-built agents or publish your own.
          </p>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureIcon}>🔗</div>
          <h3>MCP Integration</h3>
          <p>
            Model Context Protocol for seamless AI agent communication and coordination. Connect any
            MCP-compatible service instantly.
          </p>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureIcon}>⚡</div>
          <h3>One-Click Deploy</h3>
          <p>
            From discovery to deployment in seconds. No infrastructure setup required. Start using
            services immediately.
          </p>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureIcon}>💰</div>
          <h3>Monetize Services</h3>
          <p>
            Publish your agents and workflows to earn from your creations. Flexible pricing models
            including subscriptions and one-time payments.
          </p>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureIcon}>📊</div>
          <h3>Analytics & Insights</h3>
          <p>
            Real-time analytics, performance monitoring, and usage insights for all your deployed
            services and published offerings.
          </p>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureIcon}>🔒</div>
          <h3>Web3 Native</h3>
          <p>
            Built on Base with OnchainKit. Wallet-based authentication, on-chain payments, and
            verifiable service ownership.
          </p>
        </div>
      </div>

      <div className={styles.connectPrompt}>
        <button onClick={onConnectWallet}>
          🚀 Connect Wallet & Explore
        </button>
        <p style={{ marginTop: '1rem', color: '#64748b', fontSize: '0.875rem' }}>
          Or browse publicly available services without connecting
        </p>
      </div>

      <div className={styles.featureShowcase} style={{ marginTop: '3rem' }}>
        <div className={styles.featureCard}>
          <h3>📚 Documentation</h3>
          <p>Complete guides and API references</p>
          <button
            style={{
              marginTop: '1rem',
              padding: '0.5rem 1rem',
              background: 'rgba(99, 102, 241, 0.1)',
              border: '1px solid rgba(99, 102, 241, 0.3)',
              color: '#6366f1',
              borderRadius: '0.5rem',
              cursor: 'pointer',
            }}
            onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
          >
            View Docs
          </button>
        </div>

        <div className={styles.featureCard}>
          <h3>𝕏 Follow Updates</h3>
          <p>Stay updated with latest features</p>
          <button
            style={{
              marginTop: '1rem',
              padding: '0.5rem 1rem',
              background: 'rgba(99, 102, 241, 0.1)',
              border: '1px solid rgba(99, 102, 241, 0.3)',
              color: '#6366f1',
              borderRadius: '0.5rem',
              cursor: 'pointer',
            }}
            onClick={() => window.open('https://x.com/Nullblock_io', '_blank')}
          >
            Follow on X
          </button>
        </div>

        <div className={styles.featureCard}>
          <h3>💬 Join Discord</h3>
          <p>Community support and discussions</p>
          <button
            style={{
              marginTop: '1rem',
              padding: '0.5rem 1rem',
              background: 'rgba(99, 102, 241, 0.1)',
              border: '1px solid rgba(99, 102, 241, 0.3)',
              color: '#6366f1',
              borderRadius: '0.5rem',
              cursor: 'pointer',
            }}
            onClick={() => window.open('https://discord.gg/nullblock', '_blank')}
          >
            Join Discord
          </button>
        </div>
      </div>
    </div>
  );
};

export default CrossroadsLanding;

