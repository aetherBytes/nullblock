import React, { useState } from 'react';
import { OnchainKitProvider } from '@coinbase/onchainkit';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WagmiProvider, createConfig, http } from 'wagmi';
import { base } from 'viem/chains';
import { coinbaseWallet } from 'wagmi/connectors';
import styles from './crossroads.module.scss';
import CrossroadsLanding from './landing/CrossroadsLanding';
import MarketplaceBrowser from './marketplace/MarketplaceBrowser';
import type { ServiceListing } from './types';

interface CrossroadsProps {
  publicKey: string | null;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
}

const config = createConfig({
  chains: [base],
  connectors: [
    coinbaseWallet({
      appName: 'NullBlock Crossroads',
      preference: 'smartWalletOnly',
    }),
  ],
  transports: {
    [base.id]: http(),
  },
});

const queryClient = new QueryClient();

type View = 'landing' | 'marketplace' | 'service-detail' | 'my-services';

const Crossroads: React.FC<CrossroadsProps> = ({ publicKey, onConnectWallet }) => {
  const [currentView, setCurrentView] = useState<View>(publicKey ? 'marketplace' : 'landing');
  const [selectedService, setSelectedService] = useState<ServiceListing | null>(null);

  // Watch publicKey changes - switch views on connect/disconnect
  React.useEffect(() => {
    if (publicKey) {
      // Wallet connected - switch to marketplace
      setCurrentView('marketplace');
    } else {
      // Wallet disconnected - reset to landing
      setCurrentView('landing');
      setSelectedService(null);
    }
  }, [publicKey]);

  const handleServiceClick = (service: ServiceListing) => {
    setSelectedService(service);
    setCurrentView('service-detail');
  };

  const handleBackToMarketplace = () => {
    setSelectedService(null);
    setCurrentView('marketplace');
  };

  const renderView = () => {
    switch (currentView) {
      case 'landing':
        return (
          <CrossroadsLanding
            onConnectWallet={() => {
              onConnectWallet();
              setCurrentView('marketplace');
            }}
          />
        );

      case 'marketplace':
        return <MarketplaceBrowser onServiceClick={handleServiceClick} />;

      case 'service-detail':
        return (
          <div>
            <button
              onClick={handleBackToMarketplace}
              style={{
                padding: '0.75rem 1.5rem',
                background: 'rgba(30, 41, 59, 0.8)',
                border: '1px solid rgba(100, 116, 139, 0.3)',
                color: '#e2e8f0',
                borderRadius: '0.5rem',
                cursor: 'pointer',
                marginBottom: '1.5rem',
              }}
            >
              ← Back to Marketplace
            </button>
            {selectedService && (
              <div
                style={{
                  background: 'rgba(30, 41, 59, 0.6)',
                  padding: '2rem',
                  borderRadius: '0.75rem',
                  border: '1px solid rgba(100, 116, 139, 0.3)',
                }}
              >
                <h2 style={{ fontSize: '2rem', marginBottom: '1rem', color: '#f1f5f9' }}>
                  {selectedService.title}
                </h2>
                <p style={{ color: '#94a3b8', marginBottom: '1.5rem' }}>
                  {selectedService.short_description}
                </p>
                <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', marginBottom: '2rem' }}>
                  {selectedService.capabilities.map((cap) => (
                    <span
                      key={cap}
                      style={{
                        padding: '0.5rem 1rem',
                        background: 'rgba(99, 102, 241, 0.1)',
                        border: '1px solid rgba(99, 102, 241, 0.3)',
                        borderRadius: '0.5rem',
                        color: '#6366f1',
                        fontSize: '0.875rem',
                      }}
                    >
                      {cap}
                    </span>
                  ))}
                </div>
                <p style={{ color: '#cbd5e1', marginBottom: '1rem' }}>
                  <strong>Version:</strong> {selectedService.version}
                </p>
                <p style={{ color: '#cbd5e1', marginBottom: '1rem' }}>
                  <strong>Endpoint:</strong> {selectedService.endpoint_url || 'N/A'}
                </p>
                <p style={{ color: '#cbd5e1', marginBottom: '1rem' }}>
                  <strong>Deployments:</strong> {selectedService.deployment_count}
                </p>
                <p style={{ color: '#cbd5e1', marginBottom: '2rem' }}>
                  <strong>Rating:</strong> {selectedService.rating_average ? `${selectedService.rating_average}/5` : 'No ratings yet'} ({selectedService.rating_count} reviews)
                </p>
                <button
                  style={{
                    padding: '1rem 2rem',
                    background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
                    color: 'white',
                    border: 'none',
                    borderRadius: '0.5rem',
                    fontSize: '1.125rem',
                    fontWeight: '600',
                    cursor: 'pointer',
                  }}
                  onClick={() => alert('Deploy functionality coming soon!')}
                >
                  Deploy Service
                </button>
              </div>
            )}
          </div>
        );

      case 'my-services':
        return (
          <div>
            <h2>My Services</h2>
            <p>Coming soon...</p>
          </div>
        );

      default:
        return <div>Unknown view</div>;
    }
  };

  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <OnchainKitProvider chain={base} apiKey={import.meta.env.VITE_ONCHAINKIT_API_KEY}>
          <div className={styles.crossroadsContainer}>
            {renderView()}
          </div>
        </OnchainKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
};

export default Crossroads;

