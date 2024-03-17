// Import React and necessary components
import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi'; // Adjust the import path as needed
import Echo from '@components/echo/echo'; // Adjust the import path as needed
import XLogo from '@assets/images/X_logo_black.png'; // Ensure the path is correct
import discordLogo from '@assets/images/discord_logo_black.png'; // Ensure the path is correct
import telegramLogo from '@assets/images/telegram_logo_black.png'; // Ensure the path is correct
import moxi_black from '@assets/images/moxi_avatar_black_200.png'; // Ensure the path is correct
import styles from './styles.module.scss'; // Adjust the import path as needed

const Home: FCRoute = () => {
  const [isEchoVisible, setIsEchoVisible] = useState(false);

  // Toggle function passed to Moxi and the new button
  const toggleEchoVisibility = () => setIsEchoVisible(prev => !prev);

  return (
    <div className={styles.backgroundImage}>
      {/* Moxi component that can toggle the ECHO UI */}
      <Moxi onToggleEcho={toggleEchoVisibility} />
      <div className={styles.buttonsContainer}>
        <a
          href="https://twitter.com/MoxiSKeeper"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.echoButton}
        >
          <img src={XLogo} alt="Twitter Logo" />
        </a>
        <a
          href="https://discord.com"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.echoButton}
        >
          <img src={discordLogo} alt="Discord Logo" />
        </a>
        <a
          href="https://telegram.org"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.echoButton}
        >
          <img src={telegramLogo} alt="Telegram Logo" />
        </a>
        {/* New button to toggle the ECHO UI */}
        <button onClick={toggleEchoVisibility} className={styles.echoButton}>
          <img src={moxi_black} alt="Moxi Logo" />
        </button>
      </div>

      {/* Conditionally render the Echo component */}
      {isEchoVisible && <Echo />}
    </div>
  );
};

export default Home;

