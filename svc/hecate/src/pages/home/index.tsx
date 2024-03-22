import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi'; // Adjust the import path as needed
import Echo from '@components/echo/echo'; // Adjust the import path as needed
import AppMenu from '@components/app-menu/app-menu'; // Adjust the import path as needed
import styles_buttons from '@components/index-button-wrapper/styles.module.scss'; // Ensure the path is correct for your styles
import menu_white from '@assets/images/menu_echo.png'; // Adjust the import path as needed
import ButtonWrapper from '@components/index-button-wrapper/button-wrapper'; // Adjust the import path as needed
import styles from './index.module.scss'; // Ensure the path is correct for your styles
import ChatInput from '@components/chat-input/chat-input'; // Ensure the import path is correct

const Home = () => {
  const [isEchoVisible, setIsEchoVisible] = useState(false);

  const toggleEchoVisibility = () => setIsEchoVisible(prev => !prev);

  return (
    <div className={styles.backgroundImage}>
      <Moxi toggleEchoVisibility={toggleEchoVisibility} />
      <div className={styles.bottomUIContainer}>
        <AppMenu />
        <ChatInput />
      </div>
      <div className={styles_buttons.buttonsContainer}>
        <ButtonWrapper title="ECHO" buttonImage={menu_white} setCurrentScreen={toggleEchoVisibility} />
      </div>
      {isEchoVisible && <Echo />}
    </div>
  );
};

export default Home;

