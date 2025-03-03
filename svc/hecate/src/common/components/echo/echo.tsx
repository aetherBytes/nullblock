import React, { useState, useEffect } from 'react';
import styles from './echo.module.scss';
import { fetchWalletData } from '@services/api';

type Screen = 'camp' | 'inventory' | 'activeReality' | 'nexusMarket' | 'interfaces';

interface EchoProps {
  publicKey: string | null;
  onDisconnect: () => void;
}

const Echo: React.FC<EchoProps> = ({ publicKey, onDisconnect }) => {
  const [screen, setScreen] = useState<Screen>('camp');
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

  const handleDisconnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          // Force disconnect and clear session
          await provider.disconnect();
          await provider.request({ method: 'disconnect' });
          localStorage.removeItem('walletPublickey');
          onDisconnect();
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <button onClick={() => setScreen('camp')} className={styles.navButton}>CAMP</button>
      <button onClick={() => setScreen('inventory')} className={styles.navButton}>INVENTORY</button>
      <button onClick={() => setScreen('activeReality')} className={styles.navButton}>ACTIVE REALITY</button>
      <button onClick={() => setScreen('nexusMarket')} className={styles.navButton}>NEXUS MARKET</button>
      <button onClick={() => setScreen('interfaces')} className={styles.navButton}>INTERFACES</button>
      <button onClick={handleDisconnect} className={styles.navButton}>DISCONNECT</button>
    </nav>
  );

  const renderCampScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>NEURAL SANCTUARY</h2>
      <div className={styles.walletInfo}>
        <p><strong>Neural Link:</strong> <span>{publicKey?.slice(0, 6)}...{publicKey?.slice(-4)}</span></p>
        <p><strong>Quantum Balance:</strong> <span>{walletData?.balance || '0'} SOL</span></p>
      </div>
      <div className={styles.campContent}>
        <p>Welcome to your neural sanctuary, a secure haven within the digital void. This encrypted space serves as your command center for reality manipulation and neural augmentation.</p>
        <p>Current System Status:</p>
        <ul>
          <li>Neural Link: <span className={styles.active}>ACTIVE</span></li>
          <li>Reality Anchors: <span className={styles.pending}>CALIBRATING</span></li>
          <li>Quantum Signature: <span className={styles.stable}>STABLE</span></li>
        </ul>
      </div>
    </div>
  );

  const renderInventoryScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>NEURAL ARSENAL</h2>
      <div className={styles.inventorySection}>
        <h3>QUANTUM AUGMENTS</h3>
        <div className={styles.emptyState}>
          <p>No neural modifications detected.</p>
          <p>Visit the Nexus Market to acquire reality-altering tools.</p>
        </div>
      </div>
      <div className={styles.inventorySection}>
        <h3>MEMORY FRAGMENTS</h3>
        <div className={styles.emptyState}>
          <p>Memory bank initialization required.</p>
          <p>Install memory cards to unlock enhanced capabilities.</p>
        </div>
      </div>
    </div>
  );

  const renderActiveRealityScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>REALITY NEXUS</h2>
      <div className={styles.realityContent}>
        <div className={styles.realityStatus}>
          <h3>CURRENT REALITY STRAND</h3>
          <p>Initialization Phase: <span>ALPHA</span></p>
          <p>Reality Stability: <span>97.3%</span></p>
        </div>
        <div className={styles.missions}>
          <h3>ACTIVE PROTOCOLS</h3>
          <p className={styles.placeholder}>Scanning quantum frequencies...</p>
          <p className={styles.placeholder}>Awaiting reality stabilization...</p>
        </div>
      </div>
    </div>
  );

  const renderNexusMarketScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>QUANTUM NEXUS EXCHANGE</h2>
      <div className={styles.marketContent}>
        <div className={styles.marketSection}>
          <h3>FEATURED AUGMENTS</h3>
          <p className={styles.placeholder}>Market protocols initializing...</p>
        </div>
        <div className={styles.marketSection}>
          <h3>MEMORY FRAGMENTS</h3>
          <p className={styles.placeholder}>Quantum signature verification required...</p>
        </div>
        <div className={styles.marketSection}>
          <h3>REALITY TOKENS</h3>
          <p className={styles.placeholder}>Token matrix stabilizing...</p>
        </div>
      </div>
    </div>
  );

  const renderInterfacesScreen = () => (
    <div className={styles.hudScreen}>
      <h2 className={styles.hudTitle}>NEURAL INTERFACES</h2>
      <div className={styles.interfaceContent}>
        <div className={styles.interfaceSection}>
          <h3>ACTIVE CONNECTIONS</h3>
          <p>Phantom Neural Bridge: <span className={styles.connected}>CONNECTED</span></p>
          <p>Reality Anchor: <span className={styles.initializing}>INITIALIZING</span></p>
        </div>
        <div className={styles.interfaceSection}>
          <h3>AVAILABLE PROTOCOLS</h3>
          <p className={styles.placeholder}>Scanning for compatible neural interfaces...</p>
        </div>
      </div>
    </div>
  );

  const renderScreen = () => {
    switch (screen) {
      case 'camp':
        return renderCampScreen();
      case 'inventory':
        return renderInventoryScreen();
      case 'activeReality':
        return renderActiveRealityScreen();
      case 'nexusMarket':
        return renderNexusMarketScreen();
      case 'interfaces':
        return renderInterfacesScreen();
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
