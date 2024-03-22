import React from 'react';
import styles from '@components/echo/echo.module.scss'; // Ensure your styles are set up correctly

const PopupContent = ({ onClose, content, isEchoScreenVisible }) => {
  const popupStyle = {
    transform: isEchoScreenVisible ? 'translate(-50%, -50%)' : 'translate(30%, 0%)',
  };

  return (
    <div className={styles.popupOverlay} onClick={onClose}>
      <div className={styles.popupContent} style={popupStyle} onClick={(e) => e.stopPropagation()}>
        {content}
      </div>
    </div>
  );
};

export default PopupContent;
