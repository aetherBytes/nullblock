
import React, { useState } from 'react';
import Moxi from '@components/moxi/parent';
import Echo from '@components/echo/echo';
import AppMenu from '@components/app-menu/app-menu';
import styles from './styles.module.scss';
import moxi_black from '@assets/images/moxi_avatar_black_200.png';

const Home = () => {
  const [isEchoVisible, setIsEchoVisible] = useState(false);

  const toggleEchoVisibility = () => setIsEchoVisible(prev => !prev);

  return (
    <div className={styles.backgroundImage}>
      <Moxi toggleEchoVisibility={toggleEchoVisibility} />
      <div className={styles.buttonsContainer}>
        <AppMenu toggleEchoVisibility={toggleEchoVisibility} />
        <button onClick={toggleEchoVisibility} className={styles.echoButton}>
          <img src={moxi_black} alt="Moxi Logo" />
        </button>
      </div>
      {isEchoVisible && <Echo />}
    </div>
  );
};

export default Home;

