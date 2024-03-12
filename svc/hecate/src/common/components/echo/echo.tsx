import React, { useState } from 'react';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import PopupContent from '@components/popup-content/popup-content';
import UnifiedEchoScreen from './echo-screen';
import styles from './styles.module.scss';
import screensConfig from './screens-config'; // Adjust the path as necessary

const Echo = () => {
  const [currentScreen, setCurrentScreen] = useState('Dashboard');
  const [showPopup, setShowPopup] = useState(false);
  const [popupContent, setPopupContent] = useState(null);

  const handleButtonClick = (screen) => {
    const screenConfig = screensConfig[screen];
    if (screenConfig.usePopup) {
      setPopupContent(screenConfig.content);
      setShowPopup(true);
    } else {
      setShowPopup(false);
    }
    setCurrentScreen(screen);
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
          screenTitle={screensConfig[currentScreen].title}
          images={{
            main: screensConfig[currentScreen].image,
            small: screensConfig[currentScreen].image_small,
          }}
          isPopupVisible={showPopup}
          onClosePopup={handleClosePopup}
          content={screensConfig[currentScreen].content}
          popupContent={popupContent}
        />
      </div>
    </div>
  );
};

export default Echo;

