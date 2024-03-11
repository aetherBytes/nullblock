import React, { useRef, useEffect } from 'react';
import styles from '@components/echo/styles.module.scss';

interface IScreenWrapperProps {
  title: string;
  image: string; // Changed to string from HTMLImageElement
  image_small?: string; // Optional image_small
  isPopupVisible: boolean; // Pass this prop down to ScreenWrapper
  content: string;
  content_2: string;
  content_3: string;
}

const ScreenWrapper: React.FC<IScreenWrapperProps> = ({
  title,
  image,
  image_small,
  image_throne,
  isPopupVisible,
  content,
  content_2,
  content_3
}) => {
  const imageRef = useRef<HTMLImageElement>(null);

  useEffect(() => {


    const handleMouseMove = (event: MouseEvent) => {
      console.log("Mouse move event triggered");
      if (imageRef.current) {
        const rect = imageRef.current.getBoundingClientRect();
        const imageCenterX = rect.left + rect.width / 2;
        const imageCenterY = rect.top + rect.height / 2;

        // Calculate the distance of the mouse from the center of the image
        const distanceX = event.clientX - imageCenterX;
        const distanceY = event.clientY - imageCenterY;

        // Calculate the maximum distance from the center (half of the image size)
        const maxDistanceX = rect.width / 2;
        const maxDistanceY = rect.height / 2;

        // Calculate the percentage of movement based on the distance from the center
        const movePercentageX = distanceX / maxDistanceX;
        const movePercentageY = distanceY / maxDistanceY;

        // Calculate the maximum movement (in pixels)
        const maxMove = 10;

        // Calculate the actual movement based on the percentage and maximum movement
        const moveX = maxMove * movePercentageX;
        const moveY = maxMove * movePercentageY;

        // Apply the transformation
        imageRef.current.style.transform = `translate(${moveX}px, ${moveY}px)`;
      }
    };



    const handleMouseLeave = () => {
      if (imageRef.current) {
        imageRef.current.style.transform = 'translate(0, 0)';
      }
    };

    if (isPopupVisible) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseleave', handleMouseLeave);
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [isPopupVisible]); // Add isPopupVisible as a dependency

  return (
    <div className={isPopupVisible ? `${styles.echoScreenWithPopup}` : styles.echoScreen}>
      <div className={styles.contentWrapper}>
        {image && <img ref={imageRef} src={image} className={styles.echoImage} />}
        {image_throne && <img src={image_throne} className={styles.echoImageThrone} />}
        <h2 className={styles.echoTitle}>{title}</h2>
        <p className={styles.echoContent}>{content}</p>
        {image_small && <img src={image_small} className={styles.echoImageSmall} />}
        <p className={styles.echoContent}>{content_2}</p>
        <p className={styles.echoContent}>{content_3}</p>
      </div>
    </div>
  );
};

export default ScreenWrapper;
