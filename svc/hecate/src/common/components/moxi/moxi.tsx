
import React from 'react';
import MoxiImage from '@assets/images/moxi_lets_go.png';
import styles from './styles.module.scss';

interface MoxiProps {
  onToggleEcho: () => void; // Ensure the prop name is onToggleEcho
}

const Moxi: React.FC<MoxiProps> = ({ onToggleEcho }) => {
  const handleClick = () => {
    onToggleEcho(); // Call the onToggleEcho function when Moxi is clicked
  };

  return (
    <div className={styles.characterContainer} onClick={handleClick} style={{ cursor: 'pointer' }}>
      <img src={MoxiImage} alt="Moxi" />
    </div>
  );
};

export default Moxi;

