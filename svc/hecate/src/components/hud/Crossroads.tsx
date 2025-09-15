import React from 'react';
import styles from './hud.module.scss';

interface CrossroadsProps {
  publicKey: string | null;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
}

const Crossroads: React.FC<CrossroadsProps> = ({ publicKey, onConnectWallet }) => {
  if (!publicKey) {
    return (
      <div className={styles.crossroadsTab}>
        <h3>Crossroads</h3>
        <div className={styles.crossroadsContent}>
          <div className={styles.crossroadsWelcome}>
            <h4>Welcome to NullBlock</h4>
            <p>Your gateway to autonomous agent workflows and Web3 automation.</p>
          </div>

          <div className={styles.crossroadsFeatures}>
            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>🤖</div>
              <h5>Agent Orchestration</h5>
              <p>Deploy autonomous agents for trading, social monitoring, and DeFi operations.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>🔗</div>
              <h5>MCP Integration</h5>
              <p>Model Context Protocol for seamless AI agent communication and coordination.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>⚡</div>
              <h5>MEV Protection</h5>
              <p>Advanced MEV protection through Flashbots integration and privacy pools.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>📊</div>
              <h5>Analytics & Insights</h5>
              <p>Real-time portfolio analytics, risk assessment, and performance monitoring.</p>
            </div>
          </div>

          <div className={styles.crossroadsActions}>
            <h4>Get Started</h4>
            <div className={styles.actionButtons}>
              <button
                className={styles.actionButton}
                onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
              >
                📚 Documentation
              </button>
              <button
                className={styles.actionButton}
                onClick={() => window.open('https://x.com/Nullblock_io', '_blank')}
              >
                𝕏 Follow Updates
              </button>
              <button
                className={styles.actionButton}
                onClick={() => window.open('https://discord.gg/nullblock', '_blank')}
              >
                💬 Join Discord
              </button>
              <button
                className={styles.primaryActionButton}
                onClick={() => onConnectWallet()}
              >
                🚀 Connect & Launch
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.crossroadsTab}>
      <h3>Crossroads Marketplace</h3>
      <div className={styles.crossroadsContent}>
        <div className={styles.crossroadsWelcome}>
          <h4>Marketplace & Discovery Service</h4>
          <p>Test the new Crossroads marketplace for agents, workflows, tools, and MCP servers.</p>
        </div>

        <div className={styles.marketplaceTests}>
          <h4>API Testing</h4>
          <div className={styles.testGrid}>
            <div className={styles.testCard}>
              <h5>🏥 Service Health</h5>
              <button
                className={styles.testButton}
                onClick={async () => {
                  try {
                    const response = await fetch('/api/crossroads/health');
                    const data = await response.json();
                    alert(`Status: ${data.status}\nService: ${data.service}\nMessage: ${data.message}`);
                  } catch (error) {
                    alert(`Error: ${error}`);
                  }
                }}
              >
                Test Health
              </button>
            </div>

            <div className={styles.testCard}>
              <h5>📋 Get Listings</h5>
              <button
                className={styles.testButton}
                onClick={async () => {
                  try {
                    const response = await fetch('/api/marketplace/listings');
                    const data = await response.json();
                    alert(`Found ${data.listings?.length || 0} listings\n\nFirst listing: ${JSON.stringify(data.listings?.[0] || 'None', null, 2)}`);
                  } catch (error) {
                    alert(`Error: ${error}`);
                  }
                }}
              >
                Get Listings
              </button>
            </div>

            <div className={styles.testCard}>
              <h5>🤖 Discover Agents</h5>
              <button
                className={styles.testButton}
                onClick={async () => {
                  try {
                    const response = await fetch('/api/discovery/agents');
                    const data = await response.json();
                    alert(`Found ${data.agents?.length || 0} agents\n\nAgents: ${JSON.stringify(data.agents || [], null, 2)}`);
                  } catch (error) {
                    alert(`Error: ${error}`);
                  }
                }}
              >
                Discover Agents
              </button>
            </div>

            <div className={styles.testCard}>
              <h5>📊 System Stats</h5>
              <button
                className={styles.testButton}
                onClick={async () => {
                  try {
                    const response = await fetch('/api/admin/system/stats');
                    const data = await response.json();
                    alert(`System Stats:\nTotal Listings: ${data.total_listings || 0}\nActive Agents: ${data.active_agents || 0}\nUptime: ${data.uptime || 'N/A'}`);
                  } catch (error) {
                    alert(`Error: ${error}`);
                  }
                }}
              >
                Get Stats
              </button>
            </div>
          </div>
        </div>

        <div className={styles.createListing}>
          <h4>Create Test Listing</h4>
          <div className={styles.listingForm}>
            <select
              className={styles.formSelect}
              defaultValue="Agent"
              id="listingType"
            >
              <option value="Agent">Agent</option>
              <option value="Workflow">Workflow</option>
              <option value="Tool">Tool</option>
              <option value="McpServer">MCP Server</option>
              <option value="Dataset">Dataset</option>
              <option value="Model">Model</option>
            </select>
            <input
              type="text"
              className={styles.formInput}
              placeholder="Listing title..."
              id="listingTitle"
            />
            <textarea
              className={styles.formTextarea}
              placeholder="Description..."
              rows={3}
              id="listingDescription"
            />
            <button
              className={styles.createButton}
              onClick={async () => {
                const typeSelect = document.getElementById('listingType') as HTMLSelectElement;
                const titleInput = document.getElementById('listingTitle') as HTMLInputElement;
                const descriptionInput = document.getElementById('listingDescription') as HTMLTextAreaElement;

                if (!titleInput.value.trim()) {
                  alert('Please enter a title');
                  return;
                }

                try {
                  const response = await fetch('/api/marketplace/listings', {
                    method: 'POST',
                    headers: {
                      'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                      title: titleInput.value,
                      description: descriptionInput.value || 'Test listing',
                      listing_type: typeSelect?.value || 'Agent',
                      price: 0.0,
                      tags: ['test', 'demo']
                    })
                  });
                  const data = await response.json();
                  alert(`Listing created!\nID: ${data.listing?.id || 'N/A'}\nTitle: ${data.listing?.title || 'N/A'}`);

                  titleInput.value = '';
                  descriptionInput.value = '';
                } catch (error) {
                  alert(`Error: ${error}`);
                }
              }}
            >
              🚀 Create Listing
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Crossroads;