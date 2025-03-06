import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData } from '@services/api';

type Screen = 'camp' | 'inventory' | 'campaign' | 'lab';

interface EchoProps {
  publicKey: string | null;
  onDisconnect: () => void;
}

const Echo: React.FC<EchoProps> = ({ publicKey, onDisconnect }) => {
  const [screen, setScreen] = useState<Screen>('camp');
  const [walletData, setWalletData] = useState<any>(null);

  // Define which screens are unlocked
  const unlockedScreens = ['camp'];

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
      <h2 className={styles.hudTitle}>BASE CAMP</h2>
      <div className={styles.walletInfo}>
        <p><strong>ID:</strong> <span>{publicKey?.slice(0, 6)}...{publicKey?.slice(-4)}</span></p>
        <p><strong>Balance:</strong> <span>{walletData?.balance || '0'} SOL</span></p>
      </div>
      <div className={styles.campContent}>
        <p>Camp Status:</p>
        <ul>
          <li>Perimeter: <span className={styles.active}>SECURE</span></li>
          <li>Systems: <span className={styles.pending}>SCANNING</span></li>
          <li>Defense: <span className={styles.stable}>ACTIVE</span></li>
        </ul>
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
    <div className={styles.echoContainer}>
      {renderControlScreen()}
      <div className={styles.hudWindow}>
        {renderScreen()}
      </div>
    </div>
  );
};

export default Echo;
