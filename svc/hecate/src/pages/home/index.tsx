import type { FCRoute } from '@lomray/vite-ssr-boost/interfaces/fc-route';
import React from 'react';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';
import Echo from '@components/echo/echo';
import Moxi from '@components/moxi/parent';
import styles from './styles.module.scss';

const Home: FCRoute = () => (
  <div className={styles.backgroundImage}>
    <Moxi />
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

export default Home;
