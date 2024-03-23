import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi'; // Adjust the import path as needed
import AppMenu from '@components/app-menu/app-menu'; // Adjust the import path as needed
import styles_buttons from '@components/index-button-wrapper/styles.module.scss'; // Ensure the path is correct for your styles
import ButtonWrapper from '@components/index-button-wrapper/button-wrapper'; // Adjust the import path as needed
import styles from './index.module.scss'; // Ensure the path is correct for your styles
import ChatInput from '@components/chat-input/chat-input'; // Ensure the import path is correct

const Home = () => {
  return (
    <div className={styles.backgroundImage}>
      <Moxi />
      <div className={styles.bottomUIContainer}>
        <AppMenu />
        <ChatInput />
      </div>
    </div>
  );
};

export default Home;

