import React, { useState } from 'react';
import Moxi from '@components/moxi/moxi';
import AppMenu from '@components/app-menu/app-menu';
import ChatInput from '@components/chat-input/chat-input';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import Echo from '@components/echo/echo'; // Adjust import path as needed
import EchoChat from '@components/echo/echo-chat/echo-chat'; // Adjust import path as needed
import baseScreensConfig from '@components/echo/screens-config'; // Adjust path as necessary
import styles from './index.module.scss';
import powerOn from '@assets/images/power-on.png';
import powerOff from '@assets/images/power-off.png';

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(false);
  const [showEchoChat, setShowEchoChat] = useState(false);

  const toggleUIVisibility = () => {
    setIsUIVisible(!isUIVisible);
    if (!isUIVisible) setShowEchoChat(false); // Resets to base Echo screen when turning off
  };

  const toggleToEchoChat = () => {
    setShowEchoChat(true);
  };

  const closeEchoScreen = () => {
    setShowEchoChat(false);
  };

  return (
    <div className={styles.backgroundImage}>
      <Moxi />
      <div className={styles.powerButton}>
        <ButtonWrapper
          title="Main Power"
          buttonText={isUIVisible ? 'Deactivate ECHO' : 'Activate ECHO'}
          // buttonImage={isUIVisible ? '/images/power-off.png' : '/images/power-on.png'}
          setCurrentScreen={toggleUIVisibility}
        />
      </div>
      <div className={styles.bottomUIContainer}> {/* Ensure this div wraps the conditional content */}
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

