import React, { useState } from 'react';
import AppMenu from '@components/app-menu/app-menu';
import EchoChat from '@components/echo/echo-chat/echo-chat';
import Echo from '@components/echo/echo';
import ChatInput from '@components/chat-input/chat-input';
import baseScreensConfig from '@components/echo/screens-config';
import styles from './index.module.scss';
import powerOn from '@assets/images/echoChimp_1.png';
import powerOff from '@assets/images/echoChimp_0.png';

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(false);
  const [showEchoChat, setShowEchoChat] = useState(false);

  const toggleUIVisibility = () => {
    setIsUIVisible(!isUIVisible);
    if (!isUIVisible) {
      setShowEchoChat(false); // Resets to base Echo screen when turning off
    }
  };

  const toggleToEchoChat = () => {
    setShowEchoChat(true);
  };

  const closeEchoScreen = () => {
    setShowEchoChat(false);
  };

  return (
    <div className={styles.backgroundImage}>
      <div className={styles.powerButtonContainer}> {/* Use a container class for the div */}
        <button
          onClick={toggleUIVisibility}
          className={styles.powerButton} // Ensure the button has its own styles
        >
          <img src={isUIVisible ? powerOff : powerOn} alt="Power Button" className={styles.powerButtonImage} />
          <span>{isUIVisible ? 'Deactivate ECHO' : 'Activate ECHO'}</span>
        </button>
      </div>
      <div className={styles.bottomUIContainer}>
        {isUIVisible && (
          <>
            <AppMenu toggleEchoVisibility={toggleToEchoChat} closeEchoScreen={closeEchoScreen} />
            {showEchoChat ? <EchoChat /> : <Echo screensConfig={baseScreensConfig} defaultScreenKey="Nexus" />}
            <ChatInput />
          </>
        )}
      </div>
    </div>
  );
};

export default Home;




