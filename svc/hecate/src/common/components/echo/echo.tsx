import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';

import { fetchWalletData } from '@services/api';
type Screen = 'home' | 'settings' | 'transactions'; // Add more screen types as needed

const Echo: React.FC = () => {
  const [screen, setScreen] = useState<Screen>('home');
  const [walletData, setWalletData] = useState<any>(null); // Adjust type according to your backend response

  useEffect(() => {
    const loadWalletData = async () => {
      try {
        const data = await fetchWalletData();
        setWalletData(data);
      } catch (error) {
        console.error('Failed to fetch wallet data:', error);
        // Optionally, set a state to show error message or fallback UI
      }
    };

    loadWalletData();
  }, []); // Empty dependency array means this effect runs once on mount

  const renderHomeScreen = () => {
    if (!walletData) return <p>Loading...</p>;

    return (
      <div className={styles.hud}>
        <h2 className={styles.hudTitle}>Wallet Overview</h2>
        <p>Balance: <span>{walletData.balance} SOL</span></p>
        <p>Address: <span>{walletData.address.slice(0, 6)}...{walletData.address.slice(-4)}</span></p>
        <p>Transactions: <span>{walletData.transactionCount}</span></p>
        {/* Add more data points as needed */}
      </div>
    );
  };

  const renderScreen = () => {
    switch (screen) {
      case 'home':
        return renderHomeScreen();
      case 'settings':
        return <p>Settings screen content...</p>; // Placeholder
      case 'transactions':
        return <p>Transactions screen content...</p>; // Placeholder
      default:
        return null;
    }
  };

  return (
    <div className={styles.echoContainer}>
      <div className={styles.echoScreen}>
        {renderScreen()}
      </div>
      <div className={styles.navButtons}>
        <button onClick={() => setScreen('home')} className={styles.navButton}>Home</button>
        <button onClick={() => setScreen('transactions')} className={styles.navButton}>Transactions</button>
        <button onClick={() => setScreen('settings')} className={styles.navButton}>Settings</button>
      </div>
    </div>
  );
};

export default Echo;
