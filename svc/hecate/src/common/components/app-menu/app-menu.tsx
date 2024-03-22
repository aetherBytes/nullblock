import React, { useState } from 'react';
import PropTypes from 'prop-types';
import styles from './app-menu.module.scss';
import ButtonWrapper2 from '@components/index-button-wrapper/button-wrapper-2';
import Echo from '@components/echo/echo'; // Assuming Echo component is properly exported and path is correct
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';

const AppMenu = ({ toggleEchoVisibility }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isEchoVisible, setIsEchoVisible] = useState(false); // State to control the visibility of the ECHO

  const toggleMenu = () => {
    setIsMenuOpen(prevState => !prevState);
  };

  // Function to toggle ECHO visibility
  const toggleEcho = () => {
    setIsEchoVisible(prevState => !prevState);
  };

  const buttons = [
    { href: 'https://twitter.com/MoxiSKeeper', icon: XLogo, alt: 'Twitter Logo' },
    { href: 'https://discord.com', icon: discordLogo, alt: 'Discord Logo' },
    { href: 'https://telegram.org', icon: telegramLogo, alt: 'Telegram Logo' },
  ];

  return (
    <div>
      {/* Conditionally render the Echo component based on isEchoVisible state */}
      {isEchoVisible && <Echo />}

      <div className={styles.appMenu}>
        {/* Add EChat button for toggling the ECHO interface */}
        <ButtonWrapper2 title="Toggle ECHO" buttonText="EChat" setCurrentScreen={toggleEcho} />
        <ButtonWrapper2 title="Toggle Social Media Menu" buttonText="Social Media" setCurrentScreen={toggleMenu} />
        <div className={`${styles.menuContainer} ${isMenuOpen ? styles.active : ''}`}>
          {buttons.map((button, index) => (
            <a
              key={index}
              href={button.href}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.appButton}
            >
              <img src={button.icon} alt={button.alt} />
            </a>
          ))}
        </div>
      </div>
    </div>
  );
};

AppMenu.propTypes = {
  toggleEchoVisibility: PropTypes.func.isRequired,
};

export default AppMenu;


