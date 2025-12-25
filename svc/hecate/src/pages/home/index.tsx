import React, { useState, useEffect, useCallback } from 'react';
import { VoidExperience } from '../../components/void-experience';
import HUD from '../../components/hud/hud';
import {
  createWalletChallenge,
  verifyWalletSignature,
  checkErebusHealth,
  detectWallets,
  getWalletStatus
} from '../../common/services/erebus-api';
import styles from './index.module.scss';


// Extend Window interface for ethereum
declare global {
  interface Window {
    ethereum?: any;
  }
}

// Login animation phases: black → stars → background → navbar → complete
type LoginAnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

// Session timeout constant
const SESSION_TIMEOUT_MS = 30 * 60 * 1000; // 30 minutes

// Helper to check if user has valid session (client-side only)
const checkExistingSession = (): { hasSession: boolean; publicKey: string | null } => {
  // Only run on client side (SSR safety)
  if (typeof window === 'undefined') {
    return { hasSession: false, publicKey: null };
  }

  try {
    const savedPublicKey = localStorage.getItem('walletPublickey');
    const lastAuth = localStorage.getItem('lastAuthTime');

    if (savedPublicKey && lastAuth) {
      const timeSinceAuth = Date.now() - Number.parseInt(lastAuth);
      if (timeSinceAuth < SESSION_TIMEOUT_MS) {
        return { hasSession: true, publicKey: savedPublicKey };
      }
    }
  } catch (e) {
    console.warn('Error checking existing session:', e);
  }
  return { hasSession: false, publicKey: null };
};

// Check session synchronously at module load for initial state (client-side only)
const initialSession = checkExistingSession();

const Home: React.FC = () => {
  // Initialize with existing session if available - start in 'black' phase to trigger animation
  const [walletConnected, setWalletConnected] = useState<boolean>(initialSession.hasSession);
  const [publicKey, setPublicKey] = useState<string | null>(initialSession.publicKey);
  const [showHUD, setShowHUD] = useState<boolean>(true);
  const [currentTheme, setCurrentTheme] = useState<'null' | 'light' | 'dark'>('null');
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [showWalletModal, setShowWalletModal] = useState<boolean>(false);
  const [isConnecting, setIsConnecting] = useState<boolean>(false);
  const [isMetaMaskPending, setIsMetaMaskPending] = useState<boolean>(false);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<'error' | 'info'>('error');
  const lastConnectionAttempt = React.useRef<number>(0);
  const [hudInitialTab, setHudInitialTab] = useState<'crossroads' | 'tasks' | 'agents' | 'logs' | 'hecate' | 'canvas' | null>(null);
  const [isMobileView, setIsMobileView] = useState<boolean>(false);

  // Login animation state - start in 'black' if user has existing session (triggers animation on refresh)
  const [loginAnimationPhase, setLoginAnimationPhase] = useState<LoginAnimationPhase>(
    initialSession.hasSession ? 'black' : 'idle'
  );
  // Pre-login animation state - for logged-out users landing page
  const [preLoginAnimationPhase, setPreLoginAnimationPhase] = useState<LoginAnimationPhase>(
    !initialSession.hasSession ? 'black' : 'idle'
  );
  const previousPublicKey = React.useRef<string | null>(initialSession.publicKey);
  const animationTriggered = React.useRef<boolean>(false);
  const preLoginAnimationTriggered = React.useRef<boolean>(false);

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

  // Initialize theme immediately to prevent flash
  useEffect(() => {
    const savedTheme = localStorage.getItem('currentTheme');
    if (savedTheme && (savedTheme === 'null' || savedTheme === 'light' || savedTheme === 'dark')) {
      setCurrentTheme(savedTheme as 'null' | 'light' | 'dark');
    } else {
      // Set default null theme if no valid saved theme
      setCurrentTheme('null');
      localStorage.setItem('currentTheme', 'null');
    }
  }, []);

  // Detect mobile view on mount and resize
  useEffect(() => {
    const checkMobileView = () => {
      setIsMobileView(window.innerWidth <= 768);
    };

    checkMobileView();
    window.addEventListener('resize', checkMobileView);

    return () => window.removeEventListener('resize', checkMobileView);
  }, []);

  // Start animation immediately on mount if user has session (page refresh scenario)
  useEffect(() => {
    if (initialSession.hasSession && !animationTriggered.current) {
      console.log('🎬 Immediate animation start for returning user');
      animationTriggered.current = true;

      // Already in 'black' phase from initial state, start the sequence
      // Stars and background fade in together
      setTimeout(() => {
        console.log('🌟🌌 Stars + Background fading in together...');
        setLoginAnimationPhase('background');
      }, 400);

      setTimeout(() => {
        console.log('⚡ Navbar flickering in...');
        setLoginAnimationPhase('navbar');
      }, 2500);

      setTimeout(() => {
        console.log('✅ Login animation complete');
        setLoginAnimationPhase('complete');
      }, 4000);
    }
  }, []); // Empty deps - run once on mount

  // Pre-login animation - for logged-out users on page load
  useEffect(() => {
    if (!initialSession.hasSession && !preLoginAnimationTriggered.current) {
      console.log('🎬 Pre-login animation start for new visitor');
      preLoginAnimationTriggered.current = true;

      // Already in 'black' phase from initial state, start the sequence
      setTimeout(() => {
        console.log('🌟 [Pre-login] Stars fading in...');
        setPreLoginAnimationPhase('stars');
      }, 400);

      setTimeout(() => {
        console.log('🌌 [Pre-login] Background + Mission text fading in...');
        setPreLoginAnimationPhase('background');
      }, 2000);

      setTimeout(() => {
        console.log('⚡ [Pre-login] Navbar + CTA flickering in...');
        setPreLoginAnimationPhase('navbar');
      }, 4500);

      setTimeout(() => {
        console.log('✅ [Pre-login] Animation complete');
        setPreLoginAnimationPhase('complete');
      }, 6000);
    }
  }, []); // Empty deps - run once on mount

  // Login animation sequence - triggers on fresh login (not page refresh)
  useEffect(() => {
    // Detect fresh login (publicKey changed from null to value)
    const isNewLogin = previousPublicKey.current === null && publicKey !== null;
    previousPublicKey.current = publicKey;

    if (isNewLogin && !animationTriggered.current) {
      animationTriggered.current = true;
      console.log('🎬 Starting login animation for new login...');

      // Skip black phase - void is already visible, just transition smoothly
      // Go directly to 'background' phase to start the transition
      setLoginAnimationPhase('background');

      // Phase 2: Complete the transition (clusters fade in via CSS)
      setTimeout(() => {
        console.log('✅ Login animation complete');
        setLoginAnimationPhase('complete');
      }, 1500);
    }

    // Reset animation when logging out and trigger pre-login animation
    if (publicKey === null && loginAnimationPhase !== 'idle') {
      setLoginAnimationPhase('idle');
      animationTriggered.current = false; // Allow animation to trigger again on next login

      // Trigger pre-login animation on logout
      preLoginAnimationTriggered.current = false;
      console.log('🎬 Starting pre-login animation after logout...');
      setPreLoginAnimationPhase('black');

      setTimeout(() => {
        console.log('🌟 [Post-logout] Stars fading in...');
        setPreLoginAnimationPhase('stars');
      }, 400);

      setTimeout(() => {
        console.log('🌌 [Post-logout] Background + Mission text fading in...');
        setPreLoginAnimationPhase('background');
      }, 2000);

      setTimeout(() => {
        console.log('⚡ [Post-logout] Navbar + CTA flickering in...');
        setPreLoginAnimationPhase('navbar');
      }, 4500);

      setTimeout(() => {
        console.log('✅ [Post-logout] Animation complete');
        setPreLoginAnimationPhase('complete');
        preLoginAnimationTriggered.current = true;
      }, 6000);
    }
  }, [publicKey, loginAnimationPhase]);

  // Initialize system on component mount
  // Note: publicKey and walletConnected are already initialized synchronously from localStorage
  useEffect(() => {
    // If no valid session was found during synchronous init, clear any stale data
    if (!initialSession.hasSession) {
      localStorage.removeItem('walletPublickey');
      localStorage.removeItem('walletType');
      localStorage.removeItem('lastAuthTime');
      localStorage.removeItem('hasSeenHUD');
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

    return timeSinceAuth < SESSION_TIMEOUT_MS;
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
          try {
            await provider.connect({ onlyIfTrusted: true });
            setPublicKey(savedPublicKey);
            setWalletConnected(true);
            localStorage.setItem('hasSeenHUD', 'true');
            updateAuthTime();
            return;
          } catch (phantomError: any) {
            console.log('Phantom auto-reconnect failed:', phantomError);
            // If auto-reconnect fails, disconnect to clear Phantom's internal state
            // This prevents -32603 errors on manual connection attempts
            try {
              await provider.disconnect();
            } catch (disconnectError) {
              console.log('Phantom disconnect during cleanup failed (safe to ignore):', disconnectError);
            }
            // Clear session data so user can reconnect manually
            localStorage.removeItem('walletPublickey');
            localStorage.removeItem('walletType');
            localStorage.removeItem('lastAuthTime');
            localStorage.removeItem('hasSeenHUD');
            setWalletConnected(false);
            setPublicKey(null);
            return;
          }
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
    
    // Debounce rapid successive calls (prevent within 1 second)
    const now = Date.now();
    if (now - lastConnectionAttempt.current < 1000) {
      console.log('Connection attempt too soon, debouncing...');
      setInfoMessage('⏱️ Please wait before trying again.');
      return;
    }
    lastConnectionAttempt.current = now;
    
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
    console.log('=== PHANTOM WALLET CONNECTION START (direct adapter) ===');

    try {
      // Import and create PhantomWalletAdapter directly (bypass provider)
      const { PhantomWalletAdapter } = await import('@solana/wallet-adapter-wallets');
      const adapter = new PhantomWalletAdapter();

      console.log('Created new PhantomWalletAdapter');
      console.log('Adapter ready state:', adapter.readyState);
      setInfoMessage('Connecting to Phantom...');

      // Connect directly
      console.log('Calling adapter.connect()...');
      await adapter.connect();

      if (!adapter.publicKey) {
        throw new Error('Failed to get public key from Phantom');
      }

      const walletAddress = adapter.publicKey.toBase58();
      console.log('✅ Connected to Phantom via adapter:', walletAddress);

      // Now do the backend authentication
      await processPhantomConnectionWithAdapterDirect(walletAddress, adapter);

    } catch (error: any) {
      console.error('Phantom connection failed:', error);

      const errorMsg = error.message || error.name || '';

      if (errorMsg.includes('User rejected')) {
        setInfoMessage('Connection cancelled. Please approve the connection in Phantom.');
      } else if (errorMsg.includes('Unexpected error') || errorMsg.includes('-32603')) {
        // Phantom service worker issue - provide helpful triage steps
        setErrorMessage(
          'Phantom wallet is not responding. This is usually a browser issue. Try these steps:\n\n' +
          '1. Quit your browser completely and reopen it\n' +
          '2. If that doesn\'t work, disable and re-enable the Phantom extension\n' +
          '3. As a last resort, clear Phantom\'s cache in its settings'
        );
      } else if (errorMsg.includes('WalletNotReady')) {
        setInfoMessage('Phantom wallet not found. Please install the Phantom browser extension.');
        window.open('https://phantom.app/', '_blank');
      } else {
        setErrorMessage(`Phantom connection failed: ${errorMsg || 'Unknown error'}`);
      }

      throw error;
    }
  };

  const processPhantomConnectionWithAdapterDirect = async (walletAddress: string, adapter: any) => {
    // Create challenge via Erebus
    console.log('Creating authentication challenge via Erebus...');
    const challengeResponse = await createWalletChallenge(walletAddress, 'phantom');

    // Sign the challenge message using the adapter directly
    console.log('Requesting signature for challenge...');
    const message = new TextEncoder().encode(challengeResponse.message);
    const signature = await adapter.signMessage(message);

    // Convert signature to string format expected by Erebus
    const signatureStr = Array.from(signature).toString();

    // Verify signature via Erebus
    console.log('Verifying signature via Erebus...');
    const verifyResponse = await verifyWalletSignature(
      challengeResponse.challenge_id,
      signatureStr,
      walletAddress
    );

    if (!verifyResponse.success) {
      throw new Error(`Signature verification failed: ${verifyResponse.message}`);
    }

    console.log('✅ Wallet verified successfully');

    // Update local state - backend handles user creation during verification
    setPublicKey(walletAddress);
    setWalletConnected(true);
    setShowWalletModal(false);
    setInfoMessage('');

    // Store session info
    localStorage.setItem('walletPublickey', walletAddress);
    localStorage.setItem('walletType', 'phantom');
    localStorage.setItem('hasSeenHUD', 'true');
    if (verifyResponse.session_token) {
      localStorage.setItem('sessionToken', verifyResponse.session_token);
    }
    updateAuthTime();

    console.log('✅ Wallet connected and authenticated!');
  };

  const connectMetaMaskWallet = async () => {
    // Prevent multiple simultaneous requests
    if (isMetaMaskPending || isConnecting) {
      console.log('MetaMask request already pending, skipping...');
      setInfoMessage('🔄 Connecting... Check MetaMask extension.');
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
        setInfoMessage('🚫 MetaMask not found. Install extension and refresh page.');
        window.open('https://metamask.io/', '_blank');
        throw new Error('MetaMask not installed');
      }

      console.log('About to request accounts...');
      console.log('MetaMask provider found:', metamaskProvider);
      console.log('MetaMask detected, requesting account access...');
      
      // First check if accounts are already available
      let accounts;
      try {
        console.log('Checking for existing accounts...');
        accounts = await metamaskProvider.request({ method: 'eth_accounts' });
        console.log('Existing accounts found:', accounts);
      } catch (error) {
        console.log('No existing accounts, will request access');
        accounts = [];
      }
      
      // If no accounts, request access
      if (accounts.length === 0) {
        console.log('About to request accounts from MetaMask...');
        
        // Show clear instructions for MetaMask popup
        setInfoMessage('🦊 Check MetaMask extension to approve connection.');
        
        try {
          accounts = await metamaskProvider.request({ 
            method: 'eth_requestAccounts' 
          });
        } catch (requestError: any) {
          // If there's already a pending request, try to wait and check again
          if (requestError.code === -32002) {
            console.log('Pending request detected, waiting...');
            setInfoMessage('⚠️ MetaMask busy. Complete pending requests and try again.');
            
            // Wait a bit and try to get existing accounts
            await new Promise(resolve => setTimeout(resolve, 2000));
            accounts = await metamaskProvider.request({ method: 'eth_accounts' });
            
            if (accounts.length === 0) {
              throw new Error('Connection timed out. Check MetaMask is unlocked and try again.');
            }
          } else {
            throw requestError;
          }
        }
      }

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
      
      // Show clear instructions for signature request
      setInfoMessage('✍️ Sign message in MetaMask to complete login.');
      
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
        console.log('🎯 REACHED USER REGISTRATION SECTION (MetaMask)');
        
        // Register user with Erebus after successful wallet connection
        try {
          console.log('👤 Registering user with Erebus...');
          console.log('📝 Wallet address:', walletAddress);
          console.log('📝 Wallet chain: ethereum');
          console.log('📝 About to import task service...');
          const { taskService } = await import('../../common/services/task-service');
          console.log('📝 Task service imported successfully');
          taskService.setWalletContext(walletAddress, 'ethereum');
          console.log('🔗 Task service wallet context set');
          console.log('📝 About to call registerUser...');
          const registrationResult = await taskService.registerUser(walletAddress, 'ethereum');
          console.log('📤 Registration result:', registrationResult);
          if (registrationResult.success) {
            console.log('✅ User registered successfully:', registrationResult.data);

            setInfoMessage('✅ Account registered successfully! Loading profile...');

            localStorage.removeItem('userProfile');
            localStorage.removeItem('userProfileTimestamp');

            window.dispatchEvent(new CustomEvent('user-registered', {
              detail: {
                walletAddress,
                network: 'ethereum'
              }
            }));
          } else {
            console.warn('⚠️ User registration failed:', registrationResult.error);
          }
        } catch (err) {
          console.warn('⚠️ User registration error:', err);
          console.error('❌ Full error details:', err);

          const errorMsg = err instanceof Error ? err.message : 'Unknown error';
          setErrorMessage(`Account registration failed: ${errorMsg}. You can continue, but some features may be limited.`);
        }
      } else {
        throw new Error(`Authentication failed: ${verifyResponse.message}`);
      }
    } catch (error: any) {
      console.error('MetaMask connection failed:', error);
      
      if (error.code === 4001) {
        setInfoMessage('❌ Connection cancelled. Try again and approve in MetaMask.');
        throw new Error('User rejected connection');
      } else if (error.code === -32002) {
        setInfoMessage('⚠️ MetaMask busy. Complete pending actions and retry.');
        throw new Error('Request pending');
      } else if (error.code === -32603) {
        setErrorMessage('💥 MetaMask error. Refresh page and ensure MetaMask is unlocked.');
        throw new Error('Internal error');
      } else {
        setErrorMessage(`❌ Connection failed: ${error.message}`);
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
        setInfoMessage('🔍 No wallets found. Install MetaMask or Phantom and refresh.');
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
      setInfoMessage('⚠️ Wallet detection failed. Refresh and check extensions.');
      setShowWalletModal(true);
    }
  };

  // Get the current animation phase (use login or pre-login based on auth state)
  const currentAnimationPhase = publicKey ? loginAnimationPhase : preLoginAnimationPhase;

  // Determine animation phase CSS class
  const getAnimationClass = () => {
    switch (currentAnimationPhase) {
      case 'black':
        return styles.animPhaseBlack;
      case 'stars':
        return styles.animPhaseStars;
      case 'background':
        return styles.animPhaseBackground;
      case 'navbar':
        return styles.animPhaseNavbar;
      case 'complete':
        return styles.animPhaseComplete;
      default:
        return '';
    }
  };

  // Determine if we should hide the loading overlay
  const hideOverlay = isInitialized && currentAnimationPhase !== 'black';

  return (
    <div
      className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]} ${isInitialized ? styles.initialized : ''} ${getAnimationClass()}`}
    >
      {/* Loading overlay - always rendered, hidden via CSS when ready */}
      <div className={`${styles.loadingOverlay} ${hideOverlay ? styles.overlayHidden : ''}`} />

      <div
        className={`${styles.backgroundImage} ${publicKey ? styles.loggedIn : styles.loggedOut} ${
          // Show background only after animation completes
          (isInitialized && currentAnimationPhase === 'complete') ? styles.bgReady : ''
        }`}
        style={{
          // Set background-image via inline style to avoid SSR hydration mismatch
          backgroundImage: isInitialized
            ? `url('${publicKey ? '/bg_without_logo.png' : '/bg_with_logo.png'}')`
            : 'none'
        }}
      />
      {/* Void Experience - all activity visible, interactive only when logged in */}
      <VoidExperience
        publicKey={publicKey}
        theme={currentTheme}
        loginAnimationPhase={currentAnimationPhase}
        isLoggedIn={!!publicKey && currentAnimationPhase === 'complete'}
        onTabSelect={(tab) => {
          if (tab === 'hecate' || tab === 'crossroads') {
            setHudInitialTab(tab);
            setShowHUD(true);
          }
        }}
      />
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
          initialTab={hudInitialTab}
          onToggleMobileMenu={isMobileView ? () => {} : undefined}
          loginAnimationPhase={currentAnimationPhase}
          onClose={() => {
            setShowHUD(false);
          }}
          onThemeChange={(theme) => {
            if (theme === 'null' || theme === 'light' || theme === 'dark') {
              setCurrentTheme(theme);
              localStorage.setItem('currentTheme', theme);
            }
          }}
        />
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
