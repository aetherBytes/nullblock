import React from 'react';
import styles from '@components/echo/styles.module.scss'; // Adjust the import path as necessary

const ButtonWrapper = ({ title, buttonText, setCurrentScreen }) => (
  <div style={{ width: 'calc(100% - 2rem)' }}>
    <button type="button" className={styles.echoButton} onClick={() => setCurrentScreen(title)}>
      {buttonText}
    </button>
  </div>
);

export default ButtonWrapper;
