import React, { useState, useEffect } from 'react';
import Spline from '@splinetool/react-spline';
import AppMenu from '@components/app-menu/app-menu';
import EchoChat from '@components/echo-screens/home-screen/echo-chat/echo-chat';
import Echo from '@components/echo-screens/home-screen/echo';
import ChatInput from '@components/chat-input/chat-input';
import baseScreensConfig from '@components/echo-screens/home-screen/screens-config';
import styles from './index.module.scss';
import powerOn from '@assets/images/echo_bot.png';
import powerOff from '@assets/images/echo_bot_black.png';

const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(false);
  const [showEchoChat, setShowEchoChat] = useState(false);

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
    <>
      <div className={styles.backgroundImage}></div>
      <div id="fogOverlay" className={styles.fogOverlay}></div>
      {/* Spline object is positioned absolutely and on top of all other content */}
      <Spline className={styles.splineObject} scene="https://prod.spline.design/1Q-qMj7C6kFIofgB/scene.splinecode" />
      <div className={styles.powerButtonContainer}> {/* Use a container class for the div */}
        <button
          onClick={toggleUIVisibility}
          className={styles.powerButton} // Ensure the button has its own styles
        >
          <img src={isUIVisible ? powerOn : powerOff} alt="Power button" className={styles.powerButtonImage}/>
          <span>{isUIVisible ? 'Deactivate' : 'Activate'}</span>
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
    </>
  );
};

export default Home;

