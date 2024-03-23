import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi'; // Adjust the import path as needed
import AppMenu from '@components/app-menu/app-menu'; // Adjust the import path as needed
import styles from './index.module.scss'; // Ensure the path is correct for your styles
import ChatInput from '@components/chat-input/chat-input'; // Ensure the import path is correct

const Home = () => {
  // State to control the visibility of the Echo feature/UI
  const [isEchoVisible, setIsEchoVisible] = useState<boolean>(false);

  // Function to toggle the visibility state
  const toggleEchoVisibility = () => setIsEchoVisible(!isEchoVisible);

  return (
    <div className={styles.backgroundImage}>
      <Moxi />
      <div className={styles.bottomUIContainer}>
        <AppMenu toggleEchoVisibility={toggleEchoVisibility} />
        <ChatInput />
      </div>
    </div>
  );
};

export default Home;

