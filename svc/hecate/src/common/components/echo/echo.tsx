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

  const changeScreen = (newScreen: Screen) => {
    setScreen(newScreen);
  };

  const handleDisconnect = () => {
    onDisconnect();
  };

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <button onClick={() => changeScreen('nexus')} className={styles.navButton}>NEXUS</button>
      <button onClick={() => changeScreen('bioMods')} className={styles.navButton}>Bio Mods</button>
      <button onClick={() => changeScreen('captainsLog')} className={styles.navButton}>Captain's Log</button>
      <button onClick={() => changeScreen('blackMarket')} className={styles.navButton}>Black Market</button>
      <button onClick={() => changeScreen('externalInterfaces')} className={styles.navButton}>Interfaces</button>
      <button onClick={handleDisconnect} className={styles.navButton}>Disconnect</button>
    </nav>
  );

  const renderNexusScreen = () => {
    if (!walletData) return <p>Loading...</p>;

    return (
      <div className={`${styles.hudScreen} ${styles.nexus}`}>
        <h2 className={styles.hudTitle}>NEXUS</h2>
        <div className={styles.walletInfo}>
          <p><strong>Balance:</strong> <span>{walletData.balance} SOL</span></p>
          <p><strong>Address:</strong> <span>{publicKey?.slice(0, 6)}...{publicKey?.slice(-4)}</span></p>
          <p><strong>Transactions:</strong> <span>{walletData.transactionCount}</span></p>
          {walletData.holdings &&
            <p><strong>Holdings:</strong>
              <span>{Object.keys(walletData.holdings).map(h => `${h}: ${walletData.holdings[h]} `)}</span>
            </p>
          }
        </div>
        <div className={styles.nexusActions}>
          <button onClick={() => alert('Feature not implemented yet')}>Send SOL</button>
          <button onClick={() => alert('Feature not implemented yet')}>Receive SOL</button>
        </div>
        <div className={styles.bottomLeftInfo}>
          <p>Electronic Communications HUB and Omnitool</p>
        </div>
      </div>
    );
  };

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
      <div className={styles.hudWindow}>
        {renderScreen()}
      </div>
      {renderControlScreen()}
    </div>
  );
};

export default Echo;
