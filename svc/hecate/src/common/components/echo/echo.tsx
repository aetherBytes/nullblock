import React, { useState, useEffect } from 'react';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import UnifiedEchoScreen from './echo-screen';
import styles from './echo.module.scss';

interface ScreenConfig {
  title: string;
  buttonText: string;
  usePopup: boolean;
  content: JSX.Element;
  popupContent?: JSX.Element;
  image?: string;
  image_small?: string;
  additionalContent?: JSX.Element[];
}

interface EchoProps {
  screensConfig: { [key: string]: ScreenConfig };
  defaultScreenKey: string;
}

const Echo: React.FC<EchoProps> = ({ screensConfig, defaultScreenKey }) => {
  const [currentScreen, setCurrentScreen] = useState<string>(defaultScreenKey);
  const [showPopup, setShowPopup] = useState<boolean>(false);
  const [popupContent, setPopupContent] = useState<JSX.Element | null>(null);
  const [isEchoVisible, setIsEchoVisible] = useState<boolean>(true);
  const [animationKey, setAnimationKey] = useState<number>(Date.now());

  useEffect(() => {
    // Ensure current screen is updated if defaultScreenKey changes,
    // which might be useful if your configs could change dynamically.
    setCurrentScreen(defaultScreenKey);
  }, [defaultScreenKey]);

  const handleButtonClick = (screen: string) => {
    const screenConfig = screensConfig[screen];

    if (screenConfig?.usePopup) {
      setPopupContent(screenConfig.popupContent);
      setShowPopup(true);
    } else {
      setShowPopup(false);
    }

    setCurrentScreen(screen);
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
        {isEchoVisible && (
          <UnifiedEchoScreen
            key={animationKey}
            screenTitle={screensConfig[currentScreen]?.title}
            images={{
              main: screensConfig[currentScreen]?.image,
              small: screensConfig[currentScreen]?.image_small,
            }}
            isPopupVisible={showPopup}
            onClosePopup={handleClosePopup}
            content={screensConfig[currentScreen]?.content}
            additionalContent={screensConfig[currentScreen]?.additionalContent || []}
            popupContent={popupContent}
          />
        )}
      </div>
    </div>
  );
};

export default Echo;

