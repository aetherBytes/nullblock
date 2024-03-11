import React from 'react';
import styles from '@components/echo/styles.module.scss';

interface IScreenWrapperProps {
  title: string;
  image: string;
  image_small?: string;
  image_throne?: string;
  isPopupVisible: boolean;
  content: string;
  content_2: string;
  content_3: string;
}

const ScreenWrapper: React.FC<IScreenWrapperProps> = ({
  title,
  image,
  image_small,
  isPopupVisible,
  content,
  content_2,
  content_3
}) => {
  return (
    <div className={isPopupVisible ? `${styles.echoScreenWithPopup}` : styles.echoScreen}>
      <div className={styles.contentWrapper}>
        {/* Main image */}
        <img src={image} alt="Main visual" className={styles.echoImage} />

        {/* Title */}
        <h2 className={styles.echoTitle}>{title}</h2>

        {/* Content paragraphs */}
        <p className={styles.echoContent}>{content}</p>
        <p className={styles.echoContent}>{content_2}</p>
        <p className={styles.echoContent}>{content_3}</p>

        {/* Optional small image */}
        {image_small && <img src={image_small} alt="Small visual" className={styles.echoImageSmall} />}
      </div>
    </div>
  );
};

export default ScreenWrapper;

