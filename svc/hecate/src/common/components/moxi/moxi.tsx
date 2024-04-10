
import React from 'react';
import MoxiImage from '@assets/images/night_wolf_1.png';
import styles from './styles.module.scss';


const Moxi: React.FC<MoxiProps> = () => {

  return (
    <div className={styles.characterContainer}>
      <img src={MoxiImage} alt="Moxi" />
    </div>
  );
};

export default Moxi;

