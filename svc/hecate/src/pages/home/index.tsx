import type { FCRoute } from '@lomray/vite-ssr-boost/interfaces/fc-route';
import React, { useState, useEffect } from 'react';
import bigBeerus from '@assets/images/big_beerus_clip.png';
import Echo from '@components/echo/echo';
import styles from './styles.module.scss';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';

const Home: FCRoute = () => {
  const [backgroundImageIndex, setBackgroundImageIndex] = useState(-1); // Initial index set to -1
  const backgroundImages = [
    '../../assets/images/green_dawn.png',
    '../../assets/images/red_dawn.png',
    '../../assets/images/red_dawn_2.png',
    '../../assets/images/nilblock.png',
    '../../assets/images/nexoliths.png',
    '../../assets/images/nexoliths_2.png',
    '../../assets/images/nexoliths_3.png',
    '../../assets/images/nexoliths_4.png',
    '../../assets/images/nexoliths_5.png',
    '../../assets/images/nex_gate.png',
    '../../assets/images/nowhere_bridge.png',
  ];

  useEffect(() => {
    const randomIndex = Math.floor(Math.random() * backgroundImages.length);
    setBackgroundImageIndex(randomIndex);
  }, []);

  return (
    <div
      className={styles.backgroundImage}
      style={{
        backgroundImage: `url(${backgroundImageIndex !== -1 ? backgroundImages[backgroundImageIndex] : 'about:blank'})` // Set initial image to blank if index is -1
      }}
    >
      <Echo />
      <div className={styles.buttonsContainer}>
        {/* Links updated to open in a new tab */}
        <a
          href="https://twitter.com/MoxiSKeeper"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.echoButton}
        >
          <img src={XLogo} alt="Twitter Logo" />
        </a>
        <a
          href="https://twitter.com"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.echoButton}
        >
          <img src={discordLogo} alt="Discord Logo" />
        </a>
        <a
          href="https://twitter.com"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.echoButton}
        >
          <img src={telegramLogo} alt="Telegram Logo" />
        </a>
      </div>
    </div>
  );
};

export default Home;

