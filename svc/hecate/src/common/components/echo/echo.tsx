import React, { useState } from 'react';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import UnifiedEchoScreen from './echo-screen';
import screensConfig from './screens-config'; // Adjust the path as necessary
import styles from './styles.module.scss';

const Echo = () => {
  const [currentScreen, setCurrentScreen] = useState('Dashboard');
  const [showPopup, setShowPopup] = useState(false);
  const [popupContent, setPopupContent] = useState(null);
  // Additional state to track the unique key for animations
  const [animationKey, setAnimationKey] = useState(Date.now());

  const handleButtonClick = (screen) => {
    const screenConfig = screensConfig[screen];

    if (screenConfig.usePopup) {
      setPopupContent(screenConfig.content);
      setShowPopup(true);
    } else {
      setShowPopup(false);
    }

    setCurrentScreen(screen);
    // Update the animation key on screen change
    setAnimationKey(Date.now());
  };

  const handleClosePopup = () => setShowPopup(false);

  return (
    <div className={styles.parentContainer}>
      <div className={styles.mainScreenContent}>
        <div className={styles.sidebar}>
          {Object.keys(screensConfig).map((screen) => (
            <ButtonWrapper
              key={screen}
              buttonText={screensConfig[screen].buttonText}
              setCurrentScreen={() => handleButtonClick(screen)}
              title={screensConfig[screen].title}
            />
          ))}
        </div>
        <UnifiedEchoScreen
          key={animationKey} // Use the unique key here
          screenTitle={screensConfig[currentScreen].title}
          images={{
            main: screensConfig[currentScreen].image,
            small: screensConfig[currentScreen].image_small,
          }}
          isPopupVisible={showPopup}
          onClosePopup={handleClosePopup}
          content={screensConfig[currentScreen].content}
          additionalContent={screensConfig[currentScreen].additionalContent || []}
          popupContent={popupContent}
        />
      </div>
    </div>
  );
};

export default Echo;
