import React, { useState } from 'react';
import ButtonWrapper from '@components/button-wrapper/button-wrapper';
import PopupContent from '@components/popup-content/popup-content';
import EchoScreen from './echo-screen';
import styles from './styles.module.scss';
import babyGoku from '@assets/images/baby_goku_clip.png';

const Echo = () => {
  const [currentScreen, setCurrentScreen] = useState<string>('Dashboard');
  const [showPopup, setShowPopup] = useState<boolean>(false);
  const [popupContent, setPopupContent] = useState<React.ReactNode>(null); // Update to use React.ReactNode

  // Function to handle button clicks to open the popup with specific content
  const handleButtonClick = (popup: boolean, screen: string, content: React.ReactNode) => {
    setCurrentScreen(screen); // Maintain the current screen state
    setPopupContent(content); // Set the popup content based on the button clicked
    setShowPopup(popup); // Show the popup
  };

  // Function to handle closing the popup
  const handleClosePopup = () => {
    setShowPopup(false); // Hide the popup
    // setCurrentScreen('Dashboard'); // Reset to default screen
  };

  return (
    <div className={styles.parentContainer}>
      <div className={styles.buttonsContainer}>
        <ButtonWrapper
          buttonText="WHY?!"
          setCurrentScreen={() => handleButtonClick(false, 'Why', (
            <div>
              {/* Popup content for 'Why?!' button */}
            </div>
          ))}
          title="About"
        />
        <ButtonWrapper
          buttonText="$LORD"
          setCurrentScreen={() => handleButtonClick(false, 'LORD', (
            <div>

              {/* Popup content for 'Why?!' button */}

            </div>
          ))}
          title="LORD"
        />
      </div>
      <EchoScreen screen={currentScreen} isPopupVisible={showPopup} onClosePopup={handleClosePopup} />{' '}
      {/* This ensures the original Echo screen is always displayed */}
      {showPopup && (
        <PopupContent
          onClose={handleClosePopup}
          content={popupContent} // Pass the JSX content directly
        />
      )}
    </div>
  );
};

export default Echo;

