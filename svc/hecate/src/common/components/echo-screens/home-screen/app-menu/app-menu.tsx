
// AppMenu.js
import React, { useState, useEffect } from 'react';
import styles from './app-menu.module.scss';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';

const AppMenu = ({
  toggleEchoVisibility,
  closeEchoScreen,
  isUIVisible,
}) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  useEffect(() => {
    if (!isUIVisible) {
      setIsMenuOpen(false);
    }
  }, [isUIVisible]);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  const buttons = [
    { href: 'https://twitter.com/MoxiSKeeper', icon: XLogo, alt: 'Twitter Logo' },
    { href: 'https://discord.com', icon: discordLogo, alt: 'Discord Logo' },
    { href: 'https://telegram.org', icon: telegramLogo, alt: 'Telegram Logo' },
  ];

  return (
    <div className={styles.appMenu}>
      <ButtonWrapper title="Toggle ECHO" buttonText="EChat" setCurrentScreen={toggleEchoVisibility} />
      <ButtonWrapper title="Toggle Social Media Menu" buttonText="Social Media" setCurrentScreen={toggleMenu} />
      <div className={`${styles.menuContainer} ${isMenuOpen ? styles.active : ''}`}>
        {buttons.map((button, index) => (
          <a key={index} href={button.href} target="_blank" rel="noopener noreferrer" className={styles.appButton}>
            <img src={button.icon} alt={button.alt} />
          </a>
        ))}
      </div>
    </div>
  );
};

export default AppMenu;

