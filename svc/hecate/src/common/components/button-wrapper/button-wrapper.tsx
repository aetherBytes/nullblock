import React from 'react';
import PropTypes from 'prop-types';
import styles from './button-wrapper.module.scss'; // Correct path to your SCSS module

const ButtonWrapper = ({ title, buttonImage, buttonText, setCurrentScreen }) => {
  const createRipple = (event) => {
    const circle = document.createElement("span");
    const diameter = Math.max(event.currentTarget.clientWidth, event.currentTarget.clientHeight);
    const radius = diameter / 2;

    circle.style.width = circle.style.height = `${diameter}px`;
    circle.style.left = `${event.clientX - event.currentTarget.offsetLeft - radius}px`;
    circle.style.top = `${event.clientY - event.currentTarget.offsetTop - radius}px`;
    circle.classList.add("ripple");

    const rippleContainer = event.currentTarget;
    rippleContainer.appendChild(circle);
    setTimeout(() => {
      circle.remove();
    }, 600); // Adjust timing as needed
  };

  return (
    <div className={styles.wrapper}> {/* Adjust className as needed */}
      <button
        type="button"
        className={styles.powerButton} // Ensure this is the correct class for your button
        onClick={(e) => {
          setCurrentScreen();
          createRipple(e);
        }}
      >
        {buttonImage && <img src={buttonImage} alt={title} className={styles.echoButtonImg} />}
        {buttonText}
      </button>
    </div>
  );
};

ButtonWrapper.propTypes = {
  title: PropTypes.string.isRequired,
  buttonText: PropTypes.string,
  buttonImage: PropTypes.string,
  setCurrentScreen: PropTypes.func.isRequired,
};

export default ButtonWrapper;

