import React from 'react';
import styles from './styles.module.scss';
import MoxiImage from '@assets/images/moxi_lets_go.png';

const Moxi: React.FC = () => {
  return (
    <div className={styles.characterContainer}>
      {/* Update the path to your character's image */}
      <img src={MoxiImage}/>
    </div>
  );
};

export default Moxi;
