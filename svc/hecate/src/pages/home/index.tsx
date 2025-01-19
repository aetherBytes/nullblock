import React, { useState, useEffect } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars'; // Import StarsCanvas component

const Home = () => {
  const [walletConnected, setWalletConnected] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);

  useEffect(() => {
    const connectPhantom = async () => {
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;
        if (provider) {
          try {
            const { publicKey } = await provider.connect();
            setPublicKey(publicKey.toString());
            setWalletConnected(true);
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

  return (
    <>
      <div className={styles.backgroundImage} />
      <StarsCanvas/>
      <div style={{ position: 'relative', zIndex: 1 }}>
        {walletConnected ? (
          <div>
            <p>Connected with: {publicKey}</p>
            <button
              onClick={() => {
                setWalletConnected(false);
                setPublicKey(null);
              }}
              className={styles.button}
            >
              Disconnect
            </button>
          </div>
        ) : (
          <button onClick={manualConnect} className={styles.button}>Connect Phantom</button>
        )}
      </div>
    </>
  );
};

export default Home;
