import React, { useState, useEffect } from 'react';
import StarsCanvas from '@components/stars/stars';
import HUD from '../../components/hud/hud';
import styles from './index.module.scss';

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
    erebus: true, // Contracts are running
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
      { key: 'defi', delay: 3600 },
    ];

    sequence.forEach(({ key, delay }) => {
      setTimeout(() => {
        setSystemStatus((prev) => ({
          ...prev,
          [key]: true,
        }));
      }, delay);
    });
  };

  useEffect(() => {
    // Check wallet connection on mount if we have wallet info
    const walletType = localStorage.getItem('walletType');

    if (walletType) {
      checkWalletConnection();
    }
  }, []);

  const SESSION_TIMEOUT = 30 * 60 * 1000; // 30 minutes in milliseconds

  const isSessionValid = () => {
    const lastAuth = localStorage.getItem('lastAuthTime');

    if (!lastAuth) {
      return false;
    }

    const timeSinceAuth = Date.now() - Number.parseInt(lastAuth);

    return timeSinceAuth < SESSION_TIMEOUT;
  };

  const updateAuthTime = () => {
    localStorage.setItem('lastAuthTime', Date.now().toString());
  };

  const checkWalletConnection = async () => {
    const savedPublicKey = localStorage.getItem('walletPublickey');
    const lastAuth = localStorage.getItem('lastAuthTime');
    const walletType = localStorage.getItem('walletType');

    if (!savedPublicKey || !lastAuth || !isSessionValid() || !walletType) {
      // Clear expired or invalid session data
      localStorage.removeItem('walletPublickey');
      localStorage.removeItem('walletType');
      localStorage.removeItem('lastAuthTime');
      localStorage.removeItem('hasSeenHUD');
      setWalletConnected(false);
      setPublicKey(null);

      return;
    }

    try {
      if (walletType === 'phantom' && 'phantom' in window) {
        const provider = (window as any).phantom?.solana;

        if (provider) {
          // Try to reconnect with existing session
          await provider.connect({ onlyIfTrusted: true });
          setPublicKey(savedPublicKey);
          setWalletConnected(true);
          localStorage.setItem('hasSeenHUD', 'true');
          updateAuthTime();

          return;
        }
      } else if (walletType === 'metamask' && window.ethereum) {
        // For MetaMask, check if we can access accounts
        const accounts = await window.ethereum.request({ method: 'eth_accounts' });

        if (accounts.length > 0 && accounts[0] === savedPublicKey) {
          setPublicKey(savedPublicKey);
          setWalletConnected(true);
          localStorage.setItem('hasSeenHUD', 'true');
          updateAuthTime();

          return;
        }
      }
    } catch (error) {
      console.log('Auto-reconnect failed:', error);
    }

    // Clear session data if reconnection failed
    localStorage.removeItem('walletPublickey');
    localStorage.removeItem('walletType');
    localStorage.removeItem('lastAuthTime');
    localStorage.removeItem('hasSeenHUD');
    setWalletConnected(false);
    setPublicKey(null);
  };

  const handleDisconnect = async () => {
    const walletType = localStorage.getItem('walletType');

    try {
      if (walletType === 'phantom' && 'phantom' in window) {
        const provider = (window as any).phantom?.solana;

        if (provider) {
          await provider.disconnect();
        }
      } else if (walletType === 'metamask' && window.ethereum) {
        // MetaMask doesn't have a direct disconnect method
        // We'll just clear the local session
      }
    } catch (error) {
      console.error('Error disconnecting wallet:', error);
    }

    // Clear wallet state
    setWalletConnected(false);
    setPublicKey(null);

    // Clear all session data
    localStorage.removeItem('walletPublickey');
    localStorage.removeItem('walletType');
    localStorage.removeItem('lastAuthTime');
    localStorage.removeItem('hasSeenHUD');
  };

  const handleConnectWallet = async (walletType?: 'phantom' | 'metamask') => {
    // If no wallet type specified, show selection modal
    if (!walletType) {
      showWalletSelectionModal();
      return;
    }

    try {
      if (walletType === 'phantom') {
        await connectPhantomWallet();
      } else if (walletType === 'metamask') {
        await connectMetaMaskWallet();
      }
    } catch (error) {
      console.error(`Failed to connect ${walletType} wallet:`, error);
      alert(`Failed to connect ${walletType}. Please try again.`);
    }
  };

  const connectPhantomWallet = async () => {
    console.log('Attempting to connect Phantom wallet...');
    
    if (!('phantom' in window)) {
      alert('👻 PHANTOM NOT FOUND\n\nPlease install the Phantom browser extension to continue.\n\nClick OK to open the Phantom website.');
      window.open('https://phantom.app/', '_blank');
      return;
    }

    const provider = (window as any).phantom?.solana;

    if (!provider) {
      alert('👻 PHANTOM NOT INITIALIZED\n\nPhantom wallet extension found but not properly initialized.\n\nPlease refresh the page or restart your browser.');
      return;
    }

    try {
      console.log('Phantom detected, requesting connection...');
      
      // This will trigger the Phantom popup UI
      const response = await provider.connect();
      
      console.log('Phantom connection response:', response);

      if (response.publicKey) {
        setPublicKey(response.publicKey.toString());
        setWalletConnected(true);
        localStorage.setItem('walletPublickey', response.publicKey.toString());
        localStorage.setItem('walletType', 'phantom');
        localStorage.setItem('hasSeenHUD', 'true');
        updateAuthTime();

        console.log('Phantom wallet connected successfully:', response.publicKey.toString());
        
        // Success feedback
        const shortKey = response.publicKey.toString();
        alert(`🔓 PHANTOM CONNECTED\n\nWallet: ${shortKey.slice(0, 4)}...${shortKey.slice(-4)}\n\nYou now have access to all NullEye features!`);
      }
    } catch (error: any) {
      console.error('Phantom connection failed:', error);
      
      if (error.code === 4001 || error.message?.includes('User rejected')) {
        // User rejected the request
        alert('🚫 CONNECTION CANCELLED\n\nWallet connection was cancelled by user.');
      } else if (error.code === -32002) {
        // Request already pending
        alert('⏳ CONNECTION PENDING\n\nPlease check your Phantom extension - a connection request is already pending.');
      } else {
        alert(`❌ PHANTOM CONNECTION FAILED\n\nError: ${error.message || 'Unknown error'}\n\nPlease try again or check your Phantom extension.`);
      }
    }
  };

  const connectMetaMaskWallet = async () => {
    console.log('Attempting to connect MetaMask wallet...');
    
    if (typeof window.ethereum === 'undefined') {
      alert('🦊 METAMASK NOT FOUND\n\nPlease install the MetaMask browser extension to continue.\n\nClick OK to open the MetaMask website.');
      window.open('https://metamask.io/', '_blank');
      return;
    }

    try {
      console.log('MetaMask detected, requesting account access...');
      
      // This will trigger the MetaMask popup UI
      const accounts = await window.ethereum.request({ 
        method: 'eth_requestAccounts' 
      });

      console.log('MetaMask accounts received:', accounts);

      if (accounts.length > 0) {
        setPublicKey(accounts[0]);
        setWalletConnected(true);
        localStorage.setItem('walletPublickey', accounts[0]);
        localStorage.setItem('walletType', 'metamask');
        localStorage.setItem('hasSeenHUD', 'true');
        updateAuthTime();

        console.log('MetaMask wallet connected successfully:', accounts[0]);
        
        // Success feedback
        alert(`🔓 METAMASK CONNECTED\n\nWallet: ${accounts[0].slice(0, 6)}...${accounts[0].slice(-4)}\n\nYou now have access to all NullEye features!`);
      }
    } catch (error: any) {
      console.error('MetaMask connection failed:', error);
      
      if (error.code === 4001) {
        // User rejected the request
        alert('🚫 CONNECTION CANCELLED\n\nWallet connection was cancelled by user.');
      } else if (error.code === -32002) {
        // Request already pending
        alert('⏳ CONNECTION PENDING\n\nPlease check your MetaMask extension - a connection request is already pending.');
      } else {
        alert(`❌ METAMASK CONNECTION FAILED\n\nError: ${error.message}\n\nPlease try again or check your MetaMask extension.`);
      }
    }
  };

  const showWalletSelectionModal = () => {
    console.log('showWalletSelectionModal called');
    const hasPhantom = 'phantom' in window && (window as any).phantom?.solana;
    const hasMetaMask = typeof window.ethereum !== 'undefined';
    
    console.log('Wallet detection:', { hasPhantom, hasMetaMask });

    if (!hasPhantom && !hasMetaMask) {
      alert('🔒 NO WEB3 WALLETS DETECTED\n\nPlease install a Web3 wallet to continue:\n\n• MetaMask (Ethereum): https://metamask.io/\n• Phantom (Solana): https://phantom.app/\n\nRefresh the page after installation.');
      return;
    }

    // If only one wallet is available, use it directly
    if (hasPhantom && !hasMetaMask) {
      console.log('Only Phantom detected, connecting directly');
      handleConnectWallet('phantom');
      return;
    }

    if (hasMetaMask && !hasPhantom) {
      console.log('Only MetaMask detected, connecting directly');
      handleConnectWallet('metamask');
      return;
    }

    // Both wallets available - show selection
    const walletOptions = [];
    if (hasPhantom) walletOptions.push('Phantom (Solana)');
    if (hasMetaMask) walletOptions.push('MetaMask (Ethereum)');

    // Create a better selection dialog
    const message = `🔐 WALLET SELECTION\n\nMultiple Web3 wallets detected. Please choose:\n\n${walletOptions.map((wallet, index) => `${index + 1}. ${wallet}`).join('\n')}\n\nClick OK for Phantom, Cancel for MetaMask`;
    
    const usePhantom = confirm(message);
    
    if (usePhantom) {
      console.log('User selected Phantom');
      handleConnectWallet('phantom');
    } else {
      console.log('User selected MetaMask');
      handleConnectWallet('metamask');
    }
  };

  return (
    <div
      className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]} ${isInitialized ? styles.initialized : ''}`}
    >
      <div className={styles.backgroundImage} />
      <StarsCanvas theme={currentTheme === 'dark' ? 'null' : currentTheme} />
      <div className={`${styles.scene} ${showHUD ? styles.hudActive : ''}`}>
        {/* System status panel moved to HUD component */}
      </div>
      {showHUD && isInitialized && (
        <HUD
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
        />
      )}

      {/* Floating hint for new features */}
      {isInitialized && systemStatus.portfolio && systemStatus.defi && (
        <div className={styles.newFeaturesHint}>
          <div className={styles.hintContent}>
            <span className={styles.hintIcon}>🚀</span>
            <span className={styles.hintText}>
              NEW: Portfolio & DeFi Trading Agents Available - Access via NullEye
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default Home;
