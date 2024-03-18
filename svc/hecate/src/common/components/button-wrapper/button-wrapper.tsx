import React from 'react';
import PropTypes from 'prop-types';
import styles from './styles.module.scss';

const ButtonWrapper = ({ title, buttonImage, buttonText, setCurrentScreen }) => {
  const handleClick = () => {
    setCurrentScreen(); // Invoke the toggleMenu function
  };

  return (
    <div>
      <button type="button" className={styles.echoButton} onClick={handleClick}>
        {buttonImage ? <img src={buttonImage} alt={title} className={styles.echoButtonImg} /> : buttonText}
      </button>
    </div>
  );
};

ButtonWrapper.propTypes = {
  title: PropTypes.string.isRequired,
  buttonText: PropTypes.string,
  buttonImage: PropTypes.string, // Image source URL
  setCurrentScreen: PropTypes.func.isRequired,
};

ButtonWrapper.defaultProps = {
  buttonText: '',
  buttonImage: '',
};

export default ButtonWrapper;

