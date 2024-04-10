
import React, { useState, useEffect } from 'react';
import AppMenu from '@components/echo-screens/home-screen/app-menu/app-menu';
import EchoChat from '@components/echo-screens/home-screen/echo-chat/echo-chat';
import ChatInput from '@components/echo-screens/home-screen/chat-input/chat-input';
import styles from './index.module.scss';
import powerOn from '@assets/images/echo_bot_night.png';
import powerOff from '@assets/images/echo_bot_white.png';
import Moxi from '@components/moxi/moxi'; // Import Moxi component

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(true);
  const [showEchoChat, setShowEchoChat] = useState(false);

  useEffect(() => {
    // Your useEffect code here
  }, []);

  const toggleUIVisibility = () => {
    setIsUIVisible(!isUIVisible);
    if (!isUIVisible) {
      setShowEchoChat(false);
    }
    setShowEchoChat(true);
  };

  return (
    <>
      <div className={styles.backgroundImage}></div>
      <div id="fogOverlay" className={styles.fogOverlay}></div>
      <div className={styles.powerButtonContainer}>
        <button onClick={toggleUIVisibility} className={styles.powerButton}>
          <img src={isUIVisible ? powerOn : powerOff} alt="Power button" className={styles.powerButtonImage} />
          <span> {isUIVisible ? 'Turn off' : 'Turn on'
          }</span>
        </button>
      </div>
      <div className={styles.bottomUIContainer}>
        {/* Render Moxi component always */}
        <Moxi />

        {isUIVisible && (
          <>
            <AppMenu
              toggleEchoVisibility={() => setShowEchoChat(!showEchoChat)}
              closeEchoScreen={() => setShowEchoChat(false)}
              isUIVisible={isUIVisible}
            />
            {showEchoChat && <EchoChat />}
          </>
        )}
        <ChatInput />
      </div>
    </>
  );
};

export default Home;


