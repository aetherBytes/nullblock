import React from 'react';
import styles from '@components/echo/styles.module.scss';
import Popup from '@components/popup-content/popup-content';
// Define props interface if using TypeScript
interface IUnifiedEchoScreenProps {
  screenTitle: string;
  images: {
    main: string;
    small?: string;
  };
  isPopupVisible: boolean;
  onClosePopup: () => void;
  content: string;
  additionalContent?: string[];
  popupContent?: React.ReactNode;
}

const UnifiedEchoScreen: React.FC<IUnifiedEchoScreenProps> = ({
  screenTitle,
  images,
  isPopupVisible,
  onClosePopup,
  content,
  additionalContent = [],
  popupContent,
}) => (
  <div className={styles.echoScreen}>
    <div className={styles.contentWrapper}>
      {/* Main image */}
      {images.main && <img src={images.main} alt="Main visual" className={styles.echoImage} />}

      {/* Title */}
      <h2 className={styles.echoTitle}>{screenTitle}</h2>

      {/* Content paragraphs */}
      <p className={styles.echoContent}>{content}</p>
      {additionalContent.map((text, index) => (
        <p key={index} className={styles.echoContent}>
          {text}
        </p>
      ))}

      {/* Optional small image */}
      {images.small && (
        <img src={images.small} alt="Small visual" className={styles.echoImageSmall} />
      )}
    </div>

    {/* Popup */}
    {isPopupVisible && popupContent && <Popup onClose={onClosePopup}>{popupContent}</Popup>}
  </div>
);

export default UnifiedEchoScreen;
