import React, { useState, useEffect, useRef } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData, fetchUserProfile, fetchAscentLevel, fetchActiveMission, MissionData } from '@services/api';
import xLogo from '../../../assets/images/X_logo_black.png';
import nullLogo from '../../../assets/images/null_logo.png';
import echoBatWhite from '../../../assets/images/echo_bat_white.png';
import echoBatBlack from '../../../assets/images/echo_bat_black.png';

type Screen = 'chambers' | 'camp' | 'inventory' | 'campaign' | 'lab';
type Theme = 'null' | 'light';
type TabType = 'missions' | 'systems' | 'defense' | 'uplink' | 'echo' | 'status';

interface EchoProps {
  publicKey: string | null;
  onDisconnect: () => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'cyber' | 'light') => void;
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
  chambers: 'E.C.H.O CHAMBERS',
  camp: 'CAMP',
  inventory: 'CACHE',
  campaign: 'CAMPAIGN',
  lab: 'LAB',
};

const Echo: React.FC<EchoProps> = ({ 
  publicKey, 
  onDisconnect, 
  theme = 'light', 
  onClose, 
  onThemeChange
}) => {
  const [screen, setScreen] = useState<Screen>('chambers');
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
  const [activeTab, setActiveTab] = useState<TabType>('echo');
  const [emberLinkStatus, setEmberLinkStatus] = useState<EmberLinkStatus>({
    connected: false,
    lastSeen: null,
    browserInfo: null
  });
  const [isDigitizing, setIsDigitizing] = useState<boolean>(false);
  const [selectedUplink, setSelectedUplink] = useState<Uplink | null>(null);
  const [showLeaderboard, setShowLeaderboard] = useState<boolean>(false);

  // Only unlock 'chambers' by default, unlock others if logged in
  const unlockedScreens = publicKey ? ['chambers', 'camp'] : ['chambers'];

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
      id: "ECHO-001",
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
      id: "ECHO-002",
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
      id: "ECHO-003",
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
      id: "ECHO-004",
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
      id: "ECHO-005",
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
      id: "ECHO-006",
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
      id: "ECHO-007",
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
      id: "ECHO-008",
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
      id: "ECHO-009",
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
      id: "ECHO-010",
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
      id: "ECHO-011",
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
          localStorage.removeItem('hasSeenEcho');
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

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <div className={styles.nullblockTitle}>NULLBLOCK</div>
      <div
        className={`${styles.screenLabel} ${screen === 'chambers' ? styles.centeredLabel : ''}`}
      >
        {SCREEN_LABELS[screen]}
      </div>
      <div className={styles.navbarButtons}>
        <button className={styles.batLogoButton} onClick={() => setScreen('chambers')}>
          <img src={echoBatWhite} alt="Bat Logo" className={styles.batLogoIcon} />
        </button>
        <button className={styles.nullLogoButton} onClick={() => setScreen('chambers')}>
          <img src={nullLogo} alt="Null Logo" className={styles.nullLogoIcon} />
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
          E.C:
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
              Ember Conduit: A medium to speak into flame. This ancient technology allows direct communication with the primordial forces of the Nullblock universe. Through an Ember Conduit, users can channel energy, access forbidden knowledge, and potentially reshape reality itself. Warning: Unauthorized use may result in spontaneous combustion or worse.
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
        <p>Return to camp and await further instructions.</p>
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

  const renderEchoTab = () => {
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
              <p>E.C.H.O system is active and operational.</p>
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
                          value: 'ECHO ID'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>🆔 ID</span>
                    <span className={styles.itemValue}>ECHO-{userProfile.id || '0000'}</span>
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
                    id: 'ec',
                    name: 'EMBER CONDUIT',
                    status: 'active',
                    icon: '🔥',
                    details: {
                      description: 'Ember Conduit: A medium to speak into flame. This ancient technology allows direct communication with the primordial forces of the Nullblock universe. Through an Ember Conduit, users can channel energy, access forbidden knowledge, and potentially reshape reality itself. Warning: Unauthorized use may result in spontaneous combustion or worse.',
                      stats: [
                        {
                          label: 'Status',
                          value: userProfile.matrix.status
                        },
                        {
                          label: 'Type',
                          value: 'Ember Conduit'
                        }
                      ]
                    }
                  })}>
                    <span className={styles.itemLabel}>🔥 EMBER CONDUIT</span>
                    <span className={styles.itemValue}>{userProfile.matrix.status}</span>
                  </div>
                </div>
                <button 
                  className={styles.addLinkButton}
                  onClick={() => alert('No Ember Conduit loaded')}
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
                  className={`${styles.statusTab} ${activeTab === 'echo' ? styles.activeTab : ''}`}
                  onClick={() => setActiveTab('echo')}
                >
                  E.C.H.O
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
              </div>
              <div className={styles.tabContent}>
                {activeTab === 'echo' && renderEchoTab()}
                {activeTab === 'systems' && (
                  <div className={styles.systemsTab}>
                    <div className={styles.lockedContent}>
                      <p>This feature is currently locked.</p>
                      <p>Return to camp and await further instructions.</p>
                    </div>
                  </div>
                )}
                {activeTab === 'defense' && (
                  <div className={styles.defenseTab}>
                    <div className={styles.lockedContent}>
                      <p>This feature is currently locked.</p>
                      <p>Return to camp and await further instructions.</p>
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
                          "Welcome, Camper, to your first trial. Tend the flame carefully.
                          Share your Base Camp on X—let its glow haunt the realm.
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
              <h3>TOP CAMPERS</h3>
              <button className={styles.closeButton} onClick={handleCloseLeaderboard}>×</button>
            </div>
            <div className={styles.legendWarning}>
              <p>⚠️ These are the legends of the last Flame. The current Conduit has yet to awaken.</p>
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
          <p>Complete missions to acquire gear.</p>
        </div>
      </div>
      <div className={styles.inventorySection}>
        <h3>SUPPLIES</h3>
        <div className={styles.emptyState}>
          <p>Cache empty.</p>
          <p>Gather resources to expand inventory.</p>
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
          <p className={styles.placeholder}>Complete training to begin</p>
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
          <p className={styles.placeholder}>Run diagnostics to begin</p>
        </div>
      </div>
    </div>
  );

  const renderChambersScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.headerContainer}>
        <h2 className={styles.hudTitle}>ECHO CHAMBERS</h2>
        <div className={styles.headerDivider}></div>
      </div>
      <div className={styles.campContent}>
        <div style={{textAlign: 'center', marginTop: '2rem'}}>
          <h3>Welcome to the ECHO Chambers</h3>
          <p>This is the default screen. Connect your wallet to unlock CAMP and other features.</p>
        </div>
      </div>
    </div>
  );

  const renderScreen = () => {
    if (!unlockedScreens.includes(screen)) {
      return renderLockedScreen();
    }
    switch (screen) {
      case 'chambers':
        return renderChambersScreen();
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
      <div className={styles.hudWindow}>
        {renderScreen()}
      </div>
    </div>
  );
};

export default Echo;