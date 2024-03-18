import React, { useState } from 'react';
import PropTypes from 'prop-types';
import XLogo from '@assets/images/X_logo_black.png';
import discordLogo from '@assets/images/discord_logo_black.png';
import telegramLogo from '@assets/images/telegram_logo_black.png';
import styles from './styles.module.scss';

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
      <button className={styles.appButton} onClick={toggleMenu}>
        Apps
      </button>
      {isMenuOpen && (
        <div className={styles.menuContainer}>
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
      )}
    </div>
  );
};

AppMenu.propTypes = {
  toggleEchoVisibility: PropTypes.func.isRequired,
};

export default AppMenu;
