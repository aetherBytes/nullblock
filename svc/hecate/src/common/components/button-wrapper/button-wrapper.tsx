import React from 'react';
import styles from './styles.module.scss'; // Adjust the import path as necessary

const ButtonWrapper = ({ title, buttonText, setCurrentScreen }) => (
  <div>
    <button type="button" className={styles.echoButton} onClick={() => setCurrentScreen(title)}>
      {buttonText}
    </button>
  </div>
);

export default ButtonWrapper;
