import React, { useState, useEffect } from 'react';
import { agentService } from '../../common/services/agent-service';
import HUD from '../../components/hud/hud';
import { VoidExperience } from '../../components/void-experience';
import type { WalletInfo } from '../../wallet-adapters';
import { useWalletAdapter, ChainType } from '../../wallet-adapters';
import styles from './index.module.scss';

// Login animation phases: black → stars → background → navbar → complete
type LoginAnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

// Session timeout constant (for initial session check only - hook manages ongoing session)
const SESSION_TIMEOUT_MS = 30 * 60 * 1000; // 30 minutes

// Helper to check if user has valid session (client-side only)
const checkExistingSession = (): { hasSession: boolean; publicKey: string | null } => {
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

// Check session synchronously at module load for initial state
const initialSession = checkExistingSession();

const Home: React.FC = () => {
  // Use the new wallet adapter hook for all wallet operations
  const {
    isConnecting,
    error: walletError,
    connectedWallet: _connectedWallet,
    connectedAddress,
    connectedChain: _connectedChain,
    sessionToken: _sessionToken,
    connect,
    disconnect,
    clearError,
    getInstalledWallets,
    getAllWalletsInfo,
  } = useWalletAdapter();

  // Local UI state
  const [showHUD, setShowHUD] = useState<boolean>(true);
  const [currentTheme, setCurrentTheme] = useState<'null' | 'light' | 'dark'>('null');
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [showWalletModal, setShowWalletModal] = useState<boolean>(false);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [messageType, setMessageType] = useState<'error' | 'info'>('error');
  const lastConnectionAttempt = React.useRef<number>(0);
  const [hudInitialTab, setHudInitialTab] = useState<
    'crossroads' | 'memcache' | 'tasks' | 'agents' | 'logs' | 'canvas' | null
  >(initialSession.hasSession ? 'memcache' : null);
  const [isMobileView, setIsMobileView] = useState<boolean>(false);

  // Shared Hecate panel state
  const [hecatePanelOpen, setHecatePanelOpen] = useState<boolean>(false);

  // Track active HUD tab for overlapping panel detection
  const [activeHudTab, setActiveHudTab] = useState<
    'crossroads' | 'memcache' | 'tasks' | 'agents' | 'logs' | 'canvas' | null
  >(initialSession.hasSession ? 'memcache' : null);

  // Crossroads orb ring alignment state
  const [triggerOrbAlignment, setTriggerOrbAlignment] = useState<boolean>(false);
  const [pendingCrossroadsTransition, setPendingCrossroadsTransition] = useState<boolean>(false);

  // Login animation state
  const [loginAnimationPhase, setLoginAnimationPhase] = useState<LoginAnimationPhase>(
    initialSession.hasSession ? 'black' : 'idle',
  );
  const [preLoginAnimationPhase, setPreLoginAnimationPhase] = useState<LoginAnimationPhase>(
    !initialSession.hasSession ? 'black' : 'idle',
  );
  const previousPublicKey = React.useRef<string | null>(initialSession.publicKey);
  const animationTriggered = React.useRef<boolean>(false);
  const preLoginAnimationTriggered = React.useRef<boolean>(false);
  const hasEverConnected = React.useRef<boolean>(false);
  const [hasLoggedOut, setHasLoggedOut] = useState<boolean>(false);

  const [systemStatus, setSystemStatus] = useState({
    hud: false,
    mcp: false,
    orchestration: false,
    agents: false,
    portfolio: false,
    defi: false,
    social: false,
    arbitrage: false,
    hecate: true,
    erebus: true,
  });

  // Sync wallet error to local error state
  useEffect(() => {
    if (walletError) {
      setConnectionError(walletError);
      setMessageType('error');
    }
  }, [walletError]);

  // Initialize theme
  useEffect(() => {
    const savedTheme = localStorage.getItem('currentTheme');

    if (savedTheme && (savedTheme === 'null' || savedTheme === 'light' || savedTheme === 'dark')) {
      setCurrentTheme(savedTheme as 'null' | 'light' | 'dark');
    } else {
      setCurrentTheme('null');
      localStorage.setItem('currentTheme', 'null');
    }
  }, []);

  // Detect mobile view
  useEffect(() => {
    const checkMobileView = () => {
      setIsMobileView(window.innerWidth <= 768);
    };

    checkMobileView();
    window.addEventListener('resize', checkMobileView);

    return () => window.removeEventListener('resize', checkMobileView);
  }, []);

  // Start animation for returning users on mount
  useEffect(() => {
    if (initialSession.hasSession && !animationTriggered.current) {
      console.log('🎬 Immediate animation start for returning user');
      animationTriggered.current = true;

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
  }, []);

  // Pre-login animation for new visitors
  useEffect(() => {
    if (!initialSession.hasSession && !preLoginAnimationTriggered.current) {
      console.log('🎬 Pre-login animation start for new visitor');
      preLoginAnimationTriggered.current = true;

      setTimeout(() => {
        console.log('🌟 [Pre-login] Stars fading in...');
        setPreLoginAnimationPhase('stars');
      }, 200);

      setTimeout(() => {
        console.log('🌌 [Pre-login] Background + Mission text fading in...');
        setPreLoginAnimationPhase('background');
      }, 800);

      setTimeout(() => {
        console.log('⚡ [Pre-login] Navbar + CTA flickering in...');
        setPreLoginAnimationPhase('navbar');
      }, 1500);

      setTimeout(() => {
        console.log('✅ [Pre-login] Animation complete');
        setPreLoginAnimationPhase('complete');
      }, 2200);
    }
  }, []);

  // Login animation sequence - triggers on fresh login
  useEffect(() => {
    const isNewLogin = previousPublicKey.current === null && connectedAddress !== null;

    previousPublicKey.current = connectedAddress;

    if (isNewLogin && !animationTriggered.current) {
      animationTriggered.current = true;
      console.log('🎬 Starting login animation for new login...');

      setLoginAnimationPhase('background');

      setTimeout(() => {
        console.log('✅ Login animation complete');
        setLoginAnimationPhase('complete');
      }, 1500);
    }

    // Track when wallet actually connects (not just session restoration)
    if (connectedAddress !== null) {
      hasEverConnected.current = true;
    }

    // Reset animation when logging out (only if we've actually connected during this session)
    const isActualLogout =
      connectedAddress === null && loginAnimationPhase !== 'idle' && hasEverConnected.current;

    if (isActualLogout) {
      setLoginAnimationPhase('idle');
      animationTriggered.current = false;
      setHasLoggedOut(true);

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
  }, [connectedAddress, loginAnimationPhase]);

  // Initialize system on mount
  useEffect(() => {
    if (!initialSession.hasSession) {
      localStorage.removeItem('walletPublickey');
      localStorage.removeItem('walletType');
      localStorage.removeItem('lastAuthTime');
      localStorage.removeItem('hasSeenHUD');
    }

    setTimeout(() => {
      setIsInitialized(true);
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

  // Message helpers
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
    clearError();
  };

  // Handle wallet connection using the adapter hook
  const handleConnectWallet = async (walletId?: string, chain?: ChainType) => {
    console.log('=== handleConnectWallet START ===');
    console.log('handleConnectWallet called with walletId:', walletId, 'chain:', chain);

    // Debounce rapid successive calls
    const now = Date.now();

    if (now - lastConnectionAttempt.current < 1000) {
      console.log('Connection attempt too soon, debouncing...');
      setInfoMessage('⏱️ Please wait before trying again.');

      return;
    }

    lastConnectionAttempt.current = now;

    // If no wallet specified, show selection modal
    if (!walletId) {
      console.log('No walletId specified, showing selection modal');
      setShowWalletModal(true);

      return;
    }

    clearMessage();

    try {
      console.log('Connecting to wallet:', walletId);
      const result = await connect(walletId, chain);

      if (result.success) {
        console.log('✅ Wallet connected successfully:', result.address);
        setShowWalletModal(false);
        setHudInitialTab('memcache');

        // Register user with task service
        try {
          const { taskService } = await import('../../common/services/task-service');
          const network = result.chain === ChainType.SOLANA ? 'solana' : 'ethereum';

          taskService.setWalletContext(result.address!, network);

          const registrationResult = await taskService.registerUser(result.address!, network);

          if (registrationResult.success) {
            console.log('✅ User registered successfully:', registrationResult.data);
            setInfoMessage('✅ Account registered successfully!');

            localStorage.removeItem('userProfile');
            localStorage.removeItem('userProfileTimestamp');

            window.dispatchEvent(
              new CustomEvent('user-registered', {
                detail: {
                  walletAddress: result.address,
                  network,
                },
              }),
            );
          } else {
            console.warn('⚠️ User registration failed:', registrationResult.error);
          }
        } catch (err) {
          console.warn('⚠️ User registration error:', err);
        }
      } else {
        console.error('❌ Connection failed:', result.error);
        handleConnectionError(result.error || 'Connection failed', walletId);
      }
    } catch (error: any) {
      console.error('Connection error:', error);
      handleConnectionError(error.message || 'Unknown error', walletId);
    }

    console.log('=== handleConnectWallet END ===');
  };

  // Format connection errors with helpful messages
  const handleConnectionError = (errorMsg: string, walletId: string) => {
    if (errorMsg.includes('User rejected') || errorMsg.includes('cancelled')) {
      setInfoMessage('❌ Connection cancelled. Please approve the connection.');
    } else if (errorMsg.includes('not installed')) {
      const wallet = getAllWalletsInfo().find((w) => w.id === walletId);

      if (wallet?.installUrl) {
        setInfoMessage(`🚫 ${wallet.name} not found. Install extension and refresh.`);
        window.open(wallet.installUrl, '_blank');
      } else {
        setInfoMessage(`🚫 Wallet not found. Install extension and refresh.`);
      }
    } else if (errorMsg.includes('-32002') || errorMsg.includes('pending')) {
      setInfoMessage('⚠️ Wallet busy. Complete pending requests and try again.');
    } else if (errorMsg.includes('-32603') || errorMsg.includes('Unexpected error')) {
      setErrorMessage(
        'Wallet is not responding. Try these steps:\n\n' +
          '1. Quit your browser completely and reopen it\n' +
          "2. If that doesn't work, disable and re-enable the extension\n" +
          "3. As a last resort, clear the wallet's cache in its settings",
      );
    } else {
      setErrorMessage(`❌ Connection failed: ${errorMsg}`);
    }
  };

  // Handle disconnect using the adapter hook
  const handleDisconnect = async () => {
    try {
      // Clear backend conversation history before disconnecting
      try {
        await agentService.clearConversation('hecate');
      } catch (clearError) {
        // Don't block disconnect if clear fails
        console.warn('Failed to clear conversation:', clearError);
      }

      await disconnect();
      setHudInitialTab(null);
    } catch (error) {
      console.error('Disconnect error:', error);
    }
  };

  // Handle "Enter the Crossroads" button - triggers orb ring alignment
  const handleEnterCrossroads = () => {
    if (activeHudTab === 'crossroads') {
      setPendingCrossroadsTransition(false);
      setTriggerOrbAlignment(false);
      return;
    }
    console.log('🔮 Triggering orb alignment for Crossroads transition');
    setPendingCrossroadsTransition(true);
    setTriggerOrbAlignment(true);
  };

  // Handle orb alignment completion - now show Crossroads UI
  const handleAlignmentComplete = () => {
    console.log('✨ Orb alignment complete, showing Crossroads');
    setTriggerOrbAlignment(false);
    if (pendingCrossroadsTransition) {
      setPendingCrossroadsTransition(false);
      setHudInitialTab('crossroads');
    }
  };

  // Get current animation phase
  // Use initialSession.hasSession for returning users before connectedAddress is restored by hook
  // After logout, use pre-login animation even if initialSession.hasSession was true
  const isReturningUser = !hasLoggedOut && (initialSession.hasSession || Boolean(connectedAddress));
  const currentAnimationPhase = isReturningUser ? loginAnimationPhase : preLoginAnimationPhase;

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

  const hideOverlay = isInitialized && currentAnimationPhase !== 'black';

  // Get wallet icon (fallback to emoji for known wallets)
  const getWalletIcon = (wallet: WalletInfo): React.ReactNode => {
    if (wallet.icon && !wallet.icon.startsWith('http')) {
      return wallet.icon;
    }

    // Fallback icons
    switch (wallet.id) {
      case 'phantom':
        return '👻';
      case 'metamask':
        return '🦊';
      case 'bitget':
        return (
          <img
            src="/wallets/bitget_logo.png"
            alt="Bitget"
            style={{ width: '2.5rem', height: '2.5rem', objectFit: 'contain' }}
          />
        );
      default:
        return '👛';
    }
  };

  // Get chain description for wallet
  const getChainDescription = (wallet: WalletInfo) => {
    const chains = wallet.supportedChains;

    if (chains.includes(ChainType.EVM) && chains.includes(ChainType.SOLANA)) {
      return 'EVM & Solana';
    } else if (chains.includes(ChainType.EVM)) {
      return 'Ethereum & EVM';
    } else if (chains.includes(ChainType.SOLANA)) {
      return 'Solana';
    }

    return 'Multi-chain';
  };

  return (
    <div
      className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]} ${isInitialized ? styles.initialized : ''} ${getAnimationClass()}`}
    >
      {/* Loading overlay */}
      <div className={`${styles.loadingOverlay} ${hideOverlay ? styles.overlayHidden : ''}`} />

      <div
        className={`${styles.backgroundImage} ${connectedAddress ? styles.loggedIn : styles.loggedOut} ${
          isInitialized && currentAnimationPhase === 'complete' ? styles.bgReady : ''
        }`}
        style={{
          backgroundImage: isInitialized
            ? `url('${connectedAddress ? '/bg_without_logo.png' : '/bg_with_logo.png'}')`
            : 'none',
        }}
      />

      {/* Void Experience */}
      <VoidExperience
        publicKey={connectedAddress || initialSession.publicKey}
        theme={currentTheme}
        loginAnimationPhase={currentAnimationPhase}
        isLoggedIn={isReturningUser && currentAnimationPhase === 'complete'}
        hecatePanelOpen={hecatePanelOpen}
        onHecatePanelChange={setHecatePanelOpen}
        hasOverlappingPanels={activeHudTab === 'memcache' || activeHudTab === 'crossroads'}
        triggerAlignment={triggerOrbAlignment}
        onAlignmentComplete={handleAlignmentComplete}
        keepAligned={activeHudTab === 'crossroads'}
      />

      <div className={`${styles.scene} ${showHUD ? styles.hudActive : ''}`} />

      {showHUD &&
        isInitialized &&
        (currentAnimationPhase === 'navbar' || currentAnimationPhase === 'complete') && (
          <HUD
            publicKey={connectedAddress || initialSession.publicKey}
            onDisconnect={handleDisconnect}
            onConnectWallet={(walletType?: 'phantom' | 'metamask') => {
              // Map legacy wallet type names to adapter IDs
              handleConnectWallet(walletType);
            }}
            theme={currentTheme}
            systemStatus={systemStatus}
            initialTab={hudInitialTab}
            onToggleMobileMenu={isMobileView ? () => {} : undefined}
            loginAnimationPhase={currentAnimationPhase}
            onClose={() => setShowHUD(false)}
            onThemeChange={(theme) => {
              if (theme === 'null' || theme === 'light' || theme === 'dark') {
                setCurrentTheme(theme);
                localStorage.setItem('currentTheme', theme);
              }
            }}
            hecatePanelOpen={hecatePanelOpen}
            onHecatePanelChange={setHecatePanelOpen}
            onActiveTabChange={setActiveHudTab}
            onEnterCrossroads={handleEnterCrossroads}
            pendingCrossroadsTransition={pendingCrossroadsTransition}
          />
        )}

      {/* Wallet Selection Modal - Now Dynamic */}
      {showWalletModal && (
        <div className={styles.modalOverlay} onClick={() => setShowWalletModal(false)}>
          <div className={styles.walletModal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2>🔐 Connect Wallet</h2>
              <button className={styles.closeButton} onClick={() => setShowWalletModal(false)}>
                ×
              </button>
            </div>

            <div className={styles.modalContent}>
              <p>Choose a Web3 wallet to connect to Nullblock:</p>

              <div className={styles.walletOptions}>
                {/* Installed wallets - show first */}
                {getInstalledWallets().map((adapter) => (
                  <button
                    key={adapter.id}
                    className={styles.walletButton}
                    onClick={() => handleConnectWallet(adapter.id)}
                    disabled={isConnecting}
                  >
                    <div className={styles.walletIcon}>{getWalletIcon(adapter.info)}</div>
                    <div className={styles.walletInfo}>
                      <div className={styles.walletName}>{adapter.info.name}</div>
                      <div className={styles.walletDescription}>
                        {getChainDescription(adapter.info)}
                      </div>
                    </div>
                    {isConnecting && <div className={styles.connecting}>Connecting...</div>}
                  </button>
                ))}

                {/* Not installed wallets - show with install prompt */}
                {getAllWalletsInfo()
                  .filter((info) => !getInstalledWallets().some((w) => w.id === info.id))
                  .map((wallet) => (
                    <button
                      key={wallet.id}
                      className={`${styles.walletButton} ${styles.notInstalled}`}
                      onClick={() => {
                        if (wallet.installUrl) {
                          window.open(wallet.installUrl, '_blank');
                        }
                      }}
                      disabled={isConnecting}
                    >
                      <div className={styles.walletIcon}>{getWalletIcon(wallet)}</div>
                      <div className={styles.walletInfo}>
                        <div className={styles.walletName}>{wallet.name}</div>
                        <div className={styles.walletDescription}>
                          {getChainDescription(wallet)} • Click to install
                        </div>
                      </div>
                    </button>
                  ))}
              </div>

              {connectionError && (
                <div className={messageType === 'error' ? styles.errorMessage : styles.infoMessage}>
                  {connectionError}
                </div>
              )}

              <div className={styles.installPrompt}>
                <p>Don't have a wallet?</p>
                <div className={styles.installLinks}>
                  {getAllWalletsInfo()
                    .filter((w) => w.installUrl)
                    .map((wallet) => (
                      <a
                        key={wallet.id}
                        href={wallet.installUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        Install {wallet.name}
                      </a>
                    ))}
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
