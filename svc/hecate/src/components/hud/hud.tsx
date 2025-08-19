import React, { useState, useEffect, useRef } from 'react';
import xLogo from '../../assets/images/X_logo_black.png';
import nullviewLogo from '../../assets/images/nullview_logo.png';
import type { MissionData } from '../../common/services/api';
import {
  fetchWalletData,
  fetchUserProfile,
  fetchAscentLevel,
  fetchActiveMission,
} from '../../common/services/api';
import {
  isAuthenticated,
  restoreSession,
  createAuthChallenge,
  verifyAuthChallenge,
  checkMCPHealth,
} from '../../common/services/mcp-api';
// Removed separate dashboard imports - all functionality is now integrated into HUD tabs
import HecateHud from '../hecateHud';
import { hecateAgent, type ChatMessage as HecateChatMessage, type HecateResponse } from '../../common/services/hecate-agent';
import styles from './hud.module.scss';

type Screen = 'home' | 'overview' | 'camp' | 'inventory' | 'campaign' | 'lab';
type Theme = 'null' | 'light' | 'dark';
type TabType =
  | 'missions'
  | 'systems'
  | 'defense'
  | 'uplink'
  | 'hud'
  | 'status'
  | 'arbitrage'
  | 'portfolio'
  | 'defi';

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
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
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
    stats: {
      label: string;
      value: string | number;
      status?: string;
    }[];
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
  systemStatus,
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
      status: 'N/A',
    },
  });
  const [ascentLevel, setAscentLevel] = useState<AscentLevel | null>(null);
  const [walletName, setWalletName] = useState<string | null>(null);
  const [isEditingName, setIsEditingName] = useState<boolean>(false);

  // Function to resolve wallet names (SNS for Solana, ENS for Ethereum)
  const resolveWalletName = async (address: string, walletType: string) => {
    try {
      if (walletType === 'phantom') {
        // For Solana - try to resolve SNS (Solana Name Service)
        // Note: This would require @solana/spl-name-service or similar
        console.log('Attempting to resolve Solana name for:', address);
        
        // For now, we'll try a simple API approach or check local storage
        const savedName = localStorage.getItem(`walletName_${address}`);
        if (savedName) {
          setWalletName(savedName);
          return savedName;
        }
        
        // TODO: Implement actual SNS resolution
        // const connection = new Connection('https://api.mainnet-beta.solana.com');
        // const nameAccount = await NameRegistryState.retrieve(connection, ...);
        
      } else if (walletType === 'metamask') {
        // For Ethereum - try to resolve ENS
        console.log('Attempting to resolve ENS name for:', address);
        
        // Check if the wallet provider supports ENS
        if (window.ethereum && typeof window.ethereum.request === 'function') {
          try {
            const ensName = await window.ethereum.request({
              method: 'wallet_lookupEnsName',
              params: [address],
            });
            
            if (ensName) {
              setWalletName(ensName);
              localStorage.setItem(`walletName_${address}`, ensName);
              return ensName;
            }
          } catch (ensError) {
            console.log('ENS lookup not supported or failed:', ensError);
          }
        }
      }
    } catch (error) {
      console.log('Failed to resolve wallet name:', error);
    }
    
    return null;
  };

  // Function to manually set wallet name
  const setCustomWalletName = (name: string) => {
    if (publicKey && name.trim()) {
      const cleanName = name.trim().replace('@', ''); // Remove @ if user typed it
      setWalletName(cleanName);
      localStorage.setItem(`walletName_${publicKey}`, cleanName);
      setIsEditingName(false);
    }
  };
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
    browserInfo: null,
  });
  const [isDigitizing, setIsDigitizing] = useState<boolean>(false);
  const [selectedUplink, setSelectedUplink] = useState<Uplink | null>(null);
  const [showLeaderboard, setShowLeaderboard] = useState<boolean>(false);
  // Remove separate dashboard overlays - everything will be integrated into HUD tabs
  const [mcpAuthenticated, setMcpAuthenticated] = useState<boolean>(false);
  const [mcpHealthStatus, setMcpHealthStatus] = useState<any>(null);
  const [nullviewState, setNulleyeState] = useState<
    | 'base'
    | 'response'
    | 'question'
    | 'thinking'
    | 'alert'
    | 'error'
    | 'warning'
    | 'success'
    | 'processing'
    | 'idle'
  >('base');
  const [showHecateHud, setShowHecateHud] = useState<boolean>(false);
  const [hecateHudActiveTab, setHecateHudActiveTab] = useState<
    'tasks' | 'mcp' | 'logs' | 'agents' | 'hecate'
  >('hecate');
  const [mainHudActiveTab, setMainHudActiveTab] = useState<
    'status' | 'tasks' | 'agents' | 'mcp' | 'logs' | 'hecate'
  >('status');

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
        description:
          'Direct connection to the Ember network, enabling secure communication and data transfer.',
        stats: [
          {
            label: 'Connection Status',
            value: emberLinkStatus.connected ? 'Connected' : 'Disconnected',
            status: emberLinkStatus.connected ? 'active' : 'inactive',
          },
          {
            label: 'Last Seen',
            value: emberLinkStatus.lastSeen
              ? new Date(emberLinkStatus.lastSeen).toLocaleString()
              : 'Never',
          },
          {
            label: 'Browser',
            value: emberLinkStatus.browserInfo
              ? `${emberLinkStatus.browserInfo.name} ${emberLinkStatus.browserInfo.version}`
              : 'Unknown',
          },
          {
            label: 'Platform',
            value: emberLinkStatus.browserInfo?.platform || 'Unknown',
          },
        ],
      },
    },
    {
      id: 'neural',
      name: 'Neural Link',
      status: 'inactive',
      icon: '🧠',
      details: {
        description:
          'Advanced neural interface for enhanced cognitive processing and system interaction.',
        stats: [
          {
            label: 'Status',
            value: 'LOCKED',
            status: 'inactive',
          },
          {
            label: 'Signal Strength',
            value: 'N/A',
          },
          {
            label: 'Latency',
            value: 'N/A',
          },
        ],
      },
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
            status: 'inactive',
          },
          {
            label: 'Last Transaction',
            value: 'N/A',
          },
          {
            label: 'Security Level',
            value: 'N/A',
          },
        ],
      },
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
            status: 'inactive',
          },
          {
            label: 'Scan Progress',
            value: 'N/A',
          },
          {
            label: 'Last Update',
            value: 'N/A',
          },
        ],
      },
    },
  ];

  // Add mock leaderboard data
  const leaderboardData: LeaderboardEntry[] = [
    {
      id: 'PervySage',
      rank: 1,
      ascent: 999,
      nether: 999999,
      cacheValue: 999999,
      memories: 999,
      matrix: {
        level: 'ARCHITECT',
        rarity: 'MYTHICAL',
        status: 'FLAME KEEPER',
      },
    },
    {
      id: 'HUD-001',
      rank: 2,
      ascent: 5,
      nether: 1500,
      cacheValue: 2500,
      memories: 12,
      matrix: {
        level: 'MASTER',
        rarity: 'LEGENDARY',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-002',
      rank: 3,
      ascent: 4,
      nether: 1200,
      cacheValue: 2000,
      memories: 10,
      matrix: {
        level: 'EXPERT',
        rarity: 'EPIC',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-003',
      rank: 4,
      ascent: 3,
      nether: 900,
      cacheValue: 1500,
      memories: 8,
      matrix: {
        level: 'ADVANCED',
        rarity: 'RARE',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-004',
      rank: 5,
      ascent: 2,
      nether: 600,
      cacheValue: 1000,
      memories: 6,
      matrix: {
        level: 'INTERMEDIATE',
        rarity: 'UNCOMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-005',
      rank: 6,
      ascent: 1,
      nether: 300,
      cacheValue: 500,
      memories: 4,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-006',
      rank: 7,
      ascent: 1,
      nether: 250,
      cacheValue: 400,
      memories: 3,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-007',
      rank: 8,
      ascent: 1,
      nether: 200,
      cacheValue: 300,
      memories: 2,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-008',
      rank: 9,
      ascent: 1,
      nether: 150,
      cacheValue: 200,
      memories: 1,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-009',
      rank: 10,
      ascent: 1,
      nether: 100,
      cacheValue: 150,
      memories: 1,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-010',
      rank: 11,
      ascent: 1,
      nether: 50,
      cacheValue: 100,
      memories: 1,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
    {
      id: 'HUD-011',
      rank: 12,
      ascent: 1,
      nether: 25,
      cacheValue: 50,
      memories: 1,
      matrix: {
        level: 'BEGINNER',
        rarity: 'COMMON',
        status: 'LAST FLAME',
      },
    },
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
    { name: 'Neural Link', status: 'SCANNING', locked: false },
    { name: 'Wallet Health', status: 'OPTIMAL', locked: false },
    { name: 'Token Analysis', status: 'IN PROGRESS', locked: false },
    { name: 'Risk Assessment', status: 'LOW', locked: true },
    { name: 'Memory Integrity', status: 'CHECKING', locked: true },
    { name: 'Network Status', status: 'CONNECTED', locked: true },
    { name: 'Matrix Sync', status: 'OFFLINE', locked: true },
    { name: 'Reality Engine', status: 'DORMANT', locked: true },
    { name: 'Core Systems', status: 'LOCKED', locked: true },
    { name: 'Neural Cache', status: 'UNAVAILABLE', locked: true },
    { name: 'Quantum Resonance', status: 'UNKNOWN', locked: true },
    { name: 'Bio-Interface', status: 'DISABLED', locked: true },
    { name: 'Temporal Alignment', status: 'DESYNCED', locked: true },
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
          // Skip old backend wallet data fetch for now - using Erebus for wallet ops
          console.log('Wallet connected:', publicKey);
          
          // Try to resolve wallet name
          const walletType = localStorage.getItem('walletType');
          if (walletType) {
            // First check if we have a saved name
            const savedName = localStorage.getItem(`walletName_${publicKey}`);
            if (savedName) {
              setWalletName(savedName);
            } else {
              // Try to resolve automatically
              await resolveWalletName(publicKey, walletType);
            }
          }
          
          // TODO: Update to use Erebus wallet data endpoints when available
          // const data = await fetchWalletData(publicKey);
          // setWalletData(data);

          // Skip old backend profile fetch for now - using Erebus for wallet ops
          // TODO: Update to use Erebus user profile endpoints when available
          // try {
          //   const profileData = await fetchUserProfile(publicKey);

          //   // Check if the wallet has Nectar tokens
          //   const hasNectarToken = profileData.active_tokens.includes('NECTAR');

          //   // Update user profile with wallet data and username if available
          //   setUserProfile((prev) => ({
          //     ...prev,
          //     nether: hasNectarToken ? data.balance : null,
          //     cacheValue: data.balance || 0, // Set cache value to wallet balance
          //     id: profileData.username
          //       ? `@${profileData.username}`
          //       : `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol`,
          //   }));

          //   // Log the profile data to debug
          //   console.log('Profile data received:', profileData);
          //   console.log('Username:', profileData.username);
          // } catch (profileError) {
          //   console.error('Failed to fetch user profile:', profileError);
          //   // Fallback to just updating with wallet data
          //   setUserProfile((prev) => ({
          //     ...prev,
          //     nether: null, // Set to null if we can't determine if Nectar exists
          //     cacheValue: data.balance || 0, // Set cache value to wallet balance
          //   }));
          // }

          // Skip old backend ascent fetch for now - using Erebus for wallet ops
          // TODO: Update to use Erebus ascent endpoints when available
          // try {
          //   const ascentData = await fetchAscentLevel(publicKey);
          //   // Convert AscentLevelData to AscentLevel
          //   setAscentLevel({
          //     level: ascentData.level,
          //     title: ascentData.name,
          //     description: ascentData.description,
          //     progress: ascentData.progress,
          //     nextLevel: ascentData.level + 1,
          //     nextTitle: `Level ${ascentData.level + 1}`,
          //     nextDescription: 'Next level description will be available soon.',
          //     accolades: ascentData.accolades,
          //   });
          //   // Update the ascent value in userProfile
          //   setUserProfile((prev) => ({
          //     ...prev,
          //     ascent: ascentData.level,
          //   }));
          // } catch (ascentError) {
          //   console.error('Failed to fetch ascent level:', ascentError);
          // }

          // Skip old backend mission fetch for now - using Erebus for wallet ops
          // TODO: Update to use Erebus mission endpoints when available
          // try {
          //   const missionData = await fetchActiveMission(publicKey);
          //   setActiveMission(missionData);
          // } catch (missionError) {
          //   console.error('Failed to fetch active mission:', missionError);
          // }
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
          platform: 'Linux',
        },
      };

      // Simulate periodic updates
      const interval = setInterval(() => {
        setEmberLinkStatus((prev) => ({
          ...prev,
          lastSeen: new Date(),
        }));
      }, 30000); // Update every 30 seconds

      return () => clearInterval(interval);
    };

    return setupEmberLinkConnection();
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
            'phantom',
          );

          if (authResponse.success) {
            setMcpAuthenticated(true);
            alert('Successfully authenticated with MCP!');
          } else {
            alert(`Authentication failed: ${authResponse.message}`);
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
      <button className={styles.nullblockTitleButton} onClick={() => setScreen('home')}>
        NULLBLOCK
      </button>

      {/* NULLVIEWS - Living system indicator */}
      <div
        className={`${styles.nullview} ${styles[nullviewState]}`}
        onClick={() => {
          if (!publicKey) {
            // Enhanced feedback for locked state
            setNulleyeState('error');
            setTimeout(() => setNulleyeState('base'), 1500);
            alert(
              '🔒 SECURE ACCESS REQUIRED\n\nConnect your Web3 wallet to unlock the NullView interface and access advanced features.',
            );

            return;
          }

          // Navigate to hecate tab in main HUD instead of showing overlay
          setMainHudActiveTab('hecate');
          setNulleyeState('processing');
        }}
        title={!publicKey ? '🔒 Connect wallet to unlock NullView' : '🔓 Access NullView Interface'}
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

      <div className={`${styles.screenLabel} ${screen === 'home' ? styles.centeredLabel : ''}`}>
        {SCREEN_LABELS[screen]}
      </div>
      <div className={styles.navbarButtons}>
        <button
          className={`${styles.walletMenuButton} ${publicKey ? styles.connected : ''}`}
          onClick={publicKey ? onDisconnect : () => onConnectWallet()}
          title={publicKey ? 'Disconnect Wallet' : 'Connect Wallet'}
        >
          <span className={styles.walletMenuText}>{publicKey ? 'Disconnect' : 'Connect'}</span>
        </button>
        <button
          className={styles.docsMenuButton}
          onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
          title="Documentation & Developer Resources"
        >
          <span className={styles.docsMenuText}>Docs</span>
        </button>
      </div>
    </nav>
  );

  const renderUserProfile = () => (
    <div className={styles.userProfile}>
      {publicKey && (
        <>
          <div className={styles.profileItem}>
            <span className={styles.label}>NAME:</span>
            {isEditingName ? (
              <input
                type="text"
                placeholder="Enter wallet name"
                className={styles.nameInput}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    setCustomWalletName(e.currentTarget.value);
                  } else if (e.key === 'Escape') {
                    setIsEditingName(false);
                  }
                }}
                onBlur={(e) => setCustomWalletName(e.target.value)}
                autoFocus
              />
            ) : (
              <span 
                className={styles.value}
                onClick={() => setIsEditingName(true)}
                style={{ cursor: 'pointer', textDecoration: walletName ? 'none' : 'underline' }}
                title="Click to set wallet name"
              >
                {walletName ? `@${walletName}` : 'Set Name'}
              </span>
            )}
          </div>
          <div className={styles.profileItem}>
            <span className={styles.label}>WALLET:</span>
            <span className={styles.value}>
              {localStorage.getItem('walletType')?.toUpperCase() || 'UNKNOWN'}
            </span>
          </div>
          <div className={styles.profileItem}>
            <span className={styles.label}>ADDRESS:</span>
            <span className={styles.value}>
              {publicKey.slice(0, 6)}...{publicKey.slice(-4)}
            </span>
          </div>
        </>
      )}
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
            <div className={styles.progressFill} style={{ width: `${35}%` }}></div>
          </div>
        </div>
        {showAscentDetails && (
          <div className={styles.ascentDetails}>
            <div className={styles.ascentDescription}>
              A digital lurker extraordinaire! You've mastered the art of watching from the shadows,
              observing the chaos without ever dipping your toes in. Like a cat watching a laser
              pointer, you're fascinated but paralyzed by indecision. At least you're not the one
              getting your digital assets rekt!
            </div>
            <div className={styles.progressText}>35% to next level</div>
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
              NETHER: Magic internet money from the void. Born from nothing, worth everything, and
              somehow gaining value by the second. The integration has passed the event horizon -
              good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion
              and an irresistible urge to dive deeper into the code.
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
              Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement
              agents. This is the total worth of all valuable assets in your wallet - coins, tokens,
              and other digital goodies that caught our eye. Coming soon: Categories for services,
              participant offerings, biological enhancements, and agent capabilities. Think of it as
              your personal inventory of everything worth something in the Nullblock universe. Don't
              spend it all in one place!
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
              Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah,
              right - another poor...soul. You need to collect the artifacts that tell your story in
              the Nullblock universe. Each memory is a unique representation of your achievements,
              collectibles, and digital identity. Collect them all to unlock the secret of why
              you're here... or don't, I'm not your digital conscience.
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
              HUD Interface: A direct neural link to the Nullblock systems. This advanced technology
              allows real-time monitoring of network activities, asset management, and system
              diagnostics. Through the HUD Interface, users can access mission briefings, track
              progress, and interface with various subsystems. Warning: Extended use may result in
              enhanced situational awareness.
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
        <p>Access the NullView for further instructions.</p>
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

  const renderHudTab = () => (
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
              <span className={styles.browserValue}>
                {emberLinkStatus.browserInfo?.name} {emberLinkStatus.browserInfo?.version} (
                {emberLinkStatus.browserInfo?.platform})
              </span>
            </div>
          </div>
          <div className={styles.echoMessage}>
            <p>H.U.D system is active and operational.</p>
            <p>Interface ready for autonomous agents.</p>
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
            <p>
              To establish a secure connection, you need to install the Aether browser extension.
            </p>
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
                            value: `#${entry.rank}`,
                          },
                          {
                            label: 'Ascent',
                            value: entry.ascent,
                          },
                          {
                            label: 'Nether',
                            value: `₦ ${entry.nether}`,
                          },
                          {
                            label: 'Cache Value',
                            value: `₦ ${entry.cacheValue}`,
                          },
                          {
                            label: 'Memories',
                            value: entry.memories,
                          },
                          {
                            label: 'Matrix Level',
                            value: entry.matrix.level,
                          },
                          {
                            label: 'Rarity',
                            value: entry.matrix.rarity,
                          },
                          {
                            label: 'Status',
                            value: entry.matrix.status,
                          },
                        ],
                      },
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
                            value: `#${entry.rank}`,
                          },
                          {
                            label: 'Ascent',
                            value: entry.ascent,
                          },
                          {
                            label: 'Nether',
                            value: `₦ ${entry.nether}`,
                          },
                          {
                            label: 'Cache Value',
                            value: `₦ ${entry.cacheValue}`,
                          },
                          {
                            label: 'Memories',
                            value: entry.memories,
                          },
                          {
                            label: 'Matrix Level',
                            value: entry.matrix.level,
                          },
                          {
                            label: 'Rarity',
                            value: entry.matrix.rarity,
                          },
                          {
                            label: 'Status',
                            value: entry.matrix.status,
                          },
                        ],
                      },
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
                  <div
                    className={styles.diagnosticsItem}
                    onClick={() =>
                      setSelectedUplink({
                        id: 'id',
                        name: 'ID',
                        status: 'active',
                        icon: '🆔',
                        details: {
                          description:
                            'Your unique identifier in the Nullblock universe. This is your digital fingerprint, your signature in the void.',
                          stats: [
                            {
                              label: 'Status',
                              value: 'Active',
                            },
                            {
                              label: 'Type',
                              value: 'HUD ID',
                            },
                          ],
                        },
                      })}
                  >
                    <span className={styles.itemLabel}>🆔 ID</span>
                    <span className={styles.itemValue}>HUD-{userProfile.id || '0000'}</span>
                  </div>
                  <div
                    className={styles.diagnosticsItem}
                    onClick={() =>
                      setSelectedUplink({
                        id: 'ascent',
                        name: 'ASCENT',
                        status: 'active',
                        icon: '↗️',
                        details: {
                          description:
                            "A digital lurker extraordinaire! You've mastered the art of watching from the shadows, observing the chaos without ever dipping your toes in. Like a cat watching a laser pointer, you're fascinated but paralyzed by indecision. At least you're not the one getting your digital assets rekt!",
                          stats: [
                            {
                              label: 'Level',
                              value: 'Net Dweller: 1',
                            },
                            {
                              label: 'Progress',
                              value: '35%',
                            },
                          ],
                        },
                      })}
                  >
                    <span className={styles.itemLabel}>
                      <span className={styles.ascentLine}></span> ASCENT
                    </span>
                    <span className={styles.itemValue}>Net Dweller: 1</span>
                  </div>
                  <div
                    className={styles.diagnosticsItem}
                    onClick={() =>
                      setSelectedUplink({
                        id: 'nether',
                        name: 'NETHER',
                        status: 'active',
                        icon: '₦',
                        details: {
                          description:
                            'NETHER: Magic internet money from the void. Born from nothing, worth everything, and somehow gaining value by the second. The integration has passed the event horizon - good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion and an irresistible urge to dive deeper into the code.',
                          stats: [
                            {
                              label: 'Balance',
                              value: `₦ ${userProfile.nether?.toFixed(2) || 'N/A'}`,
                            },
                            {
                              label: 'Status',
                              value: userProfile.nether ? 'Active' : 'Inactive',
                            },
                          ],
                        },
                      })}
                  >
                    <span className={styles.itemLabel}>₦ NETHER</span>
                    <span className={styles.itemValue}>
                      ₦ {userProfile.nether?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                  <div
                    className={styles.diagnosticsItem}
                    onClick={() =>
                      setSelectedUplink({
                        id: 'cache',
                        name: 'CACHE VALUE',
                        status: 'active',
                        icon: '💰',
                        details: {
                          description:
                            "Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement agents. This is the total worth of all valuable assets in your wallet - coins, tokens, and other digital goodies that caught our eye. Coming soon: Categories for services, participant offerings, biological enhancements, and agent capabilities. Think of it as your personal inventory of everything worth something in the Nullblock universe. Don't spend it all in one place!",
                          stats: [
                            {
                              label: 'Value',
                              value: '₦ N/A',
                            },
                            {
                              label: 'Status',
                              value: 'Pending',
                            },
                          ],
                        },
                      })}
                  >
                    <span className={styles.itemLabel}>💰 CACHE VALUE</span>
                    <span className={styles.itemValue}>₦ N/A</span>
                  </div>
                  <div
                    className={styles.diagnosticsItem}
                    onClick={() =>
                      setSelectedUplink({
                        id: 'memories',
                        name: 'MEMORIES',
                        status: 'active',
                        icon: '🧠',
                        details: {
                          description:
                            "Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah, right - another poor...soul. You need to collect the artifacts that tell your story in the Nullblock universe. Each memory is a unique representation of your achievements, collectibles, and digital identity. Collect them all to unlock the secret of why you're here... or don't, I'm not your digital conscience.",
                          stats: [
                            {
                              label: 'Count',
                              value: userProfile.memories,
                            },
                            {
                              label: 'Status',
                              value: userProfile.memories > 0 ? 'Active' : 'Empty',
                            },
                          ],
                        },
                      })}
                  >
                    <span className={styles.itemLabel}>🧠 MEMORIES</span>
                    <span className={styles.itemValue}>{userProfile.memories}</span>
                  </div>
                  <div
                    className={styles.diagnosticsItem}
                    onClick={() =>
                      setSelectedUplink({
                        id: 'hud',
                        name: 'HUD INTERFACE',
                        status: 'active',
                        icon: '📊',
                        details: {
                          description:
                            'HUD Interface: A direct neural link to the Nullblock systems. This advanced technology allows real-time monitoring of network activities, asset management, and system diagnostics. Through the HUD Interface, users can access mission briefings, track progress, and interface with various subsystems. Warning: Extended use may result in enhanced situational awareness.',
                          stats: [
                            {
                              label: 'Status',
                              value: userProfile.matrix.status,
                            },
                            {
                              label: 'Type',
                              value: 'HUD Interface',
                            },
                          ],
                        },
                      })}
                  >
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
                            <span
                              className={
                                mcpHealthStatus.status === 'operational'
                                  ? styles.active
                                  : styles.inactive
                              }
                            >
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
                {activeTab === 'portfolio' && (
                  <div className={styles.portfolioTab}>
                    <div className={styles.portfolioHeader}>
                      <h3>MCP Portfolio Management</h3>
                      {mcpHealthStatus && (
                        <div className={styles.mcpStatus}>
                          <div className={styles.statusContainer}>
                            <span className={styles.statusLabel}>MCP Status:</span>
                            <span
                              className={
                                mcpHealthStatus.status === 'operational'
                                  ? styles.active
                                  : styles.inactive
                              }
                            >
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
                            <li>✓ Social sentiment integration</li>
                            <li>✓ Community-driven trading signals</li>
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
                            <span
                              className={
                                mcpHealthStatus.status === 'operational'
                                  ? styles.active
                                  : styles.inactive
                              }
                            >
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
                        <p>
                          DeFi automation system ready. Optimize yield and manage liquidity
                          positions.
                        </p>
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
                      <p>Access the NullView for further instructions.</p>
                    </div>
                  </div>
                )}
                {activeTab === 'defense' && (
                  <div className={styles.defenseTab}>
                    <div className={styles.lockedContent}>
                      <p>This feature is currently locked.</p>
                      <p>Access the NullView for further instructions.</p>
                    </div>
                  </div>
                )}
                {activeTab === 'missions' && (
                  <div className={styles.missionsTab}>
                    <div className={styles.missionHeader}>
                      <div className={styles.active}>
                        <span className={styles.missionLabel}>ACTIVE:</span>
                        <span className={styles.missionTitle}>
                          {activeMission?.title || 'Share on X'}
                        </span>
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
                            <span className={`${styles.missionReward} ${styles.blurred}`}>
                              ??? NETHER
                            </span>
                          </div>
                          <div className={`${styles.missionItem} ${styles.blurred}`}>
                            <div className={styles.missionItemContent}>
                              <span className={styles.missionTitle}>Mission 3</span>
                              <span className={styles.missionStatus}>LOCKED</span>
                            </div>
                            <span className={`${styles.missionReward} ${styles.blurred}`}>
                              ??? NETHER
                            </span>
                          </div>
                        </div>
                      </div>

                      <div className={styles.missionDescription}>
                        <h4>MISSION BRIEF</h4>
                        <p className={styles.missionText}>
                          "Welcome, Agent, to your first trial. Interface with the HUD carefully.
                          Share your NullView interface on X—let its glow haunt the realm. More souls
                          drawn, more NETHER gained. Don't let it fade."
                        </p>
                        <div className={styles.missionInstructions}>
                          <h4>QUALIFICATION REQUIREMENTS</h4>
                          <ul>
                            <li>
                              Follow<span className={styles.highlight}>@Nullblock_io</span>
                            </li>
                            <li>
                              Tweet out the cashtag{' '}
                              <span className={styles.highlight}>$NETHER</span>
                            </li>
                            <li>
                              Include the official CA: <span className={styles.highlight}>TBD</span>
                            </li>
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
              <button className={styles.closeButton} onClick={handleCloseModal}>
                ×
              </button>
            </div>
            <div className={styles.modalContent}>
              <div className={styles.statusSection}>
                <h4>Status Overview</h4>
                <div className={styles.statusGrid}>
                  {selectedUplink.details.stats.map((stat, index) => (
                    <div key={index} className={styles.statusItem}>
                      <div className={styles.label}>{stat.label}</div>
                      <div className={`${styles.value} ${stat.status || ''}`}>{stat.value}</div>
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
              <button className={styles.closeButton} onClick={handleCloseLeaderboard}>
                ×
              </button>
            </div>
            <div className={styles.legendWarning}>
              <p>
                ⚠️ These are the legends of the last Interface. The current HUD has yet to awaken.
              </p>
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
          <p>Access the NullView to acquire gear.</p>
        </div>
      </div>
      <div className={styles.inventorySection}>
        <h3>SUPPLIES</h3>
        <div className={styles.emptyState}>
          <p>Cache empty.</p>
          <p>Access the NullView to expand inventory.</p>
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
          <p>
            Current Level: <span>1</span>
          </p>
          <p>
            Completion: <span>0%</span>
          </p>
        </div>
        <div className={styles.missions}>
          <h3>OBJECTIVES</h3>
          <p className={styles.placeholder}>No active missions</p>
          <p className={styles.placeholder}>Access the NullView to begin</p>
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
          <p>
            Phantom: <span className={styles.connected}>CONNECTED</span>
          </p>
          <p>
            Core: <span className={styles.initializing}>INITIALIZING</span>
          </p>
        </div>
        <div className={styles.interfaceSection}>
          <h3>CONFIGURATIONS</h3>
          <p className={styles.placeholder}>No active modifications</p>
          <p className={styles.placeholder}>Access the NullView to begin</p>
        </div>
      </div>
    </div>
  );

  // Add complete HecateHud state and functionality
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([
    {
      id: '1',
      timestamp: new Date(Date.now() - 300000),
      sender: 'hecate',
      message: 'Welcome to NullBlock! I\'m here to help you navigate the agentic ecosystem. What would you like to explore today?',
      type: 'text'
    }
  ]);
  const [chatInput, setChatInput] = useState('');
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [activeLens, setActiveLens] = useState<string | null>(null);
  const [agentConnected, setAgentConnected] = useState(false);
  const [currentModel, setCurrentModel] = useState<string | null>(null);
  const [isConnectingAgent, setIsConnectingAgent] = useState(false);

  // Initialize Hecate agent connection
  useEffect(() => {
    const initializeHecateAgent = async () => {
      try {
        setIsConnectingAgent(true);
        const connected = await hecateAgent.connect();
        setAgentConnected(connected);
        
        if (connected) {
          // Get initial model status
          const modelStatus = await hecateAgent.getModelStatus();
          setCurrentModel(modelStatus.current_model);
          console.log('Hecate agent connected successfully');
        } else {
          console.warn('Failed to connect to Hecate agent');
        }
      } catch (error) {
        console.error('Error connecting to Hecate agent:', error);
        setAgentConnected(false);
      } finally {
        setIsConnectingAgent(false);
      }
    };

    initializeHecateAgent();
  }, []);

  // Update avatar state based on agent connection
  useEffect(() => {
    if (isConnectingAgent) {
      setNulleyeState('processing');
    } else if (agentConnected) {
      setNulleyeState('base');
    } else {
      setNulleyeState('idle');
    }
  }, [agentConnected, isConnectingAgent]);

  // Define lens options (scopes)
  const lensOptions: LensOption[] = [
    {
      id: 'templates',
      icon: '📋',
      title: 'Templates',
      description: 'Task templates',
      color: '#00a8ff'
    },
    {
      id: 'workflow',
      icon: '🔗',
      title: 'Workflow',
      description: 'Workflow automation',
      color: '#b967ff'
    },
    {
      id: 'suggestions',
      icon: '💡',
      title: 'Suggestions',
      description: 'AI suggestions',
      color: '#ff7f00'
    },
    {
      id: 'visualizer',
      icon: '📊',
      title: 'Visualizer',
      description: 'Data visualization',
      color: '#00ff9d'
    },
    {
      id: 'sandbox',
      icon: '⚡',
      title: 'Sandbox',
      description: 'Code playground',
      color: '#e6c200'
    },
    {
      id: 'voice',
      icon: '🎤',
      title: 'Voice',
      description: 'Voice interface',
      color: '#ff3333'
    },
    {
      id: 'settings',
      icon: '⚙️',
      title: 'Settings',
      description: 'Theme & preferences',
      color: '#888888'
    }
  ];

  interface ChatMessage {
    id: string;
    timestamp: Date;
    sender: 'user' | 'hecate';
    message: string;
    type?: 'text' | 'update' | 'question' | 'suggestion';
    model_used?: string;
    metadata?: any;
  }

  interface LensOption {
    id: string;
    icon: string;
    title: string;
    description: string;
    color: string;
    expanded?: boolean;
  }

  const handleChatSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!chatInput.trim()) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString() + '-user',
      timestamp: new Date(),
      sender: 'user',
      message: chatInput,
      type: 'text'
    };

    setChatMessages(prev => [...prev, userMessage]);
    const currentInput = chatInput;
    setChatInput('');

    // Get agent response if connected
    if (agentConnected) {
      try {
        // Prepare user context
        const userContext = {
          wallet_address: publicKey || undefined,
          wallet_type: localStorage.getItem('walletType') || undefined,
          session_time: publicKey ? `${Math.floor((Date.now() - parseInt(localStorage.getItem('lastAuthTime') || '0')) / 60000)}m` : undefined
        };

        const response: HecateResponse = await hecateAgent.sendMessage(currentInput, userContext);
        
        // Update current model
        setCurrentModel(response.model_used);
        
        const hecateResponse: ChatMessage = {
          id: Date.now().toString() + '-hecate',
          timestamp: new Date(),
          sender: 'hecate',
          message: response.content,
          type: 'text',
          model_used: response.model_used,
          metadata: response.metadata
        };
        
        setChatMessages(prev => [...prev, hecateResponse]);
      } catch (error) {
        console.error('Failed to get Hecate response:', error);
        
        // Fallback response
        const errorResponse: ChatMessage = {
          id: Date.now().toString() + '-hecate',
          timestamp: new Date(),
          sender: 'hecate',
          message: `I'm having trouble connecting to my backend systems. Please ensure the Hecate agent server is running or try again in a moment.`,
          type: 'text',
          model_used: 'error'
        };
        
        setChatMessages(prev => [...prev, errorResponse]);
      }
    } else {
      // Fallback when agent not connected
      const fallbackResponse: ChatMessage = {
        id: Date.now().toString() + '-hecate',
        timestamp: new Date(),
        sender: 'hecate',
        message: `Agent offline. I understand you're asking about "${currentInput}". Please start the Hecate agent server to enable full functionality.`,
        type: 'text',
        model_used: 'offline'
      };
      
      setChatMessages(prev => [...prev, fallbackResponse]);
    }
  };

  const handleChatInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setChatInput(e.target.value);
    setShowSuggestions(e.target.value.length > 0 && e.target.value.length < 3);
  };

  const handleSuggestionClick = (suggestion: string) => {
    setChatInput(suggestion);
    setShowSuggestions(false);
  };

  const handleLensClick = (lensId: string) => {
    setActiveLens(activeLens === lensId ? null : lensId);
  };

  const renderLensContent = (lensId: string) => {
    switch (lensId) {
      case 'templates':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>📋 Task Templates</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.templateGrid}>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>🐍</div>
                <h6>Python Script</h6>
                <p>Generate Python code for automation</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>📊</div>
                <h6>Data Analysis</h6>
                <p>Analyze CSV data with visualizations</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>📝</div>
                <h6>Content Writer</h6>
                <p>Create blog posts and articles</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>🤖</div>
                <h6>Bot Builder</h6>
                <p>Create automated workflows</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
            </div>
          </div>
        );

      case 'workflow':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🔗 Workflow Builder</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.workflowCanvas}>
              <div className={styles.workflowNode}>
                <div className={styles.nodeIcon}>📥</div>
                <span>Input Data</span>
              </div>
              <div className={styles.workflowArrow}>→</div>
              <div className={styles.workflowNode}>
                <div className={styles.nodeIcon}>⚙️</div>
                <span>Process</span>
              </div>
              <div className={styles.workflowArrow}>→</div>
              <div className={styles.workflowNode}>
                <div className={styles.nodeIcon}>📤</div>
                <span>Output</span>
              </div>
            </div>
            <div className={styles.workflowControls}>
              <button className={styles.workflowBtn}>Add Node</button>
              <button className={styles.workflowBtn}>Save Workflow</button>
              <button className={styles.workflowBtn}>Run</button>
            </div>
          </div>
        );

      case 'suggestions':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>💡 AI Suggestions</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.lensPlaceholder}>
              <p>This lens feature is coming soon...</p>
              <button className={styles.comingSoonBtn}>Notify When Ready</button>
            </div>
          </div>
        );

      case 'visualizer':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>📊 Data Visualizer</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.visualizerCanvas}>
              <div className={styles.chartPlaceholder}>
                <div className={styles.chartBar} style={{ height: '60%' }}></div>
                <div className={styles.chartBar} style={{ height: '80%' }}></div>
                <div className={styles.chartBar} style={{ height: '40%' }}></div>
                <div className={styles.chartBar} style={{ height: '90%' }}></div>
                <div className={styles.chartBar} style={{ height: '70%' }}></div>
              </div>
            </div>
            <div className={styles.visualizerControls}>
              <button className={styles.vizBtn}>Bar Chart</button>
              <button className={styles.vizBtn}>Line Chart</button>
              <button className={styles.vizBtn}>Pie Chart</button>
              <button className={styles.vizBtn}>Export</button>
            </div>
          </div>
        );

      case 'sandbox':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>⚡ Code Sandbox</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.codeEditor}>
              <div className={styles.editorHeader}>
                <span>main.py</span>
                <div className={styles.editorControls}>
                  <button className={styles.editorBtn}>Run</button>
                  <button className={styles.editorBtn}>Save</button>
                </div>
              </div>
              <textarea
                className={styles.codeTextarea}
                placeholder="print('Hello, World!')"
                defaultValue="import requests&#10;&#10;response = requests.get('https://api.example.com/data')&#10;print(response.json())"
              />
            </div>
            <div className={styles.outputPanel}>
              <h6>Output:</h6>
              <div className={styles.outputContent}>
                <span className={styles.outputLine}>Running code...</span>
                <span className={styles.outputLine}>Data fetched successfully</span>
              </div>
            </div>
          </div>
        );

      case 'voice':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🎤 Voice Controls</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.lensPlaceholder}>
              <p>This lens feature is coming soon...</p>
              <button className={styles.comingSoonBtn}>Notify When Ready</button>
            </div>
          </div>
        );

      case 'analytics':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>📈 Analytics Dashboard</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.analyticsGrid}>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>📊</div>
                <h6>Performance</h6>
                <div className={styles.analyticsValue}>98.5%</div>
              </div>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>⚡</div>
                <h6>Speed</h6>
                <div className={styles.analyticsValue}>2.3s</div>
              </div>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>👥</div>
                <h6>Users</h6>
                <div className={styles.analyticsValue}>1,247</div>
              </div>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>🔄</div>
                <h6>Uptime</h6>
                <div className={styles.analyticsValue}>99.9%</div>
              </div>
            </div>
          </div>
        );

      case 'automation':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🤖 Automation Hub</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.automationList}>
              <div className={styles.automationItem}>
                <div className={styles.automationIcon}>📧</div>
                <div className={styles.automationInfo}>
                  <h6>Email Automation</h6>
                  <p>Auto-respond to customer inquiries</p>
                </div>
                <div className={styles.automationStatus}>Active</div>
              </div>
              <div className={styles.automationItem}>
                <div className={styles.automationIcon}>📊</div>
                <div className={styles.automationInfo}>
                  <h6>Data Sync</h6>
                  <p>Sync data across platforms</p>
                </div>
                <div className={styles.automationStatus}>Active</div>
              </div>
              <div className={styles.automationItem}>
                <div className={styles.automationIcon}>🔔</div>
                <div className={styles.automationInfo}>
                  <h6>Notifications</h6>
                  <p>Smart alert system</p>
                </div>
                <div className={styles.automationStatus}>Paused</div>
              </div>
            </div>
          </div>
        );

      case 'security':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🔒 Security Center</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.securityGrid}>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>🛡️</div>
                <h6>Access Control</h6>
                <p>Manage user permissions</p>
                <button className={styles.securityBtn}>Configure</button>
              </div>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>🔐</div>
                <h6>Encryption</h6>
                <p>Data protection settings</p>
                <button className={styles.securityBtn}>Settings</button>
              </div>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>📋</div>
                <h6>Audit Log</h6>
                <p>Activity monitoring</p>
                <button className={styles.securityBtn}>View Log</button>
              </div>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>🚨</div>
                <h6>Alerts</h6>
                <p>Security notifications</p>
                <button className={styles.securityBtn}>Configure</button>
              </div>
            </div>
          </div>
        );

      case 'integration':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🔌 Integration Hub</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.integrationList}>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>📊</div>
                <div className={styles.integrationInfo}>
                  <h6>Google Analytics</h6>
                  <p>Connected • Last sync: 2 min ago</p>
                </div>
                <div className={styles.integrationStatus}>Active</div>
              </div>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>💳</div>
                <div className={styles.integrationInfo}>
                  <h6>Stripe</h6>
                  <p>Connected • Last sync: 5 min ago</p>
                </div>
                <div className={styles.integrationStatus}>Active</div>
              </div>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>📧</div>
                <div className={styles.integrationInfo}>
                  <h6>Mailchimp</h6>
                  <p>Disconnected</p>
                </div>
                <div className={styles.integrationStatus}>Inactive</div>
              </div>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>☁️</div>
                <div className={styles.integrationInfo}>
                  <h6>AWS S3</h6>
                  <p>Connected • Last sync: 1 min ago</p>
                </div>
                <div className={styles.integrationStatus}>Active</div>
              </div>
            </div>
          </div>
        );

      case 'settings':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>⚙️ Settings</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.settingsGrid}>
              {/* Theme Settings */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>🎨</div>
                <h6>Theme Settings</h6>
                <p>Customize interface appearance</p>
                <div className={styles.themeControls}>
                  <button
                    className={`${styles.themeControlBtn} ${theme === 'light' ? styles.active : ''}`}
                    onClick={() => onThemeChange && onThemeChange('light')}
                  >
                    ☀️ Light
                  </button>
                  <button
                    className={`${styles.themeControlBtn} ${theme === 'dark' ? styles.active : ''}`}
                    onClick={() => onThemeChange && onThemeChange('dark')}
                  >
                    🌙 Dark
                  </button>
                  <button
                    className={`${styles.themeControlBtn} ${theme === 'null' ? styles.active : ''}`}
                    onClick={() => onThemeChange && onThemeChange('cyber')}
                  >
                    ⚡ Cyber
                  </button>
                </div>
              </div>
              
              {/* Social Links */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>🌍</div>
                <h6>Social Links</h6>
                <p>Connect with the community</p>
                <div className={styles.socialLinks}>
                  <a
                    href="https://x.com/Nullblock_io"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.socialLinkBtn}
                  >
                    𝕏 Follow on X
                  </a>
                  <a
                    href="https://discord.gg/nullblock"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.socialLinkBtn}
                  >
                    🎮 Discord
                  </a>
                  <a
                    href="https://github.com/nullblock-io"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.socialLinkBtn}
                  >
                    💻 GitHub
                  </a>
                </div>
              </div>
              
              {/* Documentation */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>📚</div>
                <h6>Documentation</h6>
                <p>Learn about Nullblock features</p>
                <a
                  href="https://aetherbytes.github.io/nullblock-sdk/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.docsBtn}
                >
                  📚 View Docs
                </a>
              </div>
              
              {/* Version Info */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>📝</div>
                <h6>Version</h6>
                <p>Current build information</p>
                <div className={styles.versionInfo}>
                  <span>Nullblock v0.8.17</span>
                  <span>Build: Alpha</span>
                </div>
              </div>
            </div>
          </div>
        );

      default:
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>💡 {lensId.charAt(0).toUpperCase() + lensId.slice(1)}</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.lensPlaceholder}>
              <p>This lens feature is coming soon...</p>
              <button className={styles.comingSoonBtn}>Notify When Ready</button>
            </div>
          </div>
        );
    }
  };

  // Add mock data for the integrated content (from HecateHud)
  const [tasks, setTasks] = useState<Task[]>([
    {
      id: '1',
      name: 'Price Oracle Sync',
      status: 'running',
      type: 'mcp',
      description: 'Synchronizing price data across multiple DEXes',
      startTime: new Date(Date.now() - 300000),
      progress: 78,
      logs: []
    },
    {
      id: '2', 
      name: 'MEV Bundle Analysis',
      status: 'completed',
      type: 'trading',
      description: 'Analyzing MEV opportunities in pending transactions',
      startTime: new Date(Date.now() - 600000),
      endTime: new Date(Date.now() - 120000),
      logs: []
    }
  ]);

  const [mcpOperations, setMcpOperations] = useState<MCPOperation[]>([
    {
      id: '1',
      name: 'Wallet Authentication',
      status: 'active',
      endpoint: '/auth/verify',
      lastActivity: new Date(),
      responseTime: 120
    },
    {
      id: '2',
      name: 'Price Data Stream',
      status: 'active', 
      endpoint: '/data/prices',
      lastActivity: new Date(Date.now() - 30000),
      responseTime: 85
    }
  ]);

  const [logs, setLogs] = useState<LogEntry[]>([
    {
      id: '1',
      timestamp: new Date(),
      level: 'info',
      source: 'arbitrage.scanner.ts:45',
      message: 'DEX scan initiated',
      data: { dexes: ['Uniswap', 'SushiSwap', 'Curve'], pairs: 247, scanTime: '1.2s' }
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 60000),
      level: 'success',
      source: 'flashbots.client.ts:203',
      message: 'MEV bundle included in block',
      data: { blockNumber: 18945672, bundleHash: '0x4f5e6d...', profit: '$23.45' }
    }
  ]);

  const [searchTerm, setSearchTerm] = useState('');
  const [logFilter, setLogFilter] = useState<'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'>('all');
  const [autoScroll, setAutoScroll] = useState(true);

  // Add interfaces for the transferred types
  interface Task {
    id: string;
    name: string;
    status: 'running' | 'completed' | 'failed' | 'pending';
    type: 'mcp' | 'agent' | 'system' | 'trading';
    description: string;
    startTime: Date;
    endTime?: Date;
    progress?: number;
    logs: LogEntry[];
  }

  interface MCPOperation {
    id: string;
    name: string;
    status: 'active' | 'idle' | 'error';
    endpoint: string;
    lastActivity: Date;
    responseTime?: number;
  }

  interface LogEntry {
    id: string;
    timestamp: Date;
    level: 'info' | 'warning' | 'error' | 'success' | 'debug';
    source: string;
    message: string;
    data?: any;
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
      case 'active':
        return styles.statusRunning;
      case 'completed':
      case 'success':
        return styles.statusCompleted;
      case 'failed':
      case 'error':
        return styles.statusFailed;
      case 'pending':
      case 'idle':
        return styles.statusPending;
      default:
        return '';
    }
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return styles.logError;
      case 'warning':
        return styles.logWarning;
      case 'success':
        return styles.logSuccess;
      case 'debug':
        return styles.logDebug;
      default:
        return styles.logInfo;
    }
  };

  const filteredLogs = logs.filter((log) => {
    const matchesSearch = log.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         log.source.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesFilter = logFilter === 'all' || log.level === logFilter;
    return matchesSearch && matchesFilter;
  });

  const getUserStats = () => {
    return {
      walletName: walletName || 'Unknown',
      walletAddress: publicKey ? `${publicKey.slice(0, 6)}...${publicKey.slice(-4)}` : 'Not Connected',
      walletType: localStorage.getItem('walletType')?.toUpperCase() || 'UNKNOWN',
      sessionDuration: '15m 23s',
      connectionStatus: publicKey ? 'Connected' : 'Disconnected'
    };
  };

  const renderMainHudTabContent = () => {
    if (!publicKey && mainHudActiveTab !== 'status') {
      return (
        <div className={styles.mainHudContent}>
          <div className={styles.lockedContent}>
            <p>Connect your wallet to access this feature.</p>
          </div>
        </div>
      );
    }

    switch (mainHudActiveTab) {
      case 'status':
        return (
          <div className={styles.mainHudContent}>
            <div className={styles.statusContent}>
              <div className={styles.statusHeader}>
                <h3>System Status</h3>
              </div>
              
              {publicKey ? (
                <div className={styles.walletStatusSection}>
                  <h4>Wallet Information</h4>
                  <div className={styles.hecateStats}>
                    <div className={styles.statCard}>
                      <div className={styles.statValue}>
                        {getUserStats().walletName || getUserStats().walletAddress}
                      </div>
                      <div className={styles.statLabel}>Wallet Identity</div>
                    </div>
                    <div className={styles.statCard}>
                      <div className={styles.statValue}>{getUserStats().walletType}</div>
                      <div className={styles.statLabel}>Wallet Type</div>
                    </div>
                    <div className={styles.statCard}>
                      <div className={styles.statValue}>{getUserStats().sessionDuration}</div>
                      <div className={styles.statLabel}>Session Time</div>
                    </div>
                    <div className={styles.statCard}>
                      <div className={styles.statValue}>{getUserStats().connectionStatus}</div>
                      <div className={styles.statLabel}>Connection Status</div>
                    </div>
                  </div>
                </div>
              ) : (
                <div className={styles.placeholder}>
                  <p>Connect your wallet to view status information.</p>
                </div>
              )}
            </div>
          </div>
        );
      case 'tasks':
        return (
          <div className={styles.mainHudContent}>
            <div className={styles.tasksContainer}>
              <div className={styles.tasksHeader}>
                <h3>Active Tasks</h3>
                <div className={styles.taskStats}>
                  <span className={styles.stat}>
                    Running: {tasks.filter((t) => t.status === 'running').length}
                  </span>
                  <span className={styles.stat}>
                    Completed: {tasks.filter((t) => t.status === 'completed').length}
                  </span>
                  <span className={styles.stat}>
                    Failed: {tasks.filter((t) => t.status === 'failed').length}
                  </span>
                </div>
              </div>
              <div className={styles.tasksList}>
                {tasks.map((task) => (
                  <div key={task.id} className={`${styles.taskItem} ${getStatusColor(task.status)}`}>
                    <div className={styles.taskHeader}>
                      <div className={styles.taskInfo}>
                        <span className={styles.taskName}>{task.name}</span>
                        <span className={styles.taskType}>{task.type}</span>
                      </div>
                      <div className={styles.taskStatus}>
                        <span className={styles.statusDot}></span>
                        {task.status}
                      </div>
                    </div>
                    <div className={styles.taskDescription}>{task.description}</div>
                    {task.progress !== undefined && (
                      <div className={styles.progressBar}>
                        <div className={styles.progressFill} style={{ width: `${task.progress}%` }}></div>
                        <span className={styles.progressText}>{task.progress}%</span>
                      </div>
                    )}
                    <div className={styles.taskTiming}>
                      <span>Started: {task.startTime.toLocaleTimeString()}</span>
                      {task.endTime && <span>Ended: {task.endTime.toLocaleTimeString()}</span>}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      case 'agents':
        return (
          <div className={styles.mainHudContent}>
            <div className={styles.agentsContainer}>
              <div className={styles.agentsHeader}>
                <h3>Active Agents</h3>
              </div>
              <div className={styles.agentsList}>
                <div className={styles.agentItem}>
                  <div className={styles.agentInfo}>
                    <span className={styles.agentName}>Arbitrage Agent</span>
                    <span className={styles.agentStatus}>Active</span>
                  </div>
                  <div className={styles.agentMetrics}>
                    <span>Opportunities Found: 12</span>
                    <span>Executed Trades: 8</span>
                    <span>Success Rate: 92%</span>
                  </div>
                </div>
                <div className={styles.agentItem}>
                  <div className={styles.agentInfo}>
                    <span className={styles.agentName}>Social Trading Agent</span>
                    <span className={styles.agentStatus}>Active</span>
                  </div>
                  <div className={styles.agentMetrics}>
                    <span>Signals Generated: 45</span>
                    <span>Accuracy: 78%</span>
                    <span>Last Update: 2m ago</span>
                  </div>
                </div>
                <div className={styles.agentItem}>
                  <div className={styles.agentInfo}>
                    <span className={styles.agentName}>Portfolio Manager</span>
                    <span className={styles.agentStatus}>Idle</span>
                  </div>
                  <div className={styles.agentMetrics}>
                    <span>Rebalancing Events: 3</span>
                    <span>Risk Score: 0.23</span>
                    <span>Last Action: 15m ago</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        );
      case 'mcp':
        return (
          <div className={styles.mainHudContent}>
            <div className={styles.mcpContainer}>
              <div className={styles.mcpHeader}>
                <h3>MCP Operations</h3>
                <div className={styles.mcpStats}>
                  <span className={styles.stat}>
                    Active: {mcpOperations.filter((op) => op.status === 'active').length}
                  </span>
                  <span className={styles.stat}>
                    Idle: {mcpOperations.filter((op) => op.status === 'idle').length}
                  </span>
                </div>
              </div>
              <div className={styles.mcpList}>
                {mcpOperations.map((operation) => (
                  <div
                    key={operation.id}
                    className={`${styles.mcpItem} ${getStatusColor(operation.status)}`}
                  >
                    <div className={styles.mcpHeader}>
                      <span className={styles.mcpName}>{operation.name}</span>
                      <span className={styles.mcpStatus}>{operation.status}</span>
                    </div>
                    <div className={styles.mcpDetails}>
                      <span className={styles.mcpEndpoint}>{operation.endpoint}</span>
                      <span className={styles.mcpLastActivity}>
                        Last: {operation.lastActivity.toLocaleTimeString()}
                      </span>
                      {operation.responseTime && (
                        <span className={styles.mcpResponseTime}>Response: {operation.responseTime}ms</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      case 'logs':
        return (
          <div className={styles.mainHudContent}>
            <div className={styles.logsContainer}>
              <div className={styles.logsHeader}>
                <h3>System Logs</h3>
                <div className={styles.logsControls}>
                  <input
                    type="text"
                    placeholder="Search logs..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className={styles.searchInput}
                  />
                  <select
                    value={logFilter}
                    onChange={(e) => setLogFilter(e.target.value as any)}
                    className={styles.filterSelect}
                  >
                    <option value="all">All Levels</option>
                    <option value="info">Info</option>
                    <option value="warning">Warning</option>
                    <option value="error">Error</option>
                    <option value="success">Success</option>
                    <option value="debug">Debug</option>
                  </select>
                  <label className={styles.autoScrollLabel}>
                    <input
                      type="checkbox"
                      checked={autoScroll}
                      onChange={(e) => setAutoScroll(e.target.checked)}
                    />
                    Auto-scroll
                  </label>
                </div>
              </div>
              <div className={styles.logsContent}>
                {filteredLogs.map((log) => (
                  <div key={log.id} className={`${styles.logEntry} ${getLogLevelColor(log.level)}`}>
                    <div className={styles.logTimestamp}>{log.timestamp.toLocaleTimeString()}</div>
                    <div className={styles.logLevel}>{log.level.toUpperCase()}</div>
                    <div className={styles.logSource}>{log.source}</div>
                    <div className={styles.logMessage}>
                      {log.message}
                      {log.data && (
                        <details className={styles.logDetails}>
                          <summary className={styles.logSummary}>📊 Data</summary>
                          <pre className={styles.logData}>
                            {JSON.stringify(log.data, null, 2)}
                          </pre>
                        </details>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      case 'hecate':
        return (
          <div className={styles.mainHudContent}>
            <div className={styles.hecateContainer}>
              <div className={styles.hecateContent}>
                <div className={styles.hecateInterface}>
                  <div className={styles.chatSection}>
                    <div className={styles.hecateChat}>
                      <div className={styles.chatHeader}>
                        <div className={styles.chatTitle}>
                          <h4>Hecate Agent</h4>
                          {currentModel && (
                            <span className={styles.modelIndicator}>
                              {currentModel}
                            </span>
                          )}
                        </div>
                        <span className={`${styles.chatStatus} ${agentConnected ? styles.connected : styles.disconnected}`}>
                          {isConnectingAgent ? 'Connecting...' : agentConnected ? 'Live' : 'Offline'}
                        </span>
                      </div>

                      <div className={styles.chatMessages}>
                        {chatMessages.map((message) => (
                          <div
                            key={message.id}
                            className={`${styles.chatMessage} ${styles[`message-${message.sender}`]} ${message.type ? styles[`type-${message.type}`] : ''}`}
                          >
                            <div className={styles.messageHeader}>
                              <span className={styles.messageSender}>
                                {message.sender === 'hecate' ? (
                                  <span className={styles.hecateMessageSender}>
                                    <div
                                      className={`${styles.nullviewChat} ${styles[`chat-${agentConnected ? (message.type || 'base') : 'idle'}`]} ${styles.clickableNulleyeChat}`}
                                    >
                                      <div className={styles.staticFieldChat}></div>
                                      <div className={styles.coreNodeChat}></div>
                                      <div className={styles.streamLineChat}></div>
                                      <div className={styles.lightningSparkChat}></div>
                                    </div>
                                    Hecate
                                  </span>
                                ) : (
                                  '👤 You'
                                )}
                              </span>
                              <span className={styles.messageTime}>
                                {message.timestamp.toLocaleTimeString()}
                              </span>
                            </div>
                            <div className={styles.messageContent}>{message.message}</div>
                          </div>
                        ))}
                      </div>

                      <form className={styles.chatInput} onSubmit={handleChatSubmit}>
                        <input
                          type="text"
                          value={chatInput}
                          onChange={handleChatInputChange}
                          placeholder="Ask Hecate anything..."
                          className={styles.chatInputField}
                        />
                        <button type="submit" className={styles.chatSendButton}>
                          <span>➤</span>
                        </button>
                      </form>

                      {showSuggestions && (
                        <div className={styles.chatSuggestions}>
                          <div className={styles.suggestionsHeader}>
                            <span>💡 Quick Actions</span>
                          </div>
                          <div className={styles.suggestionsList}>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Show me available templates')}
                            >
                              📋 Browse Templates
                            </button>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Create a new workflow')}
                            >
                              🔗 New Workflow
                            </button>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Analyze market data')}
                            >
                              📊 Market Analysis
                            </button>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Generate code for trading bot')}
                            >
                              ⚡ Code Generator
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  </div>

                  <div className={styles.lensSection}>
                    {activeLens ? (
                      <div className={styles.lensExpanded}>{renderLensContent(activeLens)}</div>
                    ) : (
                      <div className={styles.lensScrollContainer}>
                        <div className={styles.lensInfoPanel}>
                          <div className={styles.lensInfoContent}>
                            <div className={styles.headerWithTooltip}>
                              <h3>🎯 Scopes</h3>
                              <div className={styles.tooltipContainer}>
                                <div className={styles.helpIcon}>?</div>
                                <div className={styles.tooltip}>
                                  <div className={styles.tooltipContent}>
                                    <h4>Scopes</h4>
                                    <p>
                                      Scopes are focused work environments, each tailored for specific tasks
                                      like code generation, data analysis, automation, and more. Select a
                                      scope to access its specialized toolset.
                                    </p>
                                  </div>
                                </div>
                              </div>
                            </div>

                            <div className={styles.lensAppsSection}>
                              <div className={styles.lensAppsGrid}>
                                {lensOptions.map((lens) => (
                                  <button
                                    key={lens.id}
                                    className={styles.lensAppButton}
                                    onClick={() => handleLensClick(lens.id)}
                                    style={{ '--lens-color': lens.color } as React.CSSProperties}
                                  >
                                    <span className={styles.lensAppIcon}>{lens.icon}</span>
                                    <span className={styles.lensAppTitle}>{lens.title}</span>
                                  </button>
                                ))}
                              </div>
                            </div>
                          </div>
                        </div>
                        
                        {/* Static Avatar container at bottom - outside of scrollable content */}
                        <div className={styles.scopesAvatarContainer}>
                          <div className={styles.hecateAvatar}>
                            <div className={styles.avatarCircle}>
                              <div
                                className={`${styles.nullviewAvatar} ${styles[nullviewState]} ${styles.clickableNulleye}`}
                              >
                                <div className={styles.pulseRingAvatar}></div>
                                <div className={styles.dataStreamAvatar}>
                                  <div className={styles.streamLineAvatar}></div>
                                  <div className={styles.streamLineAvatar}></div>
                                  <div className={styles.streamLineAvatar}></div>
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
                                <div className={styles.coreNodeAvatar}></div>
                              </div>
                            </div>
                            <div className={styles.avatarInfo}>
                              <h4>Hecate, Primary Interface Agent</h4>
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
        );
      default:
        return null;
    }
  };

  const renderHomeScreen = () => (
    <div className={styles.hudScreen}>
      {/* Menu bar at the top of innermost HUD screen */}
      <div className={styles.innerHudMenuBar}>
        <button 
          className={`${styles.menuButton} ${mainHudActiveTab === 'status' ? styles.active : ''}`}
          onClick={() => setMainHudActiveTab('status')}
        >
          Status
        </button>
        
        {/* Additional menu buttons that appear when logged in */}
        {publicKey && (
          <>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'tasks' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('tasks')}
            >
              Tasks
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'agents' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('agents')}
            >
              Agents
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'mcp' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('mcp')}
            >
              MCP
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'logs' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('logs')}
            >
              Logs
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'hecate' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('hecate')}
            >
              Hecate
            </button>
          </>
        )}
      </div>
      <div className={styles.homeContent}>
        <div className={styles.landingContent}>
          {renderMainHudTabContent()}
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
        <div style={{ textAlign: 'center', marginTop: '2rem' }}>
          <h3>Platform Overview</h3>
          <div className={styles.overviewGrid}>
            <div className={styles.overviewCard}>
              <h4>🎯 Arbitrage Agents</h4>
              <p>Real-time price monitoring across DEXs with automated MEV-protected execution</p>
            </div>
            <div className={styles.overviewCard}>
              <h4>📊 Portfolio Management</h4>
              <p>AI-driven rebalancing and risk assessment for optimal DeFi yields</p>
            </div>
            <div className={styles.overviewCard}>
              <h4>🤝 Social Trading</h4>
              <p>Sentiment analysis and community-driven trading signals</p>
            </div>
            <div className={styles.overviewCard}>
              <h4>🏛️ DAO Integration</h4>
              <p>Automated governance participation and proposal analysis</p>
            </div>
          </div>
          <p className={styles.overviewNote}>
            Connect your wallet to unlock live agents and start earning yield.
          </p>
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
      {renderControlScreen()}
      <div className={styles.hudWindow}>{renderScreen()}</div>
    </div>
  );
};

export default HUD;
