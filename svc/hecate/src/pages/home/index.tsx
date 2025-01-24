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
    const checkWalletConnection = async () => {
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;
        if (provider) {
          // Check if Phantom is already connected
          if (provider.isConnected) {
            try {
              const connectedPublicKey = await provider.getPublicKey();
              setPublicKey(connectedPublicKey.toString());
              setWalletConnected(true);
              setShowEcho(true);
              localStorage.setItem('walletPublickey', connectedPublicKey.toString());
            } catch (error) {
              console.error('Failed to get public key:', error);
              // If we can't get the public key, we might need to connect again
              localStorage.removeItem('walletPublickey');
            }
          } else {
            // Check if there's a saved public key from a previous session
            const savedPublicKey = localStorage.getItem('walletPublickey');
            if (savedPublicKey) {
              try {
                // Attempt to reconnect with the saved public key
                await provider.connect();
                setPublicKey(savedPublicKey);
                setWalletConnected(true);
                setShowEcho(true);
              } catch (error) {
                console.error('Failed to auto-reconnect:', error);
                localStorage.removeItem('walletPublickey');
              }
            }
          }
        }
      }
    };

    checkWalletConnection();
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
          localStorage.setItem('walletPublickey', publicKey.toString());
        } catch (error) {
          console.error('Manual connect error:', error);
        }
      } else {
        alert('Phantom wallet not detected');
      }
    } else {
      alert('Please install Phantom wallet extension from the Chrome Web Store.');
    }
  };

  const handleDisconnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          await provider.disconnect();
          setWalletConnected(false);
          setPublicKey(null);
          setShowEcho(false);
          localStorage.removeItem('walletPublickey');
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
      {showEcho && <Echo publicKey={publicKey} onDisconnect={handleDisconnect} />}
    </>
  );
};

export default Home;
