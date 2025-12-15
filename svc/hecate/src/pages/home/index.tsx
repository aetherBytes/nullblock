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

      // Phase 1: Start with black screen
      setLoginAnimationPhase('black');

      // Phase 2: Stars and background fade in together (after 400ms)
      setTimeout(() => {
        console.log('🌟🌌 Stars + Background fading in together...');
        setLoginAnimationPhase('background');
      }, 400);

      // Phase 3: Navbar flickers in (after 2500ms)
      setTimeout(() => {
        console.log('⚡ Navbar flickering in...');
        setLoginAnimationPhase('navbar');
      }, 2500);

      // Phase 4: Animation complete (after 4000ms)
      setTimeout(() => {
        console.log('✅ Login animation complete');
        setLoginAnimationPhase('complete');
      }, 4000);
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

    if (!provider.isPhantom) {
      setInfoMessage('Invalid Phantom provider detected. Please refresh the page.');
      throw new Error('Invalid Phantom provider');
    }

    // Give Phantom more time to fully initialize (increased from 100ms to 500ms)
    // This helps prevent -32603 errors from premature connection attempts
    console.log('⏳ Waiting for Phantom to fully initialize...');
    await new Promise(resolve => setTimeout(resolve, 500));

    // Verify Phantom is ready
    if (!provider.isPhantom) {
      throw new Error('Phantom provider not properly initialized');
    }

    try {
      console.log('Phantom detected, checking status...');
      console.log('Provider info:', {
        isPhantom: provider.isPhantom,
        isConnected: provider.isConnected,
        publicKey: provider.publicKey?.toString(),
      });

      // If already connected, use existing connection
      if (provider.isConnected && provider.publicKey) {
        console.log('Phantom is already connected, using existing connection');
        const walletAddress = provider.publicKey.toString();
        console.log('Using existing Phantom connection:', walletAddress);

        // Skip signature and go straight to backend verification
        await processPhantomConnection(walletAddress, provider);
        return;
      }

      // First connect to get public key
      let response;
      try {
        console.log('Requesting new Phantom connection...');

        // Proactively disconnect to clear any stale internal state
        // This prevents -32603 errors from previous failed connection attempts
        try {
          console.log('Disconnecting any existing Phantom connection to ensure clean state...');
          await provider.disconnect();
          console.log('Phantom disconnected successfully');
        } catch (disconnectError) {
          console.log('Phantom disconnect before connection failed (safe to ignore):', disconnectError);
        }

        // For error code -32603 (Internal error), do a simple connect without options
        // This error often happens when provider isn't fully ready or has internal state issues
        console.log('Attempting direct connection (no options)...');

        let retryCount = 0;
        const maxRetries = 2;

        while (retryCount <= maxRetries) {
          try {
            response = await provider.connect();
            console.log('Phantom connection response:', response);
            break;
          } catch (retryError: any) {
            if (retryError?.code === -32603 && retryCount < maxRetries) {
              retryCount++;
              console.log(`⚠️ Phantom -32603 error, retrying (attempt ${retryCount}/${maxRetries})...`);
              setInfoMessage(`⏳ Phantom initializing, retrying (${retryCount}/${maxRetries})...`);
              // Increase delay between retries from 500ms to 1000ms
              await new Promise(resolve => setTimeout(resolve, 1000));
            } else {
              throw retryError;
            }
          }
        }

        if (!response) {
          throw new Error('Failed to connect after retries');
        }
      } catch (connectError: any) {
        console.error('Phantom connect() error:', connectError);
        console.error('Error type:', typeof connectError);
        console.error('Error properties:', Object.keys(connectError));
        console.error('Error code:', connectError?.code);
        console.error('Error data:', connectError?.data);
        console.error('Error message:', connectError?.message);

        // Handle common Phantom errors
        if (connectError?.message?.includes('User rejected') || connectError?.message?.includes('rejected')) {
          setInfoMessage('Connection cancelled. Please approve the connection in Phantom wallet.');
          throw new Error('User rejected connection');
        } else if (connectError?.code === 4001) {
          setInfoMessage('Connection cancelled. Please approve the connection in Phantom wallet.');
          throw new Error('User rejected connection');
        } else if (connectError?.code === -32002) {
          setInfoMessage('Phantom is busy processing another request. Please wait and try again.');
          throw new Error('Request pending');
        } else if (connectError?.code === -32603) {
          // Internal error from Phantom - usually a timing or state issue
          setErrorMessage('⚠️ Phantom connection failed after automatic retries.\n\nQuick fixes (try in order):\n\n1. 🔓 Make sure Phantom is UNLOCKED (click extension)\n2. 🔄 REFRESH this page and try again\n3. 🔌 Disconnect & reconnect:\n   • Open Phantom → Settings → Trusted Apps\n   • Remove this site if listed\n   • Refresh page and connect again\n4. 🔄 Restart your browser if still failing\n\nThis is usually a temporary Phantom state issue.');
          throw new Error('Phantom internal error (-32603)');
        } else if (connectError?.message?.includes('locked')) {
          setInfoMessage('Phantom wallet is locked. Please unlock it and try again.');
          throw new Error('Wallet locked');
        } else if (connectError?.message === 'Unexpected error' || connectError?.message?.includes('Unexpected error')) {
          // Check if error.data has more details
          const errorDetails = connectError?.data ? JSON.stringify(connectError.data) : 'none';
          console.error('Unexpected error details from data:', errorDetails);

          // This is the generic error - provide helpful troubleshooting
          let troubleshootingMsg = 'Cannot connect to Phantom. Troubleshooting steps:\n\n';
          troubleshootingMsg += '1. Make sure Phantom is UNLOCKED (click the extension)\n';
          troubleshootingMsg += '2. Ensure you have at least ONE Solana account in Phantom\n';
          troubleshootingMsg += '3. Try REFRESHING this page\n';
          troubleshootingMsg += '4. If still failing, try RESTARTING your browser\n';
          troubleshootingMsg += '5. Check if Phantom extension needs an update';

          setErrorMessage(troubleshootingMsg);
          throw new Error('Phantom unexpected error');
        } else {
          const errorMsg = connectError?.message || connectError?.toString() || 'Unknown error';
          setErrorMessage(`Failed to connect to Phantom: ${errorMsg}. Please make sure Phantom is unlocked and has at least one account.`);
          throw connectError;
        }
      }

      if (!response.publicKey) {
        throw new Error('Failed to get public key from Phantom');
      }

      const walletAddress = response.publicKey.toString();
      console.log('Connected to Phantom wallet:', walletAddress);

      // Process the connection
      await processPhantomConnection(walletAddress, provider);
    } catch (error: any) {
      console.error('Phantom connection failed:', error);
      console.error('Error details:', {
        message: error.message,
        code: error.code,
        name: error.name,
        stack: error.stack,
        fullError: error
      });

      if (error.code === 4001 || error.message?.includes('User rejected')) {
        setInfoMessage('Connection cancelled by user.');
        throw new Error('User rejected connection');
      } else if (error.code === -32002) {
        setInfoMessage('Phantom is processing another request. Please check your extension or wait a moment and try again.');
        throw new Error('Request pending');
      } else if (error.message?.includes('Wallet locked')) {
        throw error;
      } else if (error.message === 'Phantom not installed' || error.message === 'Phantom not initialized') {
        throw error;
      } else {
        const errorMsg = error.message || error.toString() || 'Unknown error';
        console.error('Unhandled Phantom error:', errorMsg);

        if (errorMsg.length < 5 || errorMsg.match(/^[A-Z][a-z]:/)) {
          setErrorMessage('Phantom wallet connection failed. Please make sure Phantom is unlocked, refresh the page, and try again.');
        } else {
          setErrorMessage(`Phantom connection failed: ${errorMsg}`);
        }
        throw error;
      }
    }
  };

  const processPhantomConnection = async (walletAddress: string, provider: any) => {
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
        console.log('🎯 REACHED USER REGISTRATION SECTION');
        
        // Register user with Erebus after successful wallet connection
        try {
          console.log('👤 Registering user with Erebus...');
          console.log('📝 Wallet address:', walletAddress);
          console.log('📝 Wallet chain: solana');
          console.log('📝 About to import task service...');
          const { taskService } = await import('../../common/services/task-service');
          console.log('📝 Task service imported successfully');
          taskService.setWalletContext(walletAddress, 'solana');
          console.log('🔗 Task service wallet context set');
          console.log('📝 About to call registerUser...');
          const registrationResult = await taskService.registerUser(walletAddress, 'solana');
          console.log('📤 Registration result:', registrationResult);
          if (registrationResult.success) {
            console.log('✅ User registered successfully:', registrationResult.data);

            // Show success message to user
            setInfoMessage('✅ Account registered successfully! Loading profile...');

            // Invalidate profile cache
            localStorage.removeItem('userProfile');
            localStorage.removeItem('userProfileTimestamp');

            // Dispatch event to trigger profile refresh
            window.dispatchEvent(new CustomEvent('user-registered', {
              detail: {
                walletAddress,
                network: 'solana'
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
      <StarsCanvas theme={currentTheme === 'light' ? 'light' : (currentTheme === 'null' ? 'null' : 'null')} loggedIn={!!publicKey} />
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
