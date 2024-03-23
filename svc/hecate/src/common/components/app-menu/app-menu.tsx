import React, { useState, useEffect } from 'react';
import styles from './app-menu.module.scss';
import ButtonWrapper2 from '@components/index-button-wrapper/button-wrapper-2';
import EchoChat from '@components/echo/echo-chat/echo-chat';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';

interface AppMenuProps {
  toggleEchoVisibility?: () => void; // Making it optional
  showDefaultEchoScreen?: boolean;
}

const AppMenu: React.FC<AppMenuProps> = ({ toggleEchoVisibility, showDefaultEchoScreen = false }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isEchoVisible, setIsEchoVisible] = useState(showDefaultEchoScreen); // Initialize based on prop

  useEffect(() => {
    // React to changes in showDefaultEchoScreen prop
    setIsEchoVisible(showDefaultEchoScreen);
  }, [showDefaultEchoScreen]);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  const toggleEcho = () => {
    setIsEchoVisible(!isEchoVisible);
    toggleEchoVisibility?.(); // Optionally call if exists
  };

  const buttons = [
    { href: 'https://twitter.com/MoxiSKeeper', icon: XLogo, alt: 'Twitter Logo' },
    { href: 'https://discord.com', icon: discordLogo, alt: 'Discord Logo' },
    { href: 'https://telegram.org', icon: telegramLogo, alt: 'Telegram Logo' },
  ];

  return (
    <div>
      {isEchoVisible && <EchoChat />}
      <div className={styles.appMenu}>
        <ButtonWrapper2 title="Toggle ECHO" buttonText="EChat" setCurrentScreen={toggleEcho} />
        <ButtonWrapper2 title="Toggle Social Media Menu" buttonText="Social Media" setCurrentScreen={toggleMenu} />
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


