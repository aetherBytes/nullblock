import React from 'react';
import styles from '@components/echo/echo.module.scss';
import Popup from '@components/popup-content/popup-content';

interface IUnifiedEchoScreenProps {
  screenTitle: string;
  images?: {
    main: string;
    small?: string;
  };
  isPopupVisible: boolean;
  onClosePopup: () => void;
  content: string;
  additionalContent?: string[];
  popupContent?: React.ReactNode;
  closeCurrentScreen: () => void; // Add prop for closing current screen
}

const UnifiedEchoScreen: React.FC<IUnifiedEchoScreenProps> = ({
  screenTitle,
  images = { main: '', small: '' },
  isPopupVisible,
  onClosePopup,
  content,
  additionalContent = [],
  popupContent,
  closeCurrentScreen, // Receive closeCurrentScreen function from parent
}) => {
  const handleButtonClick = () => {
    closeCurrentScreen(); // Close current screen before opening a new one
    // Additional logic for handling button click if needed
  };

  return (
    <div className={styles.echoScreen}>
      <div className={styles.contentWrapper}>
        {images.main && <img src={images.main} alt="Main visual" className={styles.echoImage} />}
        <h2 className={styles.echoTitle}>{screenTitle}</h2>
        <p className={styles.echoContent}>{content}</p>
        {additionalContent.map((text, index) => (
          <p key={index} className={styles.echoContent}>
            {text}
          </p>
        ))}
        {images.small && (
          <img src={images.small} alt="Small visual" className={styles.echoImageSmall} />
        )}
      </div>
      {isPopupVisible && popupContent && <Popup onClose={onClosePopup}>{popupContent}</Popup>}
    </div>
  );
};

export default UnifiedEchoScreen;

