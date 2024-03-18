
import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi'; // Adjust the import path as needed
import Echo from '@components/echo/echo'; // Adjust the import path as needed
import AppMenu from '@components/app-menu/app-menu'; // Adjust the import path as needed
import styles from './styles.module.scss'; // Ensure the path is correct for your styles
import moxi_black from '@assets/images/moxi_avatar_white.png'; // Adjust the import path as needed

const Home = () => {
  const [isEchoVisible, setIsEchoVisible] = useState(true);

  const toggleEchoVisibility = () => setIsEchoVisible(prev => !prev);

  return (
    <div className={styles.backgroundImage}>
      <Moxi toggleEchoVisibility={toggleEchoVisibility} />
      <div className={styles.buttonsContainer}>
        <AppMenu toggleEchoVisibility={toggleEchoVisibility} /> {/* Pass the toggleEchoVisibility prop */}
        <button onClick={toggleEchoVisibility} className={styles.echoButton}>
          <img src={moxi_black} alt="Moxi Logo" className={styles.echoImage} />
        </button>
      </div>
      {isEchoVisible && <Echo />}
    </div>
  );
};

export default Home;

