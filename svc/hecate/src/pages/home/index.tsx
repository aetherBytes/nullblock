
import React, { useState, useEffect } from 'react';
import AppMenu from '@components/echo-screens/home-screen/app-menu/app-menu';
import EchoChat from '@components/echo-screens/home-screen/echo-chat/echo-chat';
import ChatInput from '@components/echo-screens/home-screen/chat-input/chat-input';
import styles from './index.module.scss';
import powerOn from '@assets/images/echo_bot_night.png';
import powerOff from '@assets/images/echo_bot_white.png';
import Moxi from '@components/moxi/moxi'; // Import Moxi component
import StarsCanvas from '@components/stars/stars'; // Import StarsCanvas component

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(true);
  const [showEchoChat, setShowEchoChat] = useState(false);

  useEffect(() => {
    // Your useEffect code here
  }, []);


  return (
    <>
      {/* Render the StarsCanvas component as the background */},
      <StarsCanvas/>
      <div className={styles.backgroundImage}/>
      <div className={styles.powerButtonContainer}>
        <button className={styles.powerButton}>
          <img src={isUIVisible ? powerOn : powerOff} alt="Profile button" className={styles.profileButtonImage} />
          <span> {isUIVisible ? 'Profile' : 'Log in'
          }</span>
        </button>
      </div>
      <div className={styles.bottomUIContainer}>
        {/* Render Moxi component always */}
        <Moxi />
        <>{showEchoChat && <EchoChat />}</>
        <AppMenu
              toggleEchoVisibility={() => setShowEchoChat(!showEchoChat)}
              closeEchoScreen={() => setShowEchoChat(false)}
              isUIVisible={isUIVisible}
            />
        <ChatInput />
      </div>
    </>
  );
};

export default Home;



