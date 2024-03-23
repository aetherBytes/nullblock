import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi'; // Adjust the import path as needed
import AppMenu from '@components/app-menu/app-menu'; // Adjust the import path as needed
import ChatInput from '@components/chat-input/chat-input'; // Ensure the import path is correct
import ButtonWrapper from '@components/button-wrapper/button-wrapper'; // Ensure the import path is correct
import styles from './index.module.scss'; // Ensure the path is correct for your styles

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(true); // Controls the visibility of all UI elements

  const toggleUIVisibility = () => {
    setIsUIVisible(!isUIVisible);
  };

  return (
    <div className={styles.backgroundImage}>
      <Moxi />
      <ButtonWrapper
        title="Main Power"
        buttonText={isUIVisible ? 'Turn Off' : 'Turn On'}
        setCurrentScreen={toggleUIVisibility}
      />
      {isUIVisible && (
        <>
          <div className={styles.bottomUIContainer}>
            <AppMenu toggleEchoVisibility={() => {}} />
            <ChatInput />
          </div>
        </>
      )}
    </div>
  );
};

export default Home;

