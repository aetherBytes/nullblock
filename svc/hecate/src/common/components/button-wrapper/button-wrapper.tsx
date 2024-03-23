import React from 'react';
import PropTypes from 'prop-types';
import styles from './button-wrapper.module.scss'; // Ensure this is the correct path

const ButtonWrapper = ({ title, buttonImage, buttonText, setCurrentScreen }) => {
  const createRipple = (event) => {
    const circle = document.createElement("span");
    const diameter = Math.max(event.currentTarget.clientWidth, event.currentTarget.clientHeight);
    const radius = diameter / 2;

    circle.style.width = circle.style.height = `${diameter}px`;
    circle.style.left = `${event.clientX - event.currentTarget.offsetLeft - radius}px`;
    circle.style.top = `${event.clientY - event.currentTarget.offsetTop - radius}px`;
    circle.classList.add(styles.ripple); // Ensured class addition through module

    const rippleContainer = event.currentTarget;
    rippleContainer.appendChild(circle);
    setTimeout(() => {
      circle.remove();
    }, 600); // Ensured ripple timing
  };

  return (
    <div className={styles.wrapper}> {/* Confirm className alignment */}
      <button
        type="button"
        className={styles.echoButton} // Updated to echoButton for consistency
        onClick={(e) => {
          setCurrentScreen();
          createRipple(e);
        }}
      >
        {buttonImage && <img src={buttonImage} alt={title} className={styles.buttonImage} />} {/* Adjusted className */}
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

