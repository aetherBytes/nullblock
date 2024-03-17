import React, { useState } from 'react';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import UnifiedEchoScreen from './echo-screen';
import screensConfig from './screens-config'; // Ensure this path is correct
import styles from './styles.module.scss';

const Echo = () => {
  const [currentScreen, setCurrentScreen] = useState('Dashboard');
  const [showPopup, setShowPopup] = useState(false);
  const [popupContent, setPopupContent] = useState(null);
  const [isEchoVisible, setIsEchoVisible] = useState(true); // State to control visibility, remains unchanged here
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
    // Removed the toggling of isEchoVisible to keep the Echo screen always visible when changing screens
    setAnimationKey(Date.now()); // Use this to trigger animations or re-renders as needed
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
        {/* Conditional rendering based on isEchoVisible state, which now remains true unless externally modified */}
        {isEchoVisible && (
          <UnifiedEchoScreen
            key={animationKey}
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
        )}
      </div>
    </div>
  );
};

export default Echo;

