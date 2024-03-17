import React, { useState } from 'react';
import Moxi from './moxi'; // Adjust the import path as needed
import Echo from '@components/echo/echo'; // Adjust the import path as needed
import styles from './styles.module.scss'; // Ensure the path is correct for your styles

const Parent = () => {
  const [isEchoVisible, setIsEchoVisible] = useState(false);
  const [currentScreen, setCurrentScreen] = useState('defaultScreen');

  const toggleEchoVisibility = () => {
    setIsEchoVisible(prevState => !prevState); // Toggles the visibility of Echo
  };

  const handleScreenChange = (screenName) => {
    setCurrentScreen(screenName); // Changes the screen without affecting visibility
  };

  return (
    <div className={styles.parentContainer}>
      <Moxi onToggleEcho={toggleEchoVisibility} />
      {isEchoVisible && (
        <Echo
          currentScreen={currentScreen}
          onChangeScreen={handleScreenChange}
        />
      )}
    </div>
  );
};

export default Parent;
