import React, { useState, useEffect } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import FogCanvas from '@components/fog/fog';
import Echo from '@components/echo/echo';

const Home: React.FC = () => {
  const [walletConnected, setWalletConnected] = useState<boolean>(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [showEcho, setShowEcho] = useState<boolean>(false);

  useEffect(() => {
    const connectPhantom = async () => {
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;
        if (provider) {
          try {
            const { publicKey } = await provider.connect();
            setPublicKey(publicKey.toString());
            setWalletConnected(true);
            setShowEcho(true);
          } catch (error) {
            console.error('Error connecting to Phantom:', error);
          }
        } else {
          console.log('Phantom wallet not detected');
        }
      } else {
        console.log('Please install Phantom wallet extension');
      }
    };

    connectPhantom();
  }, []);

  const manualConnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          const { publicKey } = await provider.connect();
          setPublicKey(publicKey.toString());
          setWalletConnected(true);
          setShowEcho(true);
        } catch (error) {
          console.error('Manual connect error:', error);
        }
      } else {
        alert('Phantom wallet not detected');
      }
    } else {
      alert('Please install Phantom wallet extension');
    }
  };

  // Function to handle disconnection
  const handleDisconnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          await provider.disconnect();
          setWalletConnected(false);
          setPublicKey(null);
          setShowEcho(false);
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  return (
    <>
      <div className={styles.backgroundImage} />
      <StarsCanvas />
      <div className={styles.scene}>
        <div className={styles.fire}></div>
        <div className={styles.robot}></div>
        <div className={styles.trader1}></div>
      </div>
      <div style={{ position: 'relative', zIndex: 2 }}>
        {!walletConnected && (
          <button onClick={manualConnect} className={styles.button}>Connect Phantom</button>
        )}
      </div>
      {showEcho && <Echo publicKey={publicKey} onDisconnect={handleDisconnect} />} {/* Pass publicKey and disconnect function */}
    </>
  );
};

export default Home;
