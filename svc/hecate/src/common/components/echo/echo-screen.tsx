import React from 'react';
import beerusBlast2 from '@assets/images/beerus_blast_2.webp';
import throne from '@assets/images/beerus_throne_clip.png';
import styles from './styles.module.scss';
import ScreenWrapper from '@components/screen-wrapper/screen-wrapper';

interface IEchoScreenProps {
  screen: string;
  isPopupVisible: boolean;
}

const EchoScreen: React.FC<IEchoScreenProps> = ({ screen, isPopupVisible }) => {
  switch (screen) {
    case 'Why':
      return <ScreenWrapper title="Why has The Destroyer arrived?"  isPopupVisible={isPopupVisible} />;
    case 'LORD':
      return (
        <ScreenWrapper
          title="Modifications vendor coming soon!"
          isPopupVisible={isPopupVisible}
        />
      );
    default:
      return (
        <ScreenWrapper
          title="ECHO initializing..."
          isPopupVisible={isPopupVisible}
        />
      );
  }
};

export default EchoScreen;


