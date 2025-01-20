import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData } from '@services/api'; // Import the new API function

type Screen = 'home' | 'settings'; // Only these two screens for now

const Echo: React.FC = () => {
  const [screen, setScreen] = useState<Screen>('home');
  const [walletData, setWalletData] = useState<any>(null);
  const [showSecondaryScreen, setShowSecondaryScreen] = useState(true); // Default to visible

  useEffect(() => {
    const loadWalletData = async () => {
      try {
        // Replace with actual public key logic
        const data = await fetchWalletData("YOUR_PUBLIC_KEY_HERE");
        setWalletData(data);
      } catch (error) {
        console.error('Failed to fetch wallet data:', error);
      }
    };

    loadWalletData();
  }, []);

  const renderControlScreen = () => (
    <div className={styles.controlScreen}>
      <button onClick={() => setScreen('home')} className={styles.controlButton}>Home</button>
      <button onClick={() => setScreen('settings')} className={styles.controlButton}>Settings</button>
      <button onClick={() => setShowSecondaryScreen(!showSecondaryScreen)} className={styles.controlButton}>Toggle HUD</button>
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
    </div>
  );

  const renderSettingsScreen = () => (
    <div className={styles.settingsScreen}>
      <p>Connected with: {walletData?.address}</p>
      <button onClick={() => {
        // Here you would handle disconnection logic
        console.log('Disconnecting wallet');
      }} className={styles.button}>Disconnect</button>
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
