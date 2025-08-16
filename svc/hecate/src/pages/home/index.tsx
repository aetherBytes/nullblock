import React, { useState, useEffect } from 'react';
import StarsCanvas from '@components/stars/stars';
import HUD from '../../components/hud/hud';
import { 
  createWalletChallenge, 
  verifyWalletSignature, 
  checkErebusHealth,
  detectWallets,
  initiateWalletConnection,
  getWalletStatus
} from '../../common/services/erebus-api';
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
  const [showWalletModal, setShowWalletModal] = useState<boolean>(false);
  const [isConnecting, setIsConnecting] = useState<boolean>(false);
  const [isMetaMaskPending, setIsMetaMaskPending] = useState<boolean>(false);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<'error' | 'info'>('error');

  // Debug showWalletModal state changes
  useEffect(() => {
    console.log('showWalletModal state changed to:', showWalletModal);
  }, [showWalletModal]);
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

  // Helper functions for setting different types of messages
  const setInfoMessage = (message: string) => {
    setConnectionError(message);
    setMessageType('info');
  };

  const setErrorMessage = (message: string) => {
    setConnectionError(message);
    setMessageType('error');
  };

  const clearMessage = () => {
    setConnectionError(null);
    setMessageType('error');
  };

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
    console.log('=== handleConnectWallet START ===');
    console.log('handleConnectWallet called with walletType:', walletType);
    console.log('typeof walletType:', typeof walletType);
    
    // If no wallet type specified, show selection modal
    if (!walletType) {
      console.log('No walletType specified, showing selection modal');
      showWalletSelectionModal();
      return;
    }

    console.log('About to connect to wallet:', walletType);
    console.log('walletType === "phantom":', walletType === 'phantom');
    console.log('walletType === "metamask":', walletType === 'metamask');
    
    setIsConnecting(true);
    clearMessage();

    try {
      if (walletType === 'metamask') {
        console.log('EXECUTING: connectMetaMaskWallet()');
        await connectMetaMaskWallet();
      } else if (walletType === 'phantom') {
        console.log('EXECUTING: connectPhantomWallet()');
        await connectPhantomWallet();
      } else {
        console.error('UNKNOWN WALLET TYPE:', walletType);
      }
      setShowWalletModal(false);
    } catch (error) {
      console.error(`Failed to connect ${walletType} wallet:`, error);
      setErrorMessage(`Failed to connect ${walletType}. Please try again.`);
    } finally {
      setIsConnecting(false);
    }
    console.log('=== handleConnectWallet END ===');
  };

  const connectPhantomWallet = async () => {
    console.log('=== PHANTOM WALLET CONNECTION START ===');
    console.log('Attempting to connect Phantom wallet via backend...');
    
    if (!('phantom' in window)) {
      setInfoMessage('Phantom wallet not found. Please install the Phantom browser extension.');
      window.open('https://phantom.app/', '_blank');
      throw new Error('Phantom not installed');
    }

    const provider = (window as any).phantom?.solana;

    if (!provider) {
      setInfoMessage('Phantom wallet extension found but not properly initialized. Please refresh the page.');
      throw new Error('Phantom not initialized');
    }

    try {
      console.log('Phantom detected, requesting connection...');
      
      // First connect to get public key
      const response = await provider.connect();
      console.log('Phantom connection response:', response);

      if (!response.publicKey) {
        throw new Error('Failed to get public key from Phantom');
      }

      const walletAddress = response.publicKey.toString();
      console.log('Connected to Phantom wallet:', walletAddress);

      // Initiate connection via backend
      console.log('Initiating wallet connection via backend...');
      const connectionResponse = await initiateWalletConnection('phantom', walletAddress, walletAddress);
      
      if (!connectionResponse.success) {
        throw new Error(`Connection failed: ${connectionResponse.message}`);
      }

      // Create challenge via Erebus
      console.log('Creating authentication challenge via Erebus...');
      const challengeResponse = await createWalletChallenge(walletAddress, 'phantom');
      
      // Sign the challenge message
      console.log('Requesting signature for challenge...');
      const message = new TextEncoder().encode(challengeResponse.message);
      const signedMessage = await provider.signMessage(message, 'utf8');
      
      // Convert signature to string format expected by Erebus
      const signature = Array.from(signedMessage.signature).toString();
      
      // Verify signature via Erebus
      console.log('Verifying signature via Erebus...');
      const verifyResponse = await verifyWalletSignature(
        challengeResponse.challenge_id, 
        signature, 
        walletAddress
      );

      if (verifyResponse.success) {
        // Store authentication data
        setPublicKey(walletAddress);
        setWalletConnected(true);
        localStorage.setItem('walletPublickey', walletAddress);
        localStorage.setItem('walletType', 'phantom');
        localStorage.setItem('hasSeenHUD', 'true');
        localStorage.setItem('sessionToken', verifyResponse.session_token || '');
        updateAuthTime();

        console.log('Phantom wallet authenticated successfully via backend!');
      } else {
        throw new Error(`Authentication failed: ${verifyResponse.message}`);
      }
    } catch (error: any) {
      console.error('Phantom connection failed:', error);
      
      if (error.code === 4001 || error.message?.includes('User rejected')) {
        setInfoMessage('Connection cancelled by user.');
        throw new Error('User rejected connection');
      } else if (error.code === -32002) {
        setInfoMessage('Phantom is processing another request. Please check your extension or wait a moment and try again.');
        throw new Error('Request pending');
      } else {
        setErrorMessage(`Phantom connection failed: ${error.message || 'Unknown error'}`);
        throw error;
      }
    }
  };

  const connectMetaMaskWallet = async () => {
    // Prevent multiple simultaneous requests
    if (isMetaMaskPending) {
      console.log('MetaMask request already pending, skipping...');
      setInfoMessage('MetaMask connection in progress. Please check your MetaMask extension.');
      return;
    }

    try {
      console.log('=== METAMASK WALLET CONNECTION START ===');
      console.log('Attempting to connect MetaMask wallet via backend...');
      setIsMetaMaskPending(true);
      
      // Check for MetaMask specifically
      const getMetaMaskProvider = () => {
        console.log('Getting MetaMask provider...');
        if (typeof window.ethereum === 'undefined') {
          console.log('window.ethereum is undefined');
          return null;
        }
        
        console.log('window.ethereum exists:', window.ethereum);
        console.log('window.ethereum.providers:', window.ethereum.providers);
        
        // If multiple providers exist
        if (window.ethereum.providers && Array.isArray(window.ethereum.providers)) {
          console.log('Multiple providers found, looking for MetaMask...');
          const provider = window.ethereum.providers.find((provider: any) => provider.isMetaMask);
          console.log('Found MetaMask provider:', provider);
          return provider;
        }
        
        // If only one provider and it's MetaMask
        if (window.ethereum.isMetaMask) {
          console.log('Single MetaMask provider found');
          return window.ethereum;
        }
        
        console.log('No MetaMask provider found');
        return null;
      };

      const metamaskProvider = getMetaMaskProvider();
      console.log('Final metamaskProvider:', metamaskProvider);
      
      if (!metamaskProvider) {
        console.log('MetaMask provider not found, showing error');
        setInfoMessage('MetaMask wallet not found. Please install MetaMask or ensure it is enabled.');
        window.open('https://metamask.io/', '_blank');
        throw new Error('MetaMask not installed');
      }

      console.log('About to request accounts...');
      console.log('MetaMask provider found:', metamaskProvider);
      console.log('MetaMask detected, requesting account access...');
      
      // Get account access from MetaMask specifically
      console.log('About to request accounts from MetaMask...');
      
      // CRITICAL: Make the MetaMask request immediately and try to bring extension to front
      console.log('Making immediate MetaMask request to preserve user gesture...');
      
      // Show gentle notification about MetaMask popup
      setInfoMessage('🦊 Please check your MetaMask extension for the connection request.');
      
      const accounts = await metamaskProvider.request({ 
        method: 'eth_requestAccounts' 
      });

      console.log('MetaMask accounts received:', accounts);
      console.log('Number of accounts:', accounts.length);
      
      // Clear the notification since account access succeeded
      clearMessage();

      if (accounts.length === 0) {
        throw new Error('No accounts available from MetaMask');
      }

      const walletAddress = accounts[0];
      console.log('Connected to MetaMask wallet:', walletAddress);

      // Initiate connection via backend
      console.log('Initiating wallet connection via backend...');
      const connectionResponse = await initiateWalletConnection('metamask', walletAddress);
      
      if (!connectionResponse.success) {
        throw new Error(`Connection failed: ${connectionResponse.message}`);
      }
      console.log('Backend connection response:', connectionResponse);

      // Create challenge via Erebus
      console.log('Creating authentication challenge via Erebus...');
      let challengeResponse;
      try {
        challengeResponse = await createWalletChallenge(walletAddress, 'metamask');
        console.log('Challenge response:', challengeResponse);
        console.log('Challenge response type:', typeof challengeResponse);
        console.log('Challenge message exists:', !!challengeResponse.message);
      } catch (challengeError) {
        console.error('Challenge creation failed:', challengeError);
        throw challengeError;
      }

      // Sign the challenge message with MetaMask
      console.log('Requesting signature for challenge...');
      console.log('Challenge message to sign:', challengeResponse.message);
      console.log('About to call personal_sign on MetaMask...');
      
      // CRITICAL: Make signature request immediately to preserve user gesture context
      console.log('Making immediate MetaMask signature request...');
      
      // Show gentle notification about signature request
      setInfoMessage('🦊 Please check your MetaMask extension to sign the request.');
      
      let signature;
      try {
        signature = await metamaskProvider.request({
          method: 'personal_sign',
          params: [challengeResponse.message, walletAddress],
        });
        console.log('Signature received:', signature);
        
        // Clear the notification since signature succeeded
        clearMessage();
      } catch (signError) {
        console.error('Signature request failed:', signError);
        throw signError;
      }

      // Verify signature via Erebus
      console.log('Verifying signature via Erebus...');
      const verifyResponse = await verifyWalletSignature(
        challengeResponse.challenge_id, 
        signature, 
        walletAddress
      );

      if (verifyResponse.success) {
        // Store authentication data
        setPublicKey(walletAddress);
        setWalletConnected(true);
        localStorage.setItem('walletPublickey', walletAddress);
        localStorage.setItem('walletType', 'metamask');
        localStorage.setItem('hasSeenHUD', 'true');
        localStorage.setItem('sessionToken', verifyResponse.session_token || '');
        updateAuthTime();

        console.log('MetaMask wallet authenticated successfully via backend!');
      } else {
        throw new Error(`Authentication failed: ${verifyResponse.message}`);
      }
    } catch (error: any) {
      console.error('MetaMask connection failed:', error);
      
      if (error.code === 4001) {
        setInfoMessage('Connection cancelled by user.');
        throw new Error('User rejected connection');
      } else if (error.code === -32002) {
        setInfoMessage('MetaMask is processing another request. Please check your extension or wait a moment and try again.');
        throw new Error('Request pending');
      } else if (error.code === -32603) {
        setErrorMessage('MetaMask encountered an issue. Please refresh the page and try again.');
        throw new Error('Internal error');
      } else {
        setErrorMessage(`MetaMask connection failed: ${error.message}`);
        throw error;
      }
    } finally {
      setIsMetaMaskPending(false);
    }
  };

  const showWalletSelectionModal = async () => {
    console.log('showWalletSelectionModal called');
    
    try {
      // Detect available wallets on frontend
      const availableWallets: string[] = [];
      if ('phantom' in window && (window as any).phantom?.solana) {
        availableWallets.push('phantom');
      }
      if (typeof window.ethereum !== 'undefined') {
        availableWallets.push('metamask');
      }

      console.log('Frontend wallet detection:', availableWallets);

      // Use backend to get comprehensive wallet information
      const detectionResponse = await detectWallets(availableWallets);
      console.log('Backend wallet detection response:', detectionResponse);

      if (detectionResponse.available_wallets.length === 0) {
        setInfoMessage('No Web3 wallets detected. Please install MetaMask or Phantom and refresh the page.');
        setShowWalletModal(true);
        return;
      }

      // Always show modal to let user choose - remove auto-connection logic
      console.log('Multiple wallets detected or installation needed, showing selection modal');
      setShowWalletModal(true);
      setConnectionError(null);

      // Store detection response for modal display
      localStorage.setItem('walletDetectionResponse', JSON.stringify(detectionResponse));
    } catch (error) {
      console.error('Failed to detect wallets:', error);
      setInfoMessage('Failed to detect wallets. Please refresh the page and try again.');
      setShowWalletModal(true);
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
          onConnectWallet={(walletType?: 'phantom' | 'metamask') => handleConnectWallet(walletType)}
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
              NEW: Portfolio & DeFi Trading Agents Available - Access via NullView
            </span>
          </div>
        </div>
      )}

      {/* Wallet Selection Modal */}
      {showWalletModal && (
        <div 
          className={styles.modalOverlay} 
          onClick={() => {
            console.log('Modal overlay clicked, closing modal');
            setShowWalletModal(false);
          }}
        >
          <div className={styles.walletModal} onClick={(e) => {
            console.log('Wallet modal clicked, preventing propagation');
            e.stopPropagation();
          }}>
            <div className={styles.modalHeader}>
              <h2>🔐 Connect Wallet</h2>
              <button 
                className={styles.closeButton} 
                onClick={() => {
                  console.log('Close button clicked');
                  setShowWalletModal(false);
                }}
              >
                ×
              </button>
            </div>
            
            <div className={styles.modalContent}>
              <p>Choose a Web3 wallet to connect to Nullblock:</p>
              
              <div className={styles.walletOptions}>
                {('phantom' in window && (window as any).phantom?.solana) && (
                  <button
                    className={styles.walletButton}
                    onClick={() => handleConnectWallet('phantom')}
                    disabled={isConnecting}
                  >
                    <div className={styles.walletIcon}>👻</div>
                    <div className={styles.walletInfo}>
                      <div className={styles.walletName}>Phantom</div>
                      <div className={styles.walletDescription}>Solana Wallet</div>
                    </div>
                    {isConnecting && <div className={styles.connecting}>Connecting...</div>}
                  </button>
                )}
                
                {(typeof window.ethereum !== 'undefined') && (
                  <button
                    className={styles.walletButton}
                    onClick={() => handleConnectWallet('metamask')}
                    disabled={isConnecting}
                  >
                    <div className={styles.walletIcon}>🦊</div>
                    <div className={styles.walletInfo}>
                      <div className={styles.walletName}>MetaMask</div>
                      <div className={styles.walletDescription}>Ethereum Wallet</div>
                    </div>
                    {isConnecting && <div className={styles.connecting}>Connecting...</div>}
                  </button>
                )}
              </div>

              {connectionError && (
                <div className={messageType === 'error' ? styles.errorMessage : styles.infoMessage}>
                  {connectionError}
                </div>
              )}

              <div className={styles.installPrompt}>
                <p>Don't have a wallet?</p>
                <div className={styles.installLinks}>
                  <a href="https://metamask.io/" target="_blank" rel="noopener noreferrer">
                    Install MetaMask
                  </a>
                  <a href="https://phantom.app/" target="_blank" rel="noopener noreferrer">
                    Install Phantom
                  </a>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Home;
