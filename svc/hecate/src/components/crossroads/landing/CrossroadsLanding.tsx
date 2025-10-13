import React, { useState } from 'react';
import styles from '../crossroads.module.scss';

interface CrossroadsLandingProps {
  onConnectWallet: () => void;
}

interface FeatureData {
  icon: string;
  title: string;
  description: string;
}

const features: FeatureData[] = [
  {
    icon: '🤖',
    title: 'Agent Marketplace',
    description: 'Discover and deploy autonomous agents for trading, social monitoring, and DeFi operations. Browse hundreds of pre-built agents or publish your own.'
  },
  {
    icon: '🔗',
    title: 'MCP Integration',
    description: 'Model Context Protocol for seamless AI agent communication and coordination. Connect any MCP-compatible service instantly.'
  },
  {
    icon: '⚡',
    title: 'One-Click Deploy',
    description: 'From discovery to deployment in seconds. No infrastructure setup required. Start using services immediately.'
  },
  {
    icon: '💰',
    title: 'Monetize Services',
    description: 'Publish your agents and workflows to earn from your creations. Flexible pricing models including subscriptions and one-time payments.'
  },
  {
    icon: '📊',
    title: 'Analytics & Insights',
    description: 'Real-time analytics, performance monitoring, and usage insights for all your deployed services and published offerings.'
  },
  {
    icon: '🔒',
    title: 'Web3 Native',
    description: 'Built on Base with OnchainKit. Wallet-based authentication, on-chain payments, and verifiable service ownership.'
  }
];

const CrossroadsLanding: React.FC<CrossroadsLandingProps> = ({ onConnectWallet }) => {
  const [hoveredCard, setHoveredCard] = useState<number | null>(null);

  return (
    <div className={styles.landingView}>
      <div className={styles.hero}>
        {/* Hero section removed - welcome is now in navbar */}
      </div>

      <div className={styles.connectPrompt} style={{ marginTop: '1rem' }}>
        <button onClick={onConnectWallet}>
          🚀 Connect Wallet & Explore
        </button>
        <p style={{ marginTop: '1rem', color: '#64748b', fontSize: '0.875rem' }}>
          Or browse publicly available services without connecting
        </p>
      </div>

      <div className={styles.featureShowcaseBottom}>
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

      <div className={styles.featureShowcaseRight}>
        {features.map((feature, index) => (
          <div
            key={index}
            className={styles.featureCardCompact}
            onMouseEnter={() => setHoveredCard(index)}
            onMouseLeave={() => setHoveredCard(null)}
            onClick={() => setHoveredCard(hoveredCard === index ? null : index)}
          >
            <div className={styles.cardContent}>
              <div className={styles.featureIcon}>{feature.icon}</div>
              <h3>{feature.title}</h3>
            </div>
            {hoveredCard === index && (
              <div className={styles.featureTooltip}>
                <p><strong>{feature.title}</strong><br />{feature.description}</p>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default CrossroadsLanding;

