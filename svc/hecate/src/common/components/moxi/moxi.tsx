// Import React
import React from 'react';
import MoxiImage from '@assets/images/moxi_lets_go.png'; // Ensure the path is correct
import styles from './styles.module.scss'; // Adjust the import path as needed

// Add a prop type for the new onToggleEcho function
interface MoxiProps {
  onToggleEcho: () => void;
}

const Moxi: React.FC<MoxiProps> = ({ onToggleEcho }) => (
  <div className={styles.characterContainer} onClick={onToggleEcho} style={{ cursor: 'pointer' }}>
    <img src={MoxiImage} alt="Moxi" />
  </div>
);

export default Moxi;

