// ButtonWrapper.jsx
import React from 'react';
import PropTypes from 'prop-types';
import styles from './styles.module.scss';

const ButtonWrapper = ({ title, buttonText, setCurrentScreen }) => {
  const handleClick = () => {
    setCurrentScreen(); // Invoke the toggleMenu function
  };

  return (
    <div>
      <button type="button" className={styles.echoButton} onClick={handleClick}>
        {buttonText}
      </button>
    </div>
  );
};

ButtonWrapper.propTypes = {
  title: PropTypes.string.isRequired,
  buttonText: PropTypes.string.isRequired,
  setCurrentScreen: PropTypes.func.isRequired,
};

export default ButtonWrapper;

