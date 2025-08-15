import React, { useState, useEffect, useRef } from 'react';
import styles from './hud.module.scss';
import { fetchWalletData, fetchUserProfile, fetchAscentLevel, fetchActiveMission, MissionData } from '../../common/services/api';
import { isAuthenticated, restoreSession, createAuthChallenge, verifyAuthChallenge, checkMCPHealth } from '../../common/services/mcp-api';
// Removed separate dashboard imports - all functionality is now integrated into HUD tabs
import xLogo from '../../assets/images/X_logo_black.png';
import nulleyeLogo from '../../assets/images/nulleye_logo.png';
import ContextDashboard from '../context-dashboard';

type Screen = 'home' | 'overview' | 'camp' | 'inventory' | 'campaign' | 'lab';
type Theme = 'null' | 'light' | 'dark';
type TabType = 'missions' | 'systems' | 'defense' | 'uplink' | 'hud' | 'status' | 'arbitrage' | 'social' | 'portfolio' | 'defi';

interface SystemStatus {
  hud: boolean;
  mcp: boolean;
  orchestration: boolean;
  agents: boolean;
  portfolio: boolean;
  defi: boolean;
  social: boolean;
  arbitrage: boolean;
  hecate: boolean;
  erebus: boolean;
}

interface HUDProps {
  publicKey: string | null;
  onDisconnect: () => void;
  onConnectWallet: () => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'cyber' | 'light' | 'dark') => void;
  systemStatus: SystemStatus;
}

interface UserProfile {
  id: string;
  ascent: number;
  nether: number | null;
  cacheValue: number;
  memories: number;
  matrix: {
    level: string;
    rarity: string;
    status: string;
  };
}

interface AscentLevel {
  level: number;
  title: string;
  description: string;
  progress: number;
  nextLevel: number;
  nextTitle: string;
  nextDescription: string;
  accolades: string[];
}

interface SystemAnalysis {
  name: string;
  status: string;
  locked: boolean;
}

// Add Ember Link status interface
interface EmberLinkStatus {
  connected: boolean;
  lastSeen: Date | null;
  browserInfo: {
    name: string;
    version: string;
    platform: string;
  } | null;
}

// Add new interfaces for uplinks
interface Uplink {
  id: string;
  name: string;
  status: 'active' | 'inactive' | 'error';
  icon: string;
  details: {
    description: string;
    stats: Array<{
      label: string;
      value: string | number;
      status?: string;
    }>;
  };
}

// Add new interfaces for leaderboard
interface LeaderboardEntry {
  id: string;
  rank: number;
  ascent: number;
  nether: number;
  cacheValue: number;
  memories: number;
  matrix: {
    level: string;
    rarity: string;
    status: string;
  };
}

// Add AscentLevelData interface
interface AscentLevelData {
  level: number;
  name: string;
  description: string;
  progress: number;
  accolades: string[];
}

const SCREEN_LABELS: Record<Screen, string> = {
  home: '',
  overview: 'OVERVIEW',
  camp: 'CAMP',
  inventory: 'CACHE',
  campaign: 'CAMPAIGN',
  lab: 'LAB',
};

const HUD: React.FC<HUDProps> = ({ 
  publicKey, 
  onDisconnect, 
  onConnectWallet,
  theme = 'light', 
  onClose, 
  onThemeChange,
  systemStatus
}) => {
  const [screen, setScreen] = useState<Screen>('home');
  const [walletData, setWalletData] = useState<any>(null);
  const [userProfile, setUserProfile] = useState<UserProfile>({
    id: publicKey ? `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol` : '',
    ascent: 1,
    nether: null,
    cacheValue: 0,
    memories: 0,
    matrix: {
      level: 'NONE',
      rarity: 'NONE',
      status: 'N/A'
    }
  });
  const [ascentLevel, setAscentLevel] = useState<AscentLevel | null>(null);
  const [showAscentDetails, setShowAscentDetails] = useState<boolean>(false);
  const [alerts, setAlerts] = useState<number>(3);
  const [showAlerts, setShowAlerts] = useState<boolean>(false);
  const [showNectarDetails, setShowNectarDetails] = useState<boolean>(false);
  const [showCacheValueDetails, setShowCacheValueDetails] = useState<boolean>(false);
  const [showEmberConduitDetails, setShowEmberConduitDetails] = useState<boolean>(false);
  const [showMemoriesDetails, setShowMemoriesDetails] = useState<boolean>(false);
  const [activeMission, setActiveMission] = useState<MissionData | null>(null);
  const [showMissionDropdown, setShowMissionDropdown] = useState(false);
  const [showMissionBrief, setShowMissionBrief] = useState(false);
  const missionDropdownRef = useRef<HTMLDivElement>(null);
  const cardRef = useRef<HTMLDivElement>(null);
  const [activeTab, setActiveTab] = useState<TabType>('hud');
  const [emberLinkStatus, setEmberLinkStatus] = useState<EmberLinkStatus>({
    connected: false,
    lastSeen: null,
    browserInfo: null
  });
  const [isDigitizing, setIsDigitizing] = useState<boolean>(false);
  const [selectedUplink, setSelectedUplink] = useState<Uplink | null>(null);
  const [showLeaderboard, setShowLeaderboard] = useState<boolean>(false);
  // Remove separate dashboard overlays - everything will be integrated into HUD tabs
  const [mcpAuthenticated, setMcpAuthenticated] = useState<boolean>(false);
  const [mcpHealthStatus, setMcpHealthStatus] = useState<any>(null);
  const [nulleyeState, setNulleyeState] = useState<'base' | 'response' | 'question' | 'thinking' | 'alert' | 'error' | 'warning' | 'success' | 'processing'>('base');
  const [showContextDashboard, setShowContextDashboard] = useState<boolean>(false);
  const [contextDashboardActiveTab, setContextDashboardActiveTab] = useState<'tasks' | 'mcp' | 'logs' | 'agents' | 'hecate'>('hecate');


  // Only unlock 'home' and 'overview' by default, unlock others if logged in
  const unlockedScreens = publicKey ? ['home', 'overview', 'camp'] : ['home', 'overview'];

  // Define uplinks
  const uplinks: Uplink[] = [
    {
      id: 'ember',
      name: 'Ember Link',
      status: emberLinkStatus.connected ? 'active' : 'inactive',
      icon: '🔥',
      details: {
        description: 'Direct connection to the Ember network, enabling secure communication and data transfer.',
        stats: [
          {
            label: 'Connection Status',
            value: emberLinkStatus.connected ? 'Connected' : 'Disconnected',
            status: emberLinkStatus.connected ? 'active' : 'inactive'
          },
          {
            label: 'Last Seen',
            value: emberLinkStatus.lastSeen ? new Date(emberLinkStatus.lastSeen).toLocaleString() : 'Never'
          },
          {
            label: 'Browser',
            value: emberLinkStatus.browserInfo ? `${emberLinkStatus.browserInfo.name} ${emberLinkStatus.browserInfo.version}` : 'Unknown'
          },
          {
            label: 'Platform',
            value: emberLinkStatus.browserInfo?.platform || 'Unknown'
          }
        ]
      }
    },
    {
      id: 'neural',
      name: 'Neural Link',
      status: 'inactive',
      icon: '🧠',
      details: {
        description: 'Advanced neural interface for enhanced cognitive processing and system interaction.',
        stats: [
          {
            label: 'Status',
            value: 'LOCKED',
            status: 'inactive'
          },
          {
            label: 'Signal Strength',
            value: 'N/A'
          },
          {
            label: 'Latency',
            value: 'N/A'
          }
        ]
      }
    },
    {
      id: 'wallet',
      name: 'Wallet Health',
      status: 'inactive',
      icon: '💳',
      details: {
        description: 'Real-time monitoring of wallet security and transaction status.',
        stats: [
          {
            label: 'Status',
            value: 'LOCKED',
            status: 'inactive'
          },
          {
            label: 'Last Transaction',
            value: 'N/A'
          },
          {
            label: 'Security Level',
            value: 'N/A'
          }
        ]
      }
    },
    {
      id: 'token',
      name: 'Token Analysis',
      status: 'inactive',
      icon: '🪙',
      details: {
        description: 'Comprehensive analysis of token holdings and market performance.',
        stats: [
          {
            label: 'Status',
            value: 'LOCKED',
            status: 'inactive'
          },
          {
            label: 'Scan Progress',
            value: 'N/A'
          },
          {
            label: 'Last Update',
            value: 'N/A'
          }
        ]
      }
    }
  ];

  // Add mock leaderboard data
  const leaderboardData: LeaderboardEntry[] = [
    {
      id: "PervySage",
      rank: 1,
      ascent: 999,
      nether: 999999,
      cacheValue: 999999,
      memories: 999,
      matrix: {
        level: "ARCHITECT",
        rarity: "MYTHICAL",
        status: "FLAME KEEPER"
      }
    },
    {
      id: "HUD-001",
      rank: 2,
      ascent: 5,
      nether: 1500,
      cacheValue: 2500,
      memories: 12,
      matrix: {
        level: "MASTER",
        rarity: "LEGENDARY",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-002",
      rank: 3,
      ascent: 4,
      nether: 1200,
      cacheValue: 2000,
      memories: 10,
      matrix: {
        level: "EXPERT",
        rarity: "EPIC",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-003",
      rank: 4,
      ascent: 3,
      nether: 900,
      cacheValue: 1500,
      memories: 8,
      matrix: {
        level: "ADVANCED",
        rarity: "RARE",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-004",
      rank: 5,
      ascent: 2,
      nether: 600,
      cacheValue: 1000,
      memories: 6,
      matrix: {
        level: "INTERMEDIATE",
        rarity: "UNCOMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-005",
      rank: 6,
      ascent: 1,
      nether: 300,
      cacheValue: 500,
      memories: 4,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-006",
      rank: 7,
      ascent: 1,
      nether: 250,
      cacheValue: 400,
      memories: 3,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-007",
      rank: 8,
      ascent: 1,
      nether: 200,
      cacheValue: 300,
      memories: 2,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-008",
      rank: 9,
      ascent: 1,
      nether: 150,
      cacheValue: 200,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-009",
      rank: 10,
      ascent: 1,
      nether: 100,
      cacheValue: 150,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-010",
      rank: 11,
      ascent: 1,
      nether: 50,
      cacheValue: 100,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "HUD-011",
      rank: 12,
      ascent: 1,
      nether: 25,
      cacheValue: 50,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    }
  ];

  const getStatusClass = (status: string): string => {
    switch (status.toLowerCase()) {
      case 'scanning':
      case 'in progress':
      case 'checking':
        return styles.scanning;
      case 'optimal':
      case 'connected':
      case 'secure':
        return styles.active;
      case 'low':
      case 'standby':
      case 'stable':
        return styles.stable;
      default:
        return styles.inactive;
    }
  };

  const systemAnalysisItems: SystemAnalysis[] = [
    { name: "Neural Link", status: "SCANNING", locked: false },
    { name: "Wallet Health", status: "OPTIMAL", locked: false },
    { name: "Token Analysis", status: "IN PROGRESS", locked: false },
    { name: "Risk Assessment", status: "LOW", locked: true },
    { name: "Memory Integrity", status: "CHECKING", locked: true },
    { name: "Network Status", status: "CONNECTED", locked: true },
    { name: "Matrix Sync", status: "OFFLINE", locked: true },
    { name: "Reality Engine", status: "DORMANT", locked: true },
    { name: "Core Systems", status: "LOCKED", locked: true },
    { name: "Neural Cache", status: "UNAVAILABLE", locked: true },
    { name: "Quantum Resonance", status: "UNKNOWN", locked: true },
    { name: "Bio-Interface", status: "DISABLED", locked: true },
    { name: "Temporal Alignment", status: "DESYNCED", locked: true }
  ];

  const handleScreenChange = (newScreen: Screen) => {
    if (unlockedScreens.includes(newScreen)) {
      setScreen(newScreen);
    }
  };

  // MCP initialization effect
  useEffect(() => {
    const initializeMCP = async () => {
      try {
        // Try to restore existing session
        const hasSession = restoreSession();
        setMcpAuthenticated(hasSession && isAuthenticated());
        
        // Check MCP health
        const health = await checkMCPHealth();
        setMcpHealthStatus(health);
      } catch (error) {
        console.error('Failed to initialize MCP:', error);
        setMcpHealthStatus(null);
      }
    };

    initializeMCP();
  }, []);

  useEffect(() => {
    const loadWalletData = async () => {
      if (publicKey) {
        try {
          // Fetch basic wallet data
          const data = await fetchWalletData(publicKey);
          setWalletData(data);
          
          // Fetch user profile data including username if available
          try {
            const profileData = await fetchUserProfile(publicKey);
            
            // Check if the wallet has Nectar tokens
            const hasNectarToken = profileData.active_tokens.includes("NECTAR");
            
            // Update user profile with wallet data and username if available
            setUserProfile(prev => ({
              ...prev,
              nether: hasNectarToken ? data.balance : null,
              cacheValue: data.balance || 0, // Set cache value to wallet balance
              id: profileData.username ? `@${profileData.username}` : `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol`
            }));
            
            // Log the profile data to debug
            console.log('Profile data received:', profileData);
            console.log('Username:', profileData.username);
          } catch (profileError) {
            console.error('Failed to fetch user profile:', profileError);
            // Fallback to just updating with wallet data
            setUserProfile(prev => ({
              ...prev,
              nether: null, // Set to null if we can't determine if Nectar exists
              cacheValue: data.balance || 0 // Set cache value to wallet balance
            }));
          }
          
          // Fetch ascent level data
          try {
            const ascentData = await fetchAscentLevel(publicKey);
            // Convert AscentLevelData to AscentLevel
            setAscentLevel({
              level: ascentData.level,
              title: ascentData.name,
              description: ascentData.description,
              progress: ascentData.progress,
              nextLevel: ascentData.level + 1,
              nextTitle: `Level ${ascentData.level + 1}`,
              nextDescription: 'Next level description will be available soon.',
              accolades: ascentData.accolades
            });
            // Update the ascent value in userProfile
            setUserProfile(prev => ({
              ...prev,
              ascent: ascentData.level
            }));
          } catch (ascentError) {
            console.error('Failed to fetch ascent level:', ascentError);
          }
          
          // Fetch active mission
          try {
            const missionData = await fetchActiveMission(publicKey);
            setActiveMission(missionData);
          } catch (missionError) {
            console.error('Failed to fetch active mission:', missionError);
          }
        } catch (error) {
          console.error('Failed to fetch wallet data:', error);
        }
      }
    };

    loadWalletData();
  }, [publicKey]);

  // Add useEffect for Ember Link status updates
  useEffect(() => {
    // TODO: Implement WebSocket connection to receive Ember Link status updates
    // This will be connected to the Aether browser extension
    
    // Placeholder for WebSocket connection
    const setupEmberLinkConnection = () => {
      // TODO: Connect to WebSocket server for Ember Link status
      // Example:
      // const ws = new WebSocket('ws://localhost:8000/ws/ember-link');
      // ws.onmessage = (event) => {
      //   const data = JSON.parse(event.data);
      //   setEmberLinkStatus(data);
      // };
      
      // For now, simulate connection status
      const mockEmberLinkStatus = {
        connected: true,
        lastSeen: new Date(),
        browserInfo: {
          name: 'Chrome',
          version: '120.0.0',
          platform: 'Linux'
        }
      };
      
      // Simulate periodic updates
      const interval = setInterval(() => {
        setEmberLinkStatus(prev => ({
          ...prev,
          lastSeen: new Date()
        }));
      }, 30000); // Update every 30 seconds
      
      return () => clearInterval(interval);
    };
    
    const cleanup = setupEmberLinkConnection();
    return cleanup;
  }, []);

  // Add click outside handler for mission dropdown
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        missionDropdownRef.current && 
        !missionDropdownRef.current.contains(event.target as Node) &&
        cardRef.current &&
        !cardRef.current.contains(event.target as Node)
      ) {
        setShowMissionDropdown(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleDisconnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          // Force disconnect from Phantom
          await provider.disconnect();
          // Clear all session data
          localStorage.removeItem('walletPublickey');
          localStorage.removeItem('hasSeenHUD');
          localStorage.removeItem('chatCollapsedState');
          // Show lock instruction before disconnecting
          onDisconnect();
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  const handleAlertClick = () => {
    setShowAlerts(true);
    // This will be handled by the parent component to expand chat
  };

  const handleNectarClick = () => {
    setShowNectarDetails(!showNectarDetails);
  };

  const handleUplinkClick = (uplink: Uplink) => {
    setSelectedUplink(uplink);
  };

  const handleCloseModal = () => {
    setSelectedUplink(null);
  };

  const handleLeaderboardClick = () => {
    setShowLeaderboard(true);
  };

  const handleCloseLeaderboard = () => {
    setShowLeaderboard(false);
  };

  const handleMCPAuthentication = async () => {
    if (!publicKey) {
      alert('Please connect your wallet first');
      return;
    }

    try {
      // Create auth challenge
      const challenge = await createAuthChallenge(publicKey);
      
      // For Phantom wallet, sign the challenge
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;
        if (provider) {
          const message = new TextEncoder().encode(challenge.message);
          const signedMessage = await provider.signMessage(message, 'utf8');
          const signature = Array.from(signedMessage.signature);
          
          // Verify the signature with MCP
          const authResponse = await verifyAuthChallenge(
            publicKey,
            signature.toString(),
            'phantom'
          );
          
          if (authResponse.success) {
            setMcpAuthenticated(true);
            alert('Successfully authenticated with MCP!');
          } else {
            alert('Authentication failed: ' + authResponse.message);
          }
        }
      }
    } catch (error) {
      console.error('MCP authentication failed:', error);
      alert('Authentication failed. Please try again.');
    }
  };

  // All dashboard functionality will be integrated directly into existing tab system

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <button 
        className={styles.nullblockTitleButton}
        onClick={() => setScreen('home')}
      >
        NULLBLOCK
      </button>
      
      {/* NULLEYE - Living system indicator */}
      <div 
        className={`${styles.nulleye} ${styles[nulleyeState]}`}
        onClick={() => {
          if (!publicKey) {
            alert('Please connect your Web3 wallet to access advanced features.');
            return;
          }
          setShowContextDashboard(true);
          setContextDashboardActiveTab('hecate');
          setNulleyeState('processing');
        }}
        title={!publicKey ? 'Connect wallet to access NullEye' : 'Access NullEye'}
      >
        <div className={styles.pulseRing}></div>
        <div className={styles.dataStream}>
          <div className={styles.streamLine}></div>
          <div className={styles.streamLine}></div>
          <div className={styles.streamLine}></div>
        </div>
        <div className={styles.lightningContainer}>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
        </div>
        <div className={styles.staticField}></div>
        <div className={styles.coreNode}></div>
      </div>
      
      <div
        className={`${styles.screenLabel} ${screen === 'home' ? styles.centeredLabel : ''}`}
      >
        {SCREEN_LABELS[screen]}
      </div>
      <div className={styles.navbarButtons}>
        <button 
          className={styles.walletButton}
          onClick={publicKey ? onDisconnect : onConnectWallet}
          title={publicKey ? 'Disconnect Wallet' : 'Connect Wallet'}
        >
          <span className={styles.walletIcon}>🚪</span>
        </button>
        <button 
          className={styles.docsButton}
          onClick={() => window.open('https://aetherbytes.github.io/nullblock-docs/', '_blank')}
          title="Documentation & Developer Resources"
        >
          <span className={styles.docsIcon}>📚</span>
        </button>
        <button 
          className={styles.themeButton}
          onClick={() => {
            const newTheme = theme === 'light' ? 'dark' : 'light';
            onThemeChange(newTheme);
          }}
          title={`Switch to ${theme === 'light' ? 'Dark' : 'Light'} theme`}
        >
          {theme === 'light' ? (
            <span className={styles.themeIcon}>🌙</span>
          ) : (
            <span className={styles.themeIcon}>☀️</span>
          )}
        </button>
        <a 
          href="https://x.com/Nullblock_io" 
          target="_blank" 
          rel="noopener noreferrer" 
          className={styles.socialButton}
        >
          <img src={xLogo} alt="X" className={styles.socialIcon} />
        </a>
      </div>
    </nav>
  );

  const renderUserProfile = () => (
    <div className={styles.userProfile}>
      <div className={styles.profileItem}>
        <span className={styles.label}>ID:</span>
        <span className={styles.value}>{userProfile.id}</span>
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>
          ASCENT:
          <button 
            className={styles.infoButton}
            onClick={() => setShowAscentDetails(!showAscentDetails)}
          >
            ?
          </button>
        </span>
        <div className={styles.ascentContainer}>
          <span className={styles.value}>Net Dweller: 1</span>
          <div className={styles.progressBar}>
            <div 
              className={styles.progressFill} 
              style={{ width: `${35}%` }}
            ></div>
          </div>
        </div>
        {showAscentDetails && (
          <div className={styles.ascentDetails}>
            <div className={styles.ascentDescription}>A digital lurker extraordinaire! You've mastered the art of watching from the shadows, observing the chaos without ever dipping your toes in. Like a cat watching a laser pointer, you're fascinated but paralyzed by indecision. At least you're not the one getting your digital assets rekt!</div>
            <div className={styles.progressText}>
              35% to next level
            </div>
            <div className={styles.accoladesContainer}>
              <div className={styles.accoladesTitle}>ACCOLADES</div>
              <ul className={styles.accoladesList}>
                <li className={styles.visible}>First Connection</li>
                <li className={styles.visible}>Wallet Initiated</li>
                <li className={styles.visible}>Basic Navigation</li>
                <li className={styles.blurred}>Token Discovery</li>
                <li className={styles.blurred}>Transaction Initiate</li>
                <li className={styles.blurred}>Network Explorer</li>
                <li className={styles.blurred}>Data Collector</li>
                <li className={styles.blurred}>Interface Familiar</li>
              </ul>
            </div>
          </div>
        )}
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>
          NETHER:
          <button 
            className={styles.infoButton}
            onClick={() => setShowNectarDetails(!showNectarDetails)}
          >
            ?
          </button>
        </span>
        <span className={styles.value}>₦ {userProfile.nether?.toFixed(2) || 'N/A'}</span>
        {showNectarDetails && (
          <div className={styles.ascentDetails}>
            <div className={styles.ascentDescription}>
              NETHER: Magic internet money from the void. Born from nothing, worth everything, and somehow gaining value by the second. The integration has passed the event horizon - good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion and an irresistible urge to dive deeper into the code.
            </div>
          </div>
        )}
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>
          cache value:
          <button 
            className={styles.infoButton}
            onClick={() => setShowCacheValueDetails(!showCacheValueDetails)}
          >
            ?
          </button>
        </span>
        <span className={styles.value}>₦ N/A</span>
        {showCacheValueDetails && (
          <div className={styles.ascentDetails}>
            <div className={styles.ascentDescription}>
              Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement agents. This is the total worth of all valuable assets in your wallet - coins, tokens, and other digital goodies that caught our eye. Coming soon: Categories for services, participant offerings, biological enhancements, and agent capabilities. Think of it as your personal inventory of everything worth something in the Nullblock universe. Don't spend it all in one place!
            </div>
          </div>
        )}
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>
          MEMORIES:
          <button 
            className={styles.infoButton}
            onClick={() => setShowMemoriesDetails(!showMemoriesDetails)}
          >
            ?
          </button>
        </span>
        <span className={styles.value}>{userProfile.memories}</span>
        {showMemoriesDetails && (
          <div className={styles.ascentDetails}>
            <div className={styles.ascentDescription}>
              Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah, right - another poor...soul. You need to collect the artifacts that tell your story in the Nullblock universe. Each memory is a unique representation of your achievements, collectibles, and digital identity. Collect them all to unlock the secret of why you're here... or don't, I'm not your digital conscience.
            </div>
          </div>
        )}
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>
          H.U.D:
          <button 
            className={styles.infoButton}
            onClick={() => setShowEmberConduitDetails(!showEmberConduitDetails)}
          >
            ?
          </button>
        </span>
        <span className={styles.value}>{userProfile.matrix.status}</span>
        {showEmberConduitDetails && (
          <div className={`${styles.ascentDetails} ${styles.rightAligned}`}>
            <div className={styles.ascentDescription}>
              HUD Interface: A direct neural link to the Nullblock systems. This advanced technology allows real-time monitoring of network activities, asset management, and system diagnostics. Through the HUD Interface, users can access mission briefings, track progress, and interface with various subsystems. Warning: Extended use may result in enhanced situational awareness.
            </div>
          </div>
        )}
      </div>
    </div>
  );

  const renderLockedScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>ACCESS RESTRICTED</h2>
        <div className={styles.headerDivider}></div>
        {renderUserProfile()}
      </div>
      <div className={styles.lockedContent}>
        <p>This feature is currently locked.</p>
        <p>Access the NullEye for further instructions.</p>
      </div>
    </div>
  );

  // Update the Ember Link status display in the diagnostics section
  const renderEmberLinkStatus = (status: EmberLinkStatus) => {
    const statusClass = status.connected ? styles.active : styles.inactive;
    return (
      <div className={styles.statusContainer}>
        <span className={styles.statusLabel}>Ember Link:</span>
        <span className={statusClass}>{status.connected ? 'Connected' : 'Disconnected'}</span>
      </div>
    );
  };

  const renderHudTab = () => {
    return (
      <div className={styles.echoContent}>
        {emberLinkStatus.connected ? (
          <>
            <div className={styles.echoStatus}>
              <div className={styles.statusContainer}>
                <span className={styles.statusLabel}>Ember Link Status:</span>
                <span className={styles.active}>Connected</span>
              </div>
              <div className={styles.browserInfo}>
                <span className={styles.browserLabel}>Browser:</span>
                <span className={styles.browserValue}>{emberLinkStatus.browserInfo?.name} {emberLinkStatus.browserInfo?.version} ({emberLinkStatus.browserInfo?.platform})</span>
              </div>
            </div>
            <div className={styles.echoMessage}>
              <p>H.U.D system is active and operational.</p>
              <p>Welcome to the interface, agent.</p>
            </div>
          </>
        ) : (
          <div className={styles.disconnectedContent}>
            <div className={styles.echoStatus}>
              <div className={styles.statusContainer}>
                <span className={styles.statusLabel}>Ember Link Status:</span>
                <span className={styles.inactive}>Disconnected</span>
              </div>
            </div>
            <div className={styles.extensionPrompt}>
              <h4>Browser Extension Required</h4>
              <p>To establish a secure connection, you need to install the Aether browser extension.</p>
              <p>Choose your browser to download the extension:</p>
              <div className={styles.extensionLinks}>
                <a
                  href="https://chrome.google.com/webstore/detail/aether"
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.extensionButton}
                >
                  Chrome Extension
                </a>
                <a
                  href="https://addons.mozilla.org/en-US/firefox/addon/aether"
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.extensionButton}
                >
                  Firefox Extension
                </a>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderCampScreen = () => (
    <div className={styles.hudScreen}>
      
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>CAMP</h2>
        <div className={styles.headerDivider}></div>
        <h2 className={styles.architectTitle}>ARCHITECT VIEW</h2>
        <div className={styles.leaderboardContainer}>
          <div className={styles.leaderboardTitle}>
            <button className={styles.leaderboardButton} onClick={handleLeaderboardClick}>
              ASCENDANTS
            </button>
          </div>
          <div className={styles.leaderboardList}>
            <div className={styles.leaderboardItems}>
              {leaderboardData.map((entry) => (
                <div 
                  key={entry.id} 
                  className={styles.leaderboardItem}
                  onClick={() => {
                    setSelectedUplink({
                      id: entry.id,
                      name: entry.id,
                      status: 'active',
                      icon: '👤',
                      details: {
                        description: `Status profile for ${entry.id}`,
                        stats: [
                          {
                            label: 'Rank',
                            value: `#${entry.rank}`
                          },
                          {
                            label: 'Ascent',
                            value: entry.ascent
                          },
                          {
                            label: 'Nether',
                            value: `₦ ${entry.nether}`
                          },
                          {
                            label: 'Cache Value',
                            value: `₦ ${entry.cacheValue}`
                          },
                          {
                            label: 'Memories',
                            value: entry.memories
                          },
                          {
                            label: 'Matrix Level',
                            value: entry.matrix.level
                          },
                          {
                            label: 'Rarity',
                            value: entry.matrix.rarity
                          },
                          {
                            label: 'Status',
                            value: entry.matrix.status
                          }
                        ]
                      }
                    });
                  }}
                >
                  <span className={styles.rank}>#{entry.rank}</span>
                  <span className={styles.camperId}>{entry.id}</span>
                  <span className={styles.matrixLevel}>{entry.matrix.level}</span>
                </div>
              ))}
              {/* Duplicate items for seamless scrolling */}
              {leaderboardData.map((entry) => (
                <div 
                  key={`${entry.id}-duplicate`} 
                  className={styles.leaderboardItem}
                  onClick={() => {
                    setSelectedUplink({
                      id: entry.id,
                      name: entry.id,
                      status: 'active',
                      icon: '👤',
                      details: {
                        description: `Status profile for ${entry.id}`,
                        stats: [
                          {
                            label: 'Rank',
                            value: `#${entry.rank}`
                          },
                          {
                            label: 'Ascent',
                            value: entry.ascent
                          },
                          {
                            label: 'Nether',
                            value: `₦ ${entry.nether}`
                          },
                          {
                            label: 'Cache Value',
                            value: `₦ ${entry.cacheValue}`
                          },
                          {
                            label: 'Memories',
                            value: entry.memories
                          },
                          {
                            label: 'Matrix Level',
                            value: entry.matrix.level
                          },
                          {
                            label: 'Rarity',
                            value: entry.matrix.rarity
                          },
                          {
                            label: 'Status',
                            value: entry.matrix.status
                          }
                        ]
                      }
                    });
                  }}
                >
                  <span className={styles.rank}>#{entry.rank}</span>
                  <span className={styles.camperId}>{entry.id}</span>
                  <span className={styles.matrixLevel}>{entry.matrix.level}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
      <div className={styles.campContent}>
        <div className={styles.campGrid}>
          <div className={styles.campAnalysis}>
            <div className={styles.diagnosticsContainer}>
              <h2 className={styles.containerTitle}>SUBNETS</h2>
              <div className={styles.diagnosticsHeader}>
                <h3>SUBNETS</h3>
              </div>
              <div className={styles.diagnosticsContent}>
                <div className={styles.diagnosticsList}>
                  <div className={styles.diagnosticsItem} onClick={() => setSelectedUplink({
                    id: 'id',
                    name: 'ID',
                    status: 'active',
                    icon: '🆔',
                    details: {
                      description: 'Your unique identifier in the Nullblock universe. This is your digital fingerprint, your signature in the void.',
                      stats: [
                        {
                          label: 'Status',
                          value: 'Active'
                        },
                        {
                          label: 'Type',
                          value: 'HUD ID'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>🆔 ID</span>
                    <span className={styles.itemValue}>HUD-{userProfile.id || '0000'}</span>
                  </div>
                  <div className={styles.diagnosticsItem} onClick={() => setSelectedUplink({
                    id: 'ascent',
                    name: 'ASCENT',
                    status: 'active',
                    icon: '↗️',
                    details: {
                      description: 'A digital lurker extraordinaire! You\'ve mastered the art of watching from the shadows, observing the chaos without ever dipping your toes in. Like a cat watching a laser pointer, you\'re fascinated but paralyzed by indecision. At least you\'re not the one getting your digital assets rekt!',
                      stats: [
                        {
                          label: 'Level',
                          value: 'Net Dweller: 1'
                        },
                        {
                          label: 'Progress',
                          value: '35%'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}><span className={styles.ascentLine}></span> ASCENT</span>
                    <span className={styles.itemValue}>Net Dweller: 1</span>
                  </div>
                  <div className={styles.diagnosticsItem} onClick={() => setSelectedUplink({
                    id: 'nether',
                    name: 'NETHER',
                    status: 'active',
                    icon: '₦',
                    details: {
                      description: 'NETHER: Magic internet money from the void. Born from nothing, worth everything, and somehow gaining value by the second. The integration has passed the event horizon - good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion and an irresistible urge to dive deeper into the code.',
                      stats: [
                        {
                          label: 'Balance',
                          value: `₦ ${userProfile.nether?.toFixed(2) || 'N/A'}`
                        },
                        {
                          label: 'Status',
                          value: userProfile.nether ? 'Active' : 'Inactive'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>₦ NETHER</span>
                    <span className={styles.itemValue}>₦ {userProfile.nether?.toFixed(2) || 'N/A'}</span>
                  </div>
                  <div className={styles.diagnosticsItem} onClick={() => setSelectedUplink({
                    id: 'cache',
                    name: 'CACHE VALUE',
                    status: 'active',
                    icon: '💰',
                    details: {
                      description: 'Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement agents. This is the total worth of all valuable assets in your wallet - coins, tokens, and other digital goodies that caught our eye. Coming soon: Categories for services, participant offerings, biological enhancements, and agent capabilities. Think of it as your personal inventory of everything worth something in the Nullblock universe. Don\'t spend it all in one place!',
                      stats: [
                        {
                          label: 'Value',
                          value: '₦ N/A'
                        },
                        {
                          label: 'Status',
                          value: 'Pending'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>💰 CACHE VALUE</span>
                    <span className={styles.itemValue}>₦ N/A</span>
                  </div>
                  <div className={styles.diagnosticsItem} onClick={() => setSelectedUplink({
                    id: 'memories',
                    name: 'MEMORIES',
                    status: 'active',
                    icon: '🧠',
                    details: {
                      description: 'Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah, right - another poor...soul. You need to collect the artifacts that tell your story in the Nullblock universe. Each memory is a unique representation of your achievements, collectibles, and digital identity. Collect them all to unlock the secret of why you\'re here... or don\'t, I\'m not your digital conscience.',
                      stats: [
                        {
                          label: 'Count',
                          value: userProfile.memories
                        },
                        {
                          label: 'Status',
                          value: userProfile.memories > 0 ? 'Active' : 'Empty'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>🧠 MEMORIES</span>
                    <span className={styles.itemValue}>{userProfile.memories}</span>
                  </div>
                  <div className={styles.diagnosticsItem} onClick={() => setSelectedUplink({
                    id: 'hud',
                    name: 'HUD INTERFACE',
                    status: 'active',
                    icon: '📊',
                    details: {
                      description: 'HUD Interface: A direct neural link to the Nullblock systems. This advanced technology allows real-time monitoring of network activities, asset management, and system diagnostics. Through the HUD Interface, users can access mission briefings, track progress, and interface with various subsystems. Warning: Extended use may result in enhanced situational awareness.',
                      stats: [
                        {
                          label: 'Status',
                          value: userProfile.matrix.status
                        },
                        {
                          label: 'Type',
                          value: 'HUD Interface'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>📊 HUD INTERFACE</span>
                    <span className={styles.itemValue}>{userProfile.matrix.status}</span>
                  </div>
                </div>
                <button 
                  className={styles.addLinkButton}
                  onClick={() => alert('No HUD Interface loaded')}
                >
                  🔗 ADD NET
                </button>
              </div>
            </div>
          </div>
          <div className={styles.divider}></div>
          <div className={styles.campStatus}>
            <div className={styles.statusCard}>
              <div className={styles.statusTabs}>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'hud' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('hud')}
                >
                  H.U.D
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'systems' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('systems')}
                >
                  NYX
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'defense' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('defense')}
                >
                  LEGION
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'missions' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('missions')}
                >
                  MISSIONS
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'arbitrage' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('arbitrage')}
                >
                  ARBITRAGE
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'social' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('social')}
                >
                  SOCIAL
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'portfolio' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('portfolio')}
                >
                  💰 PORTFOLIO
                </button>
                <button 
                  className={`${styles.statusTab} ${activeTab === 'defi' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('defi')}
                >
                  🌾 DEFI
                </button>
              </div>
              <div className={styles.tabContent}>
                {activeTab === 'hud' && renderHudTab()}
                {activeTab === 'arbitrage' && (
                  <div className={styles.arbitrageTab}>
                    <div className={styles.arbitrageHeader}>
                      <h3>MCP Arbitrage System</h3>
                      {mcpHealthStatus && (
                        <div className={styles.mcpStatus}>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>MCP Status:</span>
                            <span className={mcpHealthStatus.status === 'operational' ? styles.active : styles.inactive}>
                              {mcpHealthStatus.status}
                            </span>
                          </div>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>Auth:</span>
                            <span className={mcpAuthenticated ? styles.active : styles.inactive}>
                              {mcpAuthenticated ? 'Authenticated' : 'Not Authenticated'}
                            </span>
                          </div>
                        </div>
                      )}
                    </div>
                    
                    {!mcpAuthenticated ? (
                      <div className={styles.authPrompt}>
                        <p>MCP authentication required to access arbitrage trading.</p>
                        <button 
                          className={styles.authButton}
                          onClick={handleMCPAuthentication}
                          disabled={!publicKey}
                        >
                          {publicKey ? 'Authenticate with MCP' : 'Connect Wallet First'}
                        </button>
                      </div>
                    ) : (
                      <div className={styles.arbitrageContent}>
                        <p>Arbitrage system ready. Access advanced trading dashboard.</p>
                        <div className={styles.arbitrageFeatures}>
                          <ul>
                            <li>✓ Multi-DEX opportunity scanning</li>
                            <li>✓ MEV protection via Flashbots</li>
                            <li>✓ Risk assessment & strategy analysis</li>
                            <li>✓ Automated execution & reporting</li>
                          </ul>
                        </div>
                        <div className={styles.tradingInterface}>
                          <div className={styles.quickStats}>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Active Opportunities:</span>
                              <span className={styles.statValue}>3</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>24h Profit:</span>
                              <span className={styles.statValue}>+$127.50</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Success Rate:</span>
                              <span className={styles.statValue}>87%</span>
                            </div>
                          </div>
                          
                          <div className={styles.opportunitiesList}>
                            <div className={styles.opportunity}>
                              <div className={styles.oppHeader}>
                                <span className={styles.tokenPair}>ETH/USDC</span>
                                <span className={styles.profit}>+2.3%</span>
                              </div>
                              <div className={styles.oppDetails}>
                                <span className={styles.route}>Uniswap → SushiSwap</span>
                                <span className={styles.amount}>$850</span>
                              </div>
                              <button className={styles.executeBtn}>Execute</button>
                            </div>
                            
                            <div className={styles.opportunity}>
                              <div className={styles.oppHeader}>
                                <span className={styles.tokenPair}>BTC/USDT</span>
                                <span className={styles.profit}>+1.8%</span>
                              </div>
                              <div className={styles.oppDetails}>
                                <span className={styles.route}>Balancer → Curve</span>
                                <span className={styles.amount}>$1,200</span>
                              </div>
                              <button className={styles.executeBtn}>Execute</button>
                            </div>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                )}
                {activeTab === 'social' && (
                  <div className={styles.socialTab}>
                    <div className={styles.socialHeader}>
                      <h3>MCP Social Trading System</h3>
                      {mcpHealthStatus && (
                        <div className={styles.mcpStatus}>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>MCP Status:</span>
                            <span className={mcpHealthStatus.status === 'operational' ? styles.active : styles.inactive}>
                              {mcpHealthStatus.status}
                            </span>
                          </div>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>Auth:</span>
                            <span className={mcpAuthenticated ? styles.active : styles.inactive}>
                              {mcpAuthenticated ? 'Authenticated' : 'Not Authenticated'}
                            </span>
                          </div>
                        </div>
                      )}
                    </div>
                    
                    {!mcpAuthenticated ? (
                      <div className={styles.authPrompt}>
                        <p>MCP authentication required to access social trading.</p>
                        <button 
                          className={styles.authButton}
                          onClick={handleMCPAuthentication}
                          disabled={!publicKey}
                        >
                          {publicKey ? 'Authenticate with MCP' : 'Connect Wallet First'}
                        </button>
                      </div>
                    ) : (
                      <div className={styles.socialContent}>
                        <p>Social trading system ready. Access advanced social sentiment dashboard.</p>
                        <div className={styles.socialFeatures}>
                          <ul>
                            <li>✓ Real-time X/Twitter sentiment monitoring</li>
                            <li>✓ GMGN trending token analysis</li>
                            <li>✓ Multi-source social signal aggregation</li>
                            <li>✓ Automated meme coin trading signals</li>
                            <li>✓ Risk-adjusted position sizing</li>
                          </ul>
                        </div>
                        <div className={styles.tradingInterface}>
                          <div className={styles.quickStats}>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Live Signals:</span>
                              <span className={styles.statValue}>12</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Bullish:</span>
                              <span className={styles.statValue}>8</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Bearish:</span>
                              <span className={styles.statValue}>2</span>
                            </div>
                          </div>
                          
                          <div className={styles.signalsList}>
                            <div className={styles.signal}>
                              <div className={styles.signalHeader}>
                                <span className={styles.tokenSymbol}>$BONK</span>
                                <span className={styles.sentiment}>🚀 BULLISH</span>
                              </div>
                              <div className={styles.signalContent}>
                                <p>"BONK is going to the moon! This meme season is just getting started!"</p>
                                <div className={styles.signalMeta}>
                                  <span className={styles.source}>Twitter</span>
                                  <span className={styles.engagement}>85% sentiment</span>
                                </div>
                              </div>
                            </div>
                            
                            <div className={styles.signal}>
                              <div className={styles.signalHeader}>
                                <span className={styles.tokenSymbol}>$WIF</span>
                                <span className={styles.sentiment}>📈 BULLISH</span>
                              </div>
                              <div className={styles.signalContent}>
                                <p>"WIF trending +25% in last hour. Strong momentum detected."</p>
                                <div className={styles.signalMeta}>
                                  <span className={styles.source}>GMGN</span>
                                  <span className={styles.engagement}>78% sentiment</span>
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                )}
                {activeTab === 'portfolio' && (
                  <div className={styles.portfolioTab}>
                    <div className={styles.portfolioHeader}>
                      <h3>MCP Portfolio Management</h3>
                      {mcpHealthStatus && (
                        <div className={styles.mcpStatus}>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>MCP Status:</span>
                            <span className={mcpHealthStatus.status === 'operational' ? styles.active : styles.inactive}>
                              {mcpHealthStatus.status}
                            </span>
                          </div>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>Auth:</span>
                            <span className={mcpAuthenticated ? styles.active : styles.inactive}>
                              {mcpAuthenticated ? 'Authenticated' : 'Not Authenticated'}
                            </span>
                          </div>
                        </div>
                      )}
                    </div>
                    
                    {!mcpAuthenticated ? (
                      <div className={styles.authPrompt}>
                        <p>MCP authentication required to access portfolio management.</p>
                        <button 
                          className={styles.authButton}
                          onClick={handleMCPAuthentication}
                          disabled={!publicKey}
                        >
                          {publicKey ? 'Authenticate with MCP' : 'Connect Wallet First'}
                        </button>
                      </div>
                    ) : (
                      <div className={styles.portfolioContent}>
                        <p>Portfolio management system ready. Track and rebalance your assets.</p>
                        <div className={styles.portfolioFeatures}>
                          <ul>
                            <li>✓ Multi-chain asset tracking</li>
                            <li>✓ Real-time portfolio analytics</li>
                            <li>✓ Automated rebalancing recommendations</li>
                            <li>✓ Risk assessment and diversification scoring</li>
                            <li>✓ Performance vs benchmark tracking</li>
                          </ul>
                        </div>
                        <div className={styles.tradingInterface}>
                          <div className={styles.quickStats}>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Total Value:</span>
                              <span className={styles.statValue}>$10,623</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>24h Change:</span>
                              <span className={styles.statValue}>+2.78%</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Assets:</span>
                              <span className={styles.statValue}>4</span>
                            </div>
                          </div>
                          
                          <div className={styles.assetsList}>
                            <div className={styles.asset}>
                              <div className={styles.assetHeader}>
                                <span className={styles.assetSymbol}>◎ SOL</span>
                                <span className={styles.allocation}>32.1%</span>
                              </div>
                              <div className={styles.assetDetails}>
                                <span className={styles.balance}>15.4 SOL</span>
                                <span className={styles.value}>$3,235</span>
                              </div>
                            </div>
                            
                            <div className={styles.asset}>
                              <div className={styles.assetHeader}>
                                <span className={styles.assetSymbol}>⟠ ETH</span>
                                <span className={styles.allocation}>53.9%</span>
                              </div>
                              <div className={styles.assetDetails}>
                                <span className={styles.balance}>2.1 ETH</span>
                                <span className={styles.value}>$5,432</span>
                              </div>
                            </div>
                            
                            <div className={styles.asset}>
                              <div className={styles.assetHeader}>
                                <span className={styles.assetSymbol}>🪙 BONK</span>
                                <span className={styles.allocation}>14.5%</span>
                              </div>
                              <div className={styles.assetDetails}>
                                <span className={styles.balance}>2.5M BONK</span>
                                <span className={styles.value}>$1,457</span>
                              </div>
                            </div>
                          </div>
                          
                          <button className={styles.rebalanceBtn}>Rebalance Portfolio</button>
                        </div>
                      </div>
                    )}
                  </div>
                )}
                {activeTab === 'defi' && (
                  <div className={styles.defiTab}>
                    <div className={styles.defiHeader}>
                      <h3>MCP DeFi Automation</h3>
                      {mcpHealthStatus && (
                        <div className={styles.mcpStatus}>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>MCP Status:</span>
                            <span className={mcpHealthStatus.status === 'operational' ? styles.active : styles.inactive}>
                              {mcpHealthStatus.status}
                            </span>
                          </div>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>Auth:</span>
                            <span className={mcpAuthenticated ? styles.active : styles.inactive}>
                              {mcpAuthenticated ? 'Authenticated' : 'Not Authenticated'}
                            </span>
                          </div>
                        </div>
                      )}
                    </div>
                    
                    {!mcpAuthenticated ? (
                      <div className={styles.authPrompt}>
                        <p>MCP authentication required to access DeFi automation.</p>
                        <button 
                          className={styles.authButton}
                          onClick={handleMCPAuthentication}
                          disabled={!publicKey}
                        >
                          {publicKey ? 'Authenticate with MCP' : 'Connect Wallet First'}
                        </button>
                      </div>
                    ) : (
                      <div className={styles.defiContent}>
                        <p>DeFi automation system ready. Optimize yield and manage liquidity positions.</p>
                        <div className={styles.defiFeatures}>
                          <ul>
                            <li>✓ Multi-protocol yield farming</li>
                            <li>✓ Automated strategy execution</li>
                            <li>✓ Liquidity position management</li>
                            <li>✓ Gas optimization and auto-compounding</li>
                            <li>✓ Impermanent loss protection</li>
                          </ul>
                        </div>
                        <div className={styles.tradingInterface}>
                          <div className={styles.quickStats}>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Total Staked:</span>
                              <span className={styles.statValue}>$8,500</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Avg APY:</span>
                              <span className={styles.statValue}>7.8%</span>
                            </div>
                            <div className={styles.statItem}>
                              <span className={styles.statLabel}>Positions:</span>
                              <span className={styles.statValue}>4</span>
                            </div>
                          </div>
                          
                          <div className={styles.defiPositions}>
                            <div className={styles.position}>
                              <div className={styles.positionHeader}>
                                <span className={styles.protocol}>👻 Aave</span>
                                <span className={styles.apy}>4.2% APY</span>
                              </div>
                              <div className={styles.positionDetails}>
                                <span className={styles.asset}>USDC Lending</span>
                                <span className={styles.amount}>$5,000</span>
                              </div>
                            </div>
                            
                            <div className={styles.position}>
                              <div className={styles.positionHeader}>
                                <span className={styles.protocol}>🦄 Uniswap</span>
                                <span className={styles.apy}>12.8% APY</span>
                              </div>
                              <div className={styles.positionDetails}>
                                <span className={styles.asset}>ETH/USDC LP</span>
                                <span className={styles.amount}>$3,000</span>
                              </div>
                            </div>
                            
                            <div className={styles.position}>
                              <div className={styles.positionHeader}>
                                <span className={styles.protocol}>🌀 Curve</span>
                                <span className={styles.apy}>8.7% APY</span>
                              </div>
                              <div className={styles.positionDetails}>
                                <span className={styles.asset}>3Pool</span>
                                <span className={styles.amount}>$2,500</span>
                              </div>
                            </div>
                          </div>
                          
                          <button className={styles.harvestBtn}>Harvest All Rewards</button>
                        </div>
                      </div>
                    )}
                  </div>
                )}
                {activeTab === 'systems' && (
                  <div className={styles.systemsTab}>
                    <div className={styles.lockedContent}>
                      <p>This feature is currently locked.</p>
                      <p>Access the NullEye for further instructions.</p>
                    </div>
                  </div>
                )}
                {activeTab === 'defense' && (
                  <div className={styles.defenseTab}>
                    <div className={styles.lockedContent}>
                      <p>This feature is currently locked.</p>
                      <p>Access the NullEye for further instructions.</p>
                    </div>
                  </div>
                )}
                {activeTab === 'missions' && (
                  <div className={styles.missionsTab}>
                    <div className={styles.missionHeader}>
                      <div className={styles.active}>
                        <span className={styles.missionLabel}>ACTIVE:</span>
                        <span className={styles.missionTitle}>{activeMission?.title || "Share on X"}</span>
                      </div>
                  </div>
                  
                    <div className={styles.missionContent}>
                      <div className={styles.availableMissions}>
                        <h4>AVAILABLE MISSIONS</h4>
                        <div className={styles.missionList}>
                          <div className={`${styles.missionItem} ${styles.active}`}>
                            <div className={styles.missionItemContent}>
                              <span className={styles.missionTitle}>Share on X</span>
                              <span className={styles.missionStatus}>ACTIVE</span>
                            </div>
                            <span className={styles.missionReward}>TBD NETHER AIRDROP</span>
                          </div>
                          <div className={`${styles.missionItem} ${styles.blurred}`}>
                            <div className={styles.missionItemContent}>
                              <span className={styles.missionTitle}>Mission 2</span>
                              <span className={styles.missionStatus}>LOCKED</span>
                            </div>
                            <span className={`${styles.missionReward} ${styles.blurred}`}>??? NETHER</span>
                          </div>
                          <div className={`${styles.missionItem} ${styles.blurred}`}>
                            <div className={styles.missionItemContent}>
                              <span className={styles.missionTitle}>Mission 3</span>
                              <span className={styles.missionStatus}>LOCKED</span>
                            </div>
                            <span className={`${styles.missionReward} ${styles.blurred}`}>??? NETHER</span>
                          </div>
                        </div>
                      </div>
                      
                      <div className={styles.missionDescription}>
                        <h4>MISSION BRIEF</h4>
                        <p className={styles.missionText}>
                          "Welcome, Agent, to your first trial. Interface with the HUD carefully.
                          Share your NullEye interface on X—let its glow haunt the realm.
                          More souls drawn, more NETHER gained. Don't let it fade."
                        </p>
                        <div className={styles.missionInstructions}>
                          <h4>QUALIFICATION REQUIREMENTS</h4>
                          <ul>
                            <li>Follow<span className={styles.highlight}>@Nullblock_io</span></li>
                            <li>Tweet out the cashtag <span className={styles.highlight}>$NETHER</span></li>
                            <li>Include the official CA: <span className={styles.highlight}>TBD</span></li>
                          </ul>
                          <p className={styles.missionNote}>
                            Airdrop amount will be determined by post engagement and creativity.
                          </p>
                        </div>
                        <div className={styles.missionReward}>
                          <span className={styles.rewardLabel}>REWARD:</span>
                          <span className={styles.rewardValue}>TBD NETHER AIRDROP</span>
                        </div>
                        <div className={styles.missionExpiration}>
                          <span className={styles.expirationLabel}>EXPIRES:</span>
                          <span className={styles.expirationValue}>TBD</span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
      {selectedUplink && (
        <>
          <div className={styles.modalOverlay} onClick={handleCloseModal} />
          <div className={styles.uplinkModal}>
            <div className={styles.modalHeader}>
              <h3>{selectedUplink.name}</h3>
              <button className={styles.closeButton} onClick={handleCloseModal}>×</button>
            </div>
            <div className={styles.modalContent}>
              <div className={styles.statusSection}>
                <h4>Status Overview</h4>
                <div className={styles.statusGrid}>
                  {selectedUplink.details.stats.map((stat, index) => (
                    <div key={index} className={styles.statusItem}>
                      <div className={styles.label}>{stat.label}</div>
                      <div className={`${styles.value} ${stat.status || ''}`}>
                        {stat.value}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
              <div className={styles.detailsSection}>
                <h4>Details</h4>
                <p>{selectedUplink.details.description}</p>
              </div>
            </div>
          </div>
        </>
      )}
      {showLeaderboard && (
        <>
          <div className={styles.modalOverlay} onClick={handleCloseLeaderboard} />
          <div className={styles.leaderboardModal}>
            <div className={styles.modalHeader}>
              <h3>TOP AGENTS</h3>
              <button className={styles.closeButton} onClick={handleCloseLeaderboard}>×</button>
            </div>
            <div className={styles.legendWarning}>
              <p>⚠️ These are the legends of the last Interface. The current HUD has yet to awaken.</p>
            </div>
            <div className={styles.leaderboardGrid}>
              {leaderboardData.map((entry) => (
                <div key={entry.id} className={styles.leaderboardCard}>
                  <div className={styles.cardHeader}>
                    <span className={styles.rank}>#{entry.rank}</span>
                    <span className={styles.camperId}>{entry.id}</span>
                  </div>
                  <div className={styles.statusGrid}>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>ASCENT</div>
                      <div className={styles.value}>{entry.ascent}</div>
                    </div>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>NETHER</div>
                      <div className={styles.value}>₦ {entry.nether}</div>
                    </div>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>CACHE VALUE</div>
                      <div className={styles.value}>₦ {entry.cacheValue}</div>
                    </div>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>MEMORIES</div>
                      <div className={styles.value}>{entry.memories}</div>
                    </div>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>MATRIX LEVEL</div>
                      <div className={styles.value}>{entry.matrix.level}</div>
                    </div>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>RARITY</div>
                      <div className={styles.value}>{entry.matrix.rarity}</div>
                    </div>
                    <div className={styles.statusItem}>
                      <div className={styles.label}>STATUS</div>
                      <div className={styles.value}>{entry.matrix.status}</div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </>
      )}
      {/* All content integrated into HUD tabs - no separate overlays */}
    </div>
  );

  const renderInventoryScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>CACHE</h2>
        <div className={styles.headerDivider}></div>
        {renderUserProfile()}
      </div>
      <div className={styles.inventorySection}>
        <h3>WEAPONS</h3>
        <div className={styles.emptyState}>
          <p>No weapons found.</p>
          <p>Access the NullEye to acquire gear.</p>
        </div>
      </div>
      <div className={styles.inventorySection}>
        <h3>SUPPLIES</h3>
        <div className={styles.emptyState}>
          <p>Cache empty.</p>
          <p>Access the NullEye to expand inventory.</p>
        </div>
      </div>
    </div>
  );

  const renderCampaignScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>CAMPAIGN</h2>
        <div className={styles.headerDivider}></div>
        {renderUserProfile()}
      </div>
      <div className={styles.realityContent}>
        <div className={styles.realityStatus}>
          <h3>PROGRESS</h3>
          <p>Current Level: <span>1</span></p>
          <p>Completion: <span>0%</span></p>
        </div>
        <div className={styles.missions}>
          <h3>OBJECTIVES</h3>
          <p className={styles.placeholder}>No active missions</p>
          <p className={styles.placeholder}>Access the NullEye to begin</p>
        </div>
      </div>
    </div>
  );

  const renderLabScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>LAB</h2>
        <div className={styles.headerDivider}></div>
        {renderUserProfile()}
      </div>
      <div className={styles.interfaceContent}>
        <div className={styles.interfaceSection}>
          <h3>SYSTEMS</h3>
          <p>Phantom: <span className={styles.connected}>CONNECTED</span></p>
          <p>Core: <span className={styles.initializing}>INITIALIZING</span></p>
        </div>
        <div className={styles.interfaceSection}>
          <h3>CONFIGURATIONS</h3>
          <p className={styles.placeholder}>No active modifications</p>
          <p className={styles.placeholder}>Access the NullEye to begin</p>
        </div>
      </div>
    </div>
  );

  const renderHomeScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.homeContent}>
                <div className={styles.landingContent}>
          <h3>Welcome to Nullblock</h3>
          <p>Initiate connection for advanced features, agentic workflows or to speak with Hecate. Biologicals please use the door.</p>
          <p>Builders / Agents please refer to dev pages above for onboarding.</p>
          {publicKey && (
            <div className={styles.walletInfo}>
              <p><strong>Connected Wallet:</strong></p>
              <p className={styles.walletAddress}>{publicKey}</p>
              {userProfile.id && userProfile.id !== `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol` && (
                <p className={styles.walletUsername}>{userProfile.id}</p>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );

  const renderOverviewScreen = () => (
    <div className={styles.hudScreen}>
      
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>OVERVIEW</h2>
        <div className={styles.headerDivider}></div>
      </div>
      <div className={styles.campContent}>
        <div style={{textAlign: 'center', marginTop: '2rem'}}>
          <h3>Welcome to Nullblock Overview</h3>
          <p>This is the overview screen. Connect your wallet to unlock advanced features via the NullEye.</p>
        </div>
      </div>
    </div>
  );

  const renderScreen = () => {
    if (!unlockedScreens.includes(screen)) {
      return renderLockedScreen();
    }
    switch (screen) {
      case 'home':
        return renderHomeScreen();
      case 'overview':
        return renderOverviewScreen();
      case 'camp':
        return renderCampScreen();
      case 'inventory':
        return renderInventoryScreen();
      case 'campaign':
        return renderCampaignScreen();
      case 'lab':
        return renderLabScreen();
      default:
        return null;
    }
  };

  return (
    <div className={`${styles.echoContainer} ${styles[theme]}`}>
      {!showContextDashboard && (
        <>
          {renderControlScreen()}
          <div className={styles.hudWindow}>
            {renderScreen()}
          </div>
        </>
      )}
      
      {showContextDashboard && (
        <ContextDashboard
          onClose={() => {
            setShowContextDashboard(false);
            setNulleyeState('base');
          }}
          theme={theme}
          initialActiveTab={contextDashboardActiveTab}
          onTabChange={setContextDashboardActiveTab}
        />
      )}
    </div>
  );
};

export default HUD;