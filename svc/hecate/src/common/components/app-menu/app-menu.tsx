import React, { useState } from 'react';
import styles from './app-menu.module.scss';
import ButtonWrapper2 from '@components/index-button-wrapper/button-wrapper-2';
import EchoChat from '@components/echo/echo-chat/echo-chat';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';

interface AppMenuProps {
  toggleEchoVisibility?: () => void; // Making it optional
}

const AppMenu: React.FC<AppMenuProps> = ({ toggleEchoVisibility }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isEchoVisible, setIsEchoVisible] = useState(false);

  const toggleMenu = () => {
    setIsMenuOpen(prevState => !prevState);
  };

  const toggleEcho = () => {
    setIsEchoVisible(prevState => !prevState);
    toggleEchoVisibility?.(); // Call if exists
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

