
import React, { useState } from 'react';
import styles from './app-menu.module.scss';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import EchoChat from '@components/echo/echo-chat/echo-chat';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';

interface AppMenuProps {
  toggleEchoVisibility?: () => void; // Making it optional
  showDefaultEchoScreen?: boolean;
  closeEchoScreen: () => void; // Function to close Echo screen
  toggleAppMenuVisibility: () => void; // Function to toggle App Menu
}

const AppMenu: React.FC<AppMenuProps> = ({ toggleEchoVisibility, showDefaultEchoScreen = false, closeEchoScreen, toggleAppMenuVisibility }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  const toggleEcho = () => {
    closeEchoScreen(); // Close existing Echo screen
    toggleEchoVisibility?.(); // Optionally call if exists
    toggleAppMenuVisibility(); // Close App menu when toggling Echo
  };

  const buttons = [
    { href: 'https://twitter.com/MoxiSKeeper', icon: XLogo, alt: 'Twitter Logo' },
    { href: 'https://discord.com', icon: discordLogo, alt: 'Discord Logo' },
    { href: 'https://telegram.org', icon: telegramLogo, alt: 'Telegram Logo' },
  ];

  return (
    <div>
      <div className={styles.appMenu}>
        <ButtonWrapper title="Toggle ECHO" buttonText="EChat" setCurrentScreen={toggleEcho} />
        <ButtonWrapper title="Toggle Social Media Menu" buttonText="Social Media" setCurrentScreen={toggleMenu} />
        <div className={`${styles.menuContainer} ${isMenuOpen ? styles.active : ''}`}>
          {buttons.map((button, index) => (
            <a key={index} href={button.href} target="_blank" rel="noopener noreferrer" className={styles.appButton}>
              <img src={button.icon} alt={button.alt} />
            </a>
          ))}
        </div>
      </div>
    </div>
  );
};

export default AppMenu;

