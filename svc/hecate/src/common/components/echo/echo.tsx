import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData } from '@services/api';

type Screen = 'nexus' | 'bioMods' | 'captainsLog' | 'blackMarket' | 'externalInterfaces';

interface EchoProps {
  publicKey: string | null;
  onDisconnect: () => void;
}

const Echo: React.FC<EchoProps> = ({ publicKey, onDisconnect }) => {
  const [screen, setScreen] = useState<Screen>('nexus');
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

  const toggleScreen = (newScreen: Screen) => {
    if (!showSecondaryScreen) {
      setShowSecondaryScreen(true);
    }
    setScreen(newScreen);
  };

  const renderControlScreen = () => (
    <div className={styles.controlWindow}>
      <button onClick={() => toggleScreen('nexus')} className={styles.controlButton}>NEXUS</button>
      <button onClick={() => toggleScreen('bioMods')} className={styles.controlButton}>Bio Mods</button>
      <button onClick={() => toggleScreen('captainsLog')} className={styles.controlButton}>Captains Log</button>
      <button onClick={() => toggleScreen('blackMarket')} className={styles.controlButton}>Black Market</button>
      <button onClick={() => toggleScreen('externalInterfaces')} className={styles.controlButton}>External Interfaces</button>
      <button onClick={() => setShowSecondaryScreen(!showSecondaryScreen)} className={styles.controlButton}>
        ECHO {showSecondaryScreen ? '(Off)' : '(On)'}
      </button>
    </div>
  );

  const renderNexusScreen = () => (
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

  const renderBioModsScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>Bio Modifications</h2>
      <p>This would be where users could manage or view their bio enhancements.</p>
    </div>
  );

  const renderCaptainsLogScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>Captain's Log</h2>
      <p>Here you could log or view mission logs, personal notes, or whatever a captain might do.</p>
    </div>
  );

  const renderBlackMarketScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>Black Market</h2>
      <p>Access to underground dealings, perhaps for trading or acquiring rare items.</p>
    </div>
  );

  const renderExternalInterfacesScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>External Interfaces</h2>
      <p>Interact with external systems or devices, possibly for hacking or system integration.</p>
    </div>
  );

  const renderScreen = () => {
    switch (screen) {
      case 'nexus':
        return renderNexusScreen();
      case 'bioMods':
        return renderBioModsScreen();
      case 'captainsLog':
        return renderCaptainsLogScreen();
      case 'blackMarket':
        return renderBlackMarketScreen();
      case 'externalInterfaces':
        return renderExternalInterfacesScreen();
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
      {renderControlScreen()}
    </div>
  );
};

export default Echo;
