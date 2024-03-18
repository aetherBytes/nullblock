
import React from 'react';
import MoxiImage from '@assets/images/moxi_lets_go.png';
import styles from './styles.module.scss';


const Moxi: React.FC<MoxiProps> = () => {

  return (
    <div className={styles.characterContainer}>
      <img src={MoxiImage} alt="Moxi" />
    </div>
  );
};

export default Moxi;

