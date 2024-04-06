
import React, { useState, useEffect } from 'react';
import Spline from '@splinetool/react-spline';
import AppMenu from '@components/echo-screens/home-screen/app-menu/app-menu';
import EchoChat from '@components/echo-screens/home-screen/echo-chat/echo-chat';
import Echo from '@components/echo-screens/home-screen/echo';
import ChatInput from '@components/echo-screens/home-screen/chat-input/chat-input';
import baseScreensConfig from '@components/echo-screens/home-screen/screens-config';
import styles from './index.module.scss';
import powerOn from '@assets/images/echo_bot.png';
import powerOff from '@assets/images/echo_bot_black.png';

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(true); // Default to true
  const [showEchoChat, setShowEchoChat] = useState(false); // Initially false
  const [showAppMenu, setShowAppMenu] = useState(false); // Initially false

  useEffect(() => {
    const updateFogEffect = (e) => {
      const fogOverlay = document.getElementById('fogOverlay');
      const x = e.clientX;
      const y = e.clientY;
      fogOverlay.style.background = `radial-gradient(circle at ${x}px ${y}px, transparent 100px, rgba(0, 0, 0, 0.25) 150px)`;
    };

    const fogOverlay = document.getElementById('fogOverlay');
    fogOverlay.style.backgroundColor = 'rgba(0, 0, 0, 0.25)';

    window.addEventListener('mousemove', updateFogEffect);

    return () => window.removeEventListener('mousemove', updateFogEffect);
  }, []);

  const toggleUIVisibility = () => {
    setShowEchoChat(false); // Close Echo screens
    setShowAppMenu(false); // Close App menu
  };

  const toggleToEchoChat = () => {
    setShowEchoChat(true);
    setShowAppMenu(false); // Close App menu when Echo chat is opened
  };

  const closeEchoScreen = () => {
    setShowEchoChat(false);
    setShowAppMenu(false); // Close App menu when Echo chat is closed
  };

  const toggleAppMenu = () => {
    setShowAppMenu(!showAppMenu);
    setShowEchoChat(false); // Close Echo chat when App menu is opened
  };

  return (
    <>
      <div className={styles.backgroundImage}></div>
      <div id="fogOverlay" className={styles.fogOverlay}></div>
      <Spline className={styles.splineObject} scene="https://prod.spline.design/1Q-qMj7C6kFIofgB/scene.splinecode" />
      <div className={styles.powerButtonContainer}>
        <button
          onClick={toggleUIVisibility}
          className={styles.powerButton}
        >
          <img src={isUIVisible ? powerOn : powerOff} alt="Power button" className={styles.powerButtonImage} />
          <span>{isUIVisible ? 'Deactivate' : 'Activate'}</span>
        </button>
      </div>
      <div className={styles.bottomUIContainer}>
        {isUIVisible && (
          <>
            <AppMenu
              toggleEchoVisibility={toggleToEchoChat}
              closeEchoScreen={closeEchoScreen}
              showAppMenu={showAppMenu} // Pass showAppMenu state to AppMenu
            />
            {showEchoChat && <EchoChat />}
            <ChatInput />
          </>
        )}
      </div>
    </>
  );
};

export default Home;

