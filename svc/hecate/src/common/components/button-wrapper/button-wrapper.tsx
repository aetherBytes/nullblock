// ButtonWrapper.tsx
import React from 'react';
import styles from './styles.module.scss';

interface ButtonWrapperProps {
  title: string;
  buttonText: string;
  setCurrentScreen: (screen: string) => void;
}

const ButtonWrapper: React.FC<ButtonWrapperProps> = ({ title, buttonText, setCurrentScreen }) => (
  <div>
    <button type="button" className={styles.echoButton} onClick={() => setCurrentScreen(title)}>
      {buttonText}
    </button>
  </div>
);

export default ButtonWrapper;
