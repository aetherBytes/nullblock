import React, { useState, useEffect } from 'react';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import HUD from '../../components/hud/hud';

// Extend Window interface for ethereum
declare global {
  interface Window {
    ethereum?: any;
  }
}

const Home: React.FC = () => {
  const [walletConnected, setWalletConnected] = useState<boolean>(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [showHUD, setShowHUD] = useState<boolean>(true);
  const [currentTheme, setCurrentTheme] = useState<'null' | 'light' | 'dark'>('dark');
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [systemStatus, setSystemStatus] = useState({
    hud: false,
    mcp: false,
    orchestration: false,
    agents: false,
    portfolio: false,
    defi: false,
    social: false,
    arbitrage: false,
    hecate: true, // Frontend is running
    erebus: true  // Contracts are running
  });

  // Initialize state from localStorage on component mount
  useEffect(() => {
    // Check if we have a saved wallet connection
    const savedPublicKey = localStorage.getItem('walletPublickey');
    const lastAuth = localStorage.getItem('lastAuthTime');
    const savedTheme = localStorage.getItem('currentTheme');
    
    // Set initial states based on localStorage
    if (savedPublicKey && lastAuth && isSessionValid()) {
      setPublicKey(savedPublicKey);
      setWalletConnected(true);
    } else {
      setWalletConnected(false);
      setPublicKey(null);
    }
    
    if (savedTheme) {
      setCurrentTheme(savedTheme as 'null' | 'light');
    }
    
    // Set initialization flag with slight delay for smooth startup
    setTimeout(() => {
      setIsInitialized(true);
      // Simulate system startup sequence
      startSystemSequence();
    }, 500);
  }, []);

  const startSystemSequence = () => {
    const sequence = [
      { key: 'hud', delay: 800 },
      { key: 'mcp', delay: 1200 },
      { key: 'orchestration', delay: 1600 },
      { key: 'agents', delay: 2000 },
      { key: 'arbitrage', delay: 2400 },
      { key: 'social', delay: 2800 },
      { key: 'portfolio', delay: 3200 },
      { key: 'defi', delay: 3600 }
    ];

    sequence.forEach(({ key, delay }) => {
      setTimeout(() => {
        setSystemStatus(prev => ({
          ...prev,
          [key]: true
        }));
      }, delay);
    });
  };

  useEffect(() => {
    const phantomExists = 'phantom' in window && (window as any).phantom?.solana;

    // Check wallet connection on mount
    if (phantomExists) {
      checkWalletConnection();
    }
  }, []);

  const SESSION_TIMEOUT = 30 * 60 * 1000; // 30 minutes in milliseconds

  const isSessionValid = () => {
    const lastAuth = localStorage.getItem('lastAuthTime');
    if (!lastAuth) return false;
    
    const timeSinceAuth = Date.now() - parseInt(lastAuth);
    return timeSinceAuth < SESSION_TIMEOUT;
  };

  const updateAuthTime = () => {
    localStorage.setItem('lastAuthTime', Date.now().toString());
  };

  const checkWalletConnection = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        const savedPublicKey = localStorage.getItem('walletPublickey');
        const lastAuth = localStorage.getItem('lastAuthTime');
        
        if (savedPublicKey && lastAuth && isSessionValid()) {
          try {
            // Try to reconnect with existing session
            await provider.connect({ onlyIfTrusted: true });
            
            // If we get here, connection was successful
            setPublicKey(savedPublicKey);
            setWalletConnected(true);
            localStorage.setItem('walletPublickey', savedPublicKey);
            localStorage.setItem('hasSeenHUD', 'true');
            updateAuthTime();
          } catch (error) {
            console.log('Auto-reconnect failed:', error);
          }
        }
        
        // Clear session data if we get here (either expired or failed)
        localStorage.removeItem('walletPublickey');
        localStorage.removeItem('lastAuthTime');
        localStorage.removeItem('hasSeenHUD');
        setWalletConnected(false);
        setPublicKey(null);
      }
    }
  };

  const handleDisconnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          await provider.disconnect();
          setWalletConnected(false);
          setPublicKey(null);
          // Clear all session data
          localStorage.removeItem('walletPublickey');
          localStorage.removeItem('lastAuthTime');
          localStorage.removeItem('hasSeenHUD');
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  const handleConnectWallet = async () => {
    // Check for Phantom first
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          const response = await provider.connect();
          setPublicKey(response.publicKey.toString());
          setWalletConnected(true);
          localStorage.setItem('walletPublickey', response.publicKey.toString());
          localStorage.setItem('hasSeenHUD', 'true');
          updateAuthTime();
        } catch (error) {
          console.error('Failed to connect Phantom wallet:', error);
        }
      }
    } else {
      // Check for MetaMask
      if (typeof window.ethereum !== 'undefined') {
        try {
          const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
          if (accounts.length > 0) {
            setPublicKey(accounts[0]);
            setWalletConnected(true);
            localStorage.setItem('walletPublickey', accounts[0]);
            localStorage.setItem('hasSeenHUD', 'true');
            updateAuthTime();
          }
        } catch (error) {
          console.error('Failed to connect MetaMask wallet:', error);
        }
      } else {
        // No wallet detected
        alert('No Web3 wallet detected. Please install Phantom or MetaMask.');
      }
    }
  };

  return (
    <div className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]} ${isInitialized ? styles.initialized : ''}`}>
      <div className={styles.backgroundImage} />
      <StarsCanvas theme={currentTheme === 'dark' ? 'null' : currentTheme} />
      <div className={`${styles.scene} ${showHUD ? styles.hudActive : ''}`}>
        {/* System status panel moved to HUD component */}
      </div>
      {showHUD && isInitialized && <HUD 
        publicKey={publicKey} 
        onDisconnect={handleDisconnect}
        onConnectWallet={handleConnectWallet}
        theme={currentTheme}
        systemStatus={systemStatus}
        onClose={() => {
          setShowHUD(false);
        }}
        onThemeChange={(theme) => {
          if (theme === 'cyber') {
            setCurrentTheme('null');
            localStorage.setItem('currentTheme', 'null');
          } else {
            setCurrentTheme(theme as 'null' | 'light' | 'dark');
            localStorage.setItem('currentTheme', theme);
          }
        }}
      />}
      
      {/* Floating hint for new features */}
      {isInitialized && systemStatus.portfolio && systemStatus.defi && (
        <div className={styles.newFeaturesHint}>
          <div className={styles.hintContent}>
            <span className={styles.hintIcon}>🚀</span>
            <span className={styles.hintText}>NEW: Portfolio & DeFi Trading Agents Available - Access via NullEye</span>
          </div>
        </div>
      )}
    </div>
  );
};

export default Home;