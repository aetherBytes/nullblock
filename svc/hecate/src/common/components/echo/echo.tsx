import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData } from '@services/api';

type Screen = 'camp' | 'inventory' | 'campaign' | 'lab';
type Theme = 'null' | 'light';

interface EchoProps {
  publicKey: string | null;
  onDisconnect: () => void;
  onExpandChat: () => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'cyber' | 'light') => void;
}

interface UserProfile {
  id: string;
  ascent: number;
  nectar: number;
  memories: number;
  matrix: {
    level: string;
    rarity: string;
    status: string;
  };
}

interface SystemAnalysis {
  name: string;
  status: string;
  locked: boolean;
}

const Echo: React.FC<EchoProps> = ({ publicKey, onDisconnect, onExpandChat, theme = 'null', onClose, onThemeChange }) => {
  const [screen, setScreen] = useState<Screen>('camp');
  const [walletData, setWalletData] = useState<any>(null);
  const [userProfile, setUserProfile] = useState<UserProfile>({
    id: publicKey ? `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol` : '',
    ascent: 1,
    nectar: 0,
    memories: 0,
    matrix: {
      level: 'NONE',
      rarity: 'NONE',
      status: 'NO MATRIX FOUND'
    }
  });
  const [alerts, setAlerts] = useState<number>(3); // Default to 3 alerts for demo
  const [showAlerts, setShowAlerts] = useState<boolean>(false);

  // Define which screens are unlocked
  const unlockedScreens = ['camp'];

  const systemAnalysisItems: SystemAnalysis[] = [
    { name: "Neural Link", status: "SCANNING", locked: false },
    { name: "Wallet Health", status: "OPTIMAL", locked: false },
    { name: "Token Analysis", status: "IN PROGRESS", locked: false },
    { name: "Risk Assessment", status: "LOW", locked: false },
    { name: "Memory Integrity", status: "CHECKING", locked: true },
    { name: "Network Status", status: "CONNECTED", locked: true },
    { name: "Matrix Sync", status: "OFFLINE", locked: true },
    { name: "Reality Engine", status: "DORMANT", locked: true },
    { name: "Core Systems", status: "LOCKED", locked: true },
    { name: "Neural Cache", status: "UNAVAILABLE", locked: true }
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
          const data = await fetchWalletData(publicKey);
          setWalletData(data);
          // Update user profile with wallet data
          setUserProfile(prev => ({
            ...prev,
            nectar: data.balance || 0
          }));
        } catch (error) {
          console.error('Failed to fetch wallet data:', error);
        }
      }
    };

    loadWalletData();
  }, [publicKey]);

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
    onExpandChat();
  };

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <button onClick={() => handleScreenChange('camp')} className={styles.navButton}>
        CAMP
      </button>
      <button 
        onClick={() => handleScreenChange('inventory')} 
        className={`${styles.navButton} ${!unlockedScreens.includes('inventory') ? styles.locked : ''}`}
        disabled={!unlockedScreens.includes('inventory')}
      >
        CACHE <span className={styles.lockIcon}>[LOCKED]</span>
      </button>
      <button 
        onClick={() => handleScreenChange('campaign')} 
        className={`${styles.navButton} ${!unlockedScreens.includes('campaign') ? styles.locked : ''}`}
        disabled={!unlockedScreens.includes('campaign')}
      >
        CAMPAIGN <span className={styles.lockIcon}>[LOCKED]</span>
      </button>
      <button 
        onClick={() => handleScreenChange('lab')} 
        className={`${styles.navButton} ${!unlockedScreens.includes('lab') ? styles.locked : ''}`}
        disabled={!unlockedScreens.includes('lab')}
      >
        LAB <span className={styles.lockIcon}>[LOCKED]</span>
      </button>
      <button onClick={handleDisconnect} className={styles.navButton}>
        DISCONNECT
      </button>
    </nav>
  );

  const renderUserProfile = () => (
    <div className={styles.userProfile}>
      <div className={styles.profileItem}>
        <span className={styles.label}>ID:</span>
        <span className={styles.value}>{userProfile.id}</span>
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>ASCENT:</span>
        <span className={styles.value}>{userProfile.ascent}</span>
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>NECTAR:</span>
        <span className={styles.value}>₦ {userProfile.nectar.toFixed(2)}</span>
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>MEMORIES:</span>
        <span className={styles.value}>{userProfile.memories}</span>
      </div>
      <div className={styles.profileItem}>
        <span className={styles.label}>MATRIX:</span>
        <span className={`${styles.value} ${styles.matrix} ${styles[userProfile.matrix.rarity.toLowerCase()]}`}>
          {userProfile.matrix.status}
        </span>
      </div>
    </div>
  );

  const renderLockedScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>ACCESS RESTRICTED</h2>
      <div className={styles.lockedContent}>
        <p>This feature is currently locked.</p>
        <p>Return to camp and await further instructions.</p>
      </div>
    </div>
  );

  const renderCampScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>CAMP</h2>
      {renderUserProfile()}
      <div className={styles.campContent}>
        <div className={styles.campGrid}>
          <div className={styles.campAnalysis}>
            <h3>SYSTEM ANALYSIS</h3>
            <ul>
              {systemAnalysisItems.map((item, index) => (
                <li key={index} className={item.locked ? styles.locked : ''}>
                  {item.name}: <span className={getStatusClass(item.status)}>{item.status}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className={styles.divider}></div>
          <div className={styles.campStatus}>
            <h3>CAMP STATUS</h3>
            <ul>
              <li>
                Alerts: <span className={`${styles.alertCount} ${styles.active}`} onClick={handleAlertClick}>{alerts}</span>
              </li>
              <li>Perimeter: <span className={styles.active}>SECURE</span></li>
              <li>Systems: <span className={styles.pending}>SCANNING</span></li>
              <li>Defense: <span className={styles.stable}>ACTIVE</span></li>
              <li>Uplink: <span className={styles.active}>STABLE</span></li>
              <li>Matrix Core: <span className={styles.pending}>INITIALIZING</span></li>
              <li>Reality Engine: <span className={styles.stable}>STANDBY</span></li>
            </ul>
            {!userProfile.matrix.level || userProfile.matrix.level === 'NONE' ? (
              <div className={styles.matrixPrompt}>
                <button className={styles.marketButton} onClick={() => handleScreenChange('inventory')}>
                  VISIT MARKETPLACE
                </button>
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );

  const renderInventoryScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>CACHE</h2>
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
      <h2 className={styles.hudTitle}>CAMPAIGN</h2>
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
      <h2 className={styles.hudTitle}>LAB</h2>
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

  const getStatusClass = (status: string): string => {
    switch (status.toLowerCase()) {
      case 'optimal':
      case 'connected':
      case 'secure':
      case 'active':
        return styles.active;
      case 'scanning':
      case 'in progress':
      case 'checking':
      case 'initializing':
        return styles.pending;
      case 'low':
      case 'standby':
      case 'stable':
        return styles.stable;
      default:
        return styles.inactive;
    }
  };

  const renderScreen = () => {
    if (!unlockedScreens.includes(screen)) {
      return renderLockedScreen();
    }

    switch (screen) {
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
