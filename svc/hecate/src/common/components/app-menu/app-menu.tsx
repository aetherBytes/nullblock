import React, { useState } from 'react';
import PropTypes from 'prop-types';
import styles from './styles.module.scss';
import ButtonWrapper2 from '@components/index-button-wrapper/button-wrapper-2';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';
// Import the image for the button face
import echoButtonImage from '@assets/images/menu.png'; // Ensure this path is correct

const AppMenu = ({ toggleEchoVisibility }) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const toggleMenu = () => {
    setIsMenuOpen(prevState => !prevState);
  };

  const buttons = [
    { href: 'https://twitter.com/MoxiSKeeper', icon: XLogo, alt: 'Twitter Logo' },
    { href: 'https://discord.com', icon: discordLogo, alt: 'Discord Logo' },
    { href: 'https://telegram.org', icon: telegramLogo, alt: 'Telegram Logo' },
  ];

  return (
    <div className={styles.appMenu}>
      {/* Use ButtonWrapper with an image */}
      <ButtonWrapper2 title="Social Medial Links" buttonText="Socials" setCurrentScreen={toggleMenu}/>
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
        {/* You can still use the regular button for other actions, like toggling the ECHO UI */}
      </div>
    </div>
  );
};

AppMenu.propTypes = {
  toggleEchoVisibility: PropTypes.func.isRequired,
};

export default AppMenu;

