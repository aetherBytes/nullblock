import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData } from '@services/api';

type Screen = 'home' | 'settings';

interface EchoProps {
  publicKey: string | null;
  onDisconnect: () => void;
}

const Echo: React.FC<EchoProps> = ({ publicKey, onDisconnect }) => {
  const [screen, setScreen] = useState<Screen>('home');
  const [walletData, setWalletData] = useState<any>(null);
  const [showSecondaryScreen, setShowSecondaryScreen] = useState(true);

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

  // Update to turn on ECHO screen when any button is clicked if it's off
  const toggleScreen = (newScreen: Screen) => {
    if (!showSecondaryScreen) {
      setShowSecondaryScreen(true);
    }
    setScreen(newScreen);
  };

  const renderControlScreen = () => (
    <div className={styles.controlScreen}>
      <button onClick={() => toggleScreen('home')} className={styles.controlButton}>NEXUS</button>
      <button onClick={() => toggleScreen('settings')} className={styles.controlButton}>Settings</button>
      <button onClick={() => setShowSecondaryScreen(!showSecondaryScreen)} className={styles.controlButton}>
        ECHO {showSecondaryScreen ? '(Off)' : '(On)'}
      </button>
    </div>
  );

  const renderHomeScreen = () => (
    <div className={styles.hudScreen}>
      {walletData ? (
        <>
          <h2 className={styles.hudTitle}>Wallet Overview</h2>
          <p>Balance: <span>{walletData.balance} SOL</span></p>
          <p>Address: <span>{walletData.address.slice(0, 6)}...{walletData.address.slice(-4)}</span></p>
          <p>Transactions: <span>{walletData.transactionCount}</span></p>
        </>
      ) : (
        <p>Loading...</p>
      )}
      <div className={styles.bottomLeftInfo}>
        <p>Electronic Communications HUB and Omnitool</p>
      </div>
      <div className={styles.bottomRightInfo}>
        <p>biological interface online... connected entity: {publicKey || 'loading'}</p>
      </div>
    </div>
  );

  const renderSettingsScreen = () => (
    <div className={styles.settingsScreen}>
      <p>Connected with: {publicKey}</p>
      <button onClick={onDisconnect} className={styles.button}>Disconnect</button>
    </div>
  );

  const renderScreen = () => {
    switch (screen) {
      case 'home':
        return renderHomeScreen();
      case 'settings':
        return renderSettingsScreen();
      default:
        return null;
    }
  };

  return (
    <div className={styles.echoContainer}>
      {showSecondaryScreen && (
        <div className={styles.hudWindow}>
          {renderScreen()}
        </div>
      )}
      <div className={styles.controlWindow}>
        {renderControlScreen()}
      </div>
    </div>
  );
};

export default Echo;
