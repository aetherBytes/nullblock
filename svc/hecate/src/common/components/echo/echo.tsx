import React from 'react';
import styles from './echo.module.scss'; // Assuming you'll create a corresponding SCSS file

const Echo: React.FC = () => {
  return (
    <div className={styles.echoContainer}>
      <div className={styles.echoScreen}>
        {/* Content of the HUD */}
        <p>Welcome to ECHO</p>
        {/* Add more interactive elements or data display here */}
      </div>
    </div>
  );
};

export default Echo;
