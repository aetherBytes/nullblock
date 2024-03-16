import React from 'react';
import MoxiImage from '@assets/images/moxi_lets_go.png';
import styles from './styles.module.scss';

const Moxi: React.FC = () => (
  <div className={styles.characterContainer}>
    {/* Update the path to your character's image */}
    <img src={MoxiImage} />
  </div>
);

export default Moxi;
