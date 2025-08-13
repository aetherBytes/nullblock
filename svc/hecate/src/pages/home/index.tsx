import React, { useState, useEffect } from 'react';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import HUD from '../../components/hud/hud';

const Home: React.FC = () => {
  const [, setWalletConnected] = useState<boolean>(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [showHUD, setShowHUD] = useState<boolean>(true);
  const [currentTheme, setCurrentTheme] = useState<'null' | 'light'>('light');
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [systemStatus, setSystemStatus] = useState({
    hud: false,
    mcp: false,
    orchestration: false,
    agents: false,
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
      { key: 'agents', delay: 2000 }
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

  return (
    <div className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]} ${isInitialized ? styles.initialized : ''}`}>
      <div className={styles.backgroundImage} />
      <StarsCanvas theme={currentTheme} />
      <div className={`${styles.scene} ${showHUD ? styles.hudActive : ''}`}>
        {isInitialized && (
          <div className={styles.statusIndicator}>
            <div className={styles.systemStatusPanel}>
              <div className={styles.statusHeader}>
                <span className={styles.statusTitle}>NULLBLOCK SYSTEMS</span>
              </div>
              <div className={styles.statusGrid}>
                <div className={styles.statusItem}>
                  <span className={`${styles.statusDot} ${systemStatus.hecate ? styles.online : styles.offline}`}></span>
                  <span className={styles.statusLabel}>HECATE</span>
                  <span className={styles.statusValue}>{systemStatus.hecate ? 'ONLINE' : 'OFFLINE'}</span>
                </div>
                <div className={styles.statusItem}>
                  <span className={`${styles.statusDot} ${systemStatus.erebus ? styles.online : styles.offline}`}></span>
                  <span className={styles.statusLabel}>EREBUS</span>
                  <span className={styles.statusValue}>{systemStatus.erebus ? 'ONLINE' : 'OFFLINE'}</span>
                </div>
                <div className={styles.statusItem}>
                  <span className={`${styles.statusDot} ${systemStatus.hud ? styles.online : styles.offline}`}></span>
                  <span className={styles.statusLabel}>HUD</span>
                  <span className={styles.statusValue}>{systemStatus.hud ? 'OPERATIONAL' : 'INITIALIZING'}</span>
                </div>
                <div className={styles.statusItem}>
                  <span className={`${styles.statusDot} ${systemStatus.mcp ? styles.online : styles.offline}`}></span>
                  <span className={styles.statusLabel}>MCP</span>
                  <span className={styles.statusValue}>{systemStatus.mcp ? 'ACTIVE' : 'STARTING'}</span>
                </div>
                <div className={styles.statusItem}>
                  <span className={`${styles.statusDot} ${systemStatus.orchestration ? styles.online : styles.offline}`}></span>
                  <span className={styles.statusLabel}>ORCHESTRATION</span>
                  <span className={styles.statusValue}>{systemStatus.orchestration ? 'READY' : 'LOADING'}</span>
                </div>
                <div className={styles.statusItem}>
                  <span className={`${styles.statusDot} ${systemStatus.agents ? styles.online : styles.offline}`}></span>
                  <span className={styles.statusLabel}>AGENTS</span>
                  <span className={styles.statusValue}>{systemStatus.agents ? 'DEPLOYED' : 'SPAWNING'}</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
      {showHUD && isInitialized && <HUD 
        publicKey={publicKey} 
        onDisconnect={handleDisconnect}
        theme={currentTheme}
        onClose={() => {
          setShowHUD(false);
        }}
        onThemeChange={(theme) => {
          if (theme === 'cyber') {
            setCurrentTheme('null');
            localStorage.setItem('currentTheme', 'null');
          } else {
            setCurrentTheme(theme as 'null' | 'light');
            localStorage.setItem('currentTheme', theme);
          }
        }}
      />}
    </div>
  );
};

export default Home;