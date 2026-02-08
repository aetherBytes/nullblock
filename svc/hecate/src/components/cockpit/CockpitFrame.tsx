import React from 'react';
import styles from './CockpitFrame.module.scss';

interface CockpitFrameProps {
  visible: boolean;
}

const CockpitFrame: React.FC<CockpitFrameProps> = ({ visible }) => {
  return (
    <div className={`${styles.cockpitFrame} ${visible ? styles.visible : ''}`}>
      <div className={styles.frameTop} />
      <div className={styles.frameBottom} />
      <div className={styles.frameLeft} />
      <div className={styles.frameRight} />

      <div className={styles.strutTL} />
      <div className={styles.strutTR} />
      <div className={styles.strutBL} />
      <div className={styles.strutBR} />

      <div className={styles.reticle}>
        <div className={styles.ringOuter} />
        <div className={styles.ringInner} />
        <div className={styles.ringCore} />
        <div className={styles.reticleCross} />
      </div>

      <div className={styles.hudTopLeft}>
        <div className={styles.hudGroup}>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>SYS</span>
            <span className={styles.hudValueGreen}>NOMINAL</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>NAV</span>
            <span className={styles.hudValueGreen}>ONLINE</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>COMMS</span>
            <span className={styles.hudValueGreen}>ACTIVE</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>HULL</span>
            <span className={styles.hudValueCyan}>100%</span>
          </div>
        </div>
      </div>

      <div className={styles.hudTopRight}>
        <div className={styles.hudGroup}>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>SECTOR</span>
            <span className={styles.hudValueGreen}>7G</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>RANGE</span>
            <span className={styles.hudValueCyan}>202.6 m</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>RATE</span>
            <span className={styles.hudValueCyan}>0.038 m/s</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>DRIFT</span>
            <span className={styles.hudValueAmber}>0.2°/s</span>
          </div>
        </div>
      </div>

      <div className={styles.hudBottomLeft}>
        <div className={styles.gaugeRow}>
          <div className={styles.gauge}>
            <div className={`${styles.gaugeFill} ${styles.fillHigh}`} />
            <span className={styles.gaugeLabel}>PWR</span>
          </div>
          <div className={styles.gauge}>
            <div className={`${styles.gaugeFill} ${styles.fillMid}`} />
            <span className={styles.gaugeLabel}>FUL</span>
          </div>
          <div className={styles.gauge}>
            <div className={`${styles.gaugeFill} ${styles.fillHigh}`} />
            <span className={styles.gaugeLabel}>O2</span>
          </div>
          <div className={styles.gauge}>
            <div className={`${styles.gaugeFill} ${styles.fillLow}`} />
            <span className={styles.gaugeLabel}>THR</span>
          </div>
        </div>
      </div>

      <div className={styles.hudBottomRight}>
        <div className={styles.hudRow}>
          <div className={`${styles.statusDot} ${styles.dotGreen}`} />
          <span className={styles.hudLabelSm}>ENGN.01</span>
        </div>
        <div className={styles.hudRow}>
          <div className={`${styles.statusDot} ${styles.dotGreen}`} />
          <span className={styles.hudLabelSm}>ENGN.02</span>
        </div>
        <div className={styles.hudRow}>
          <div className={`${styles.statusDot} ${styles.dotAmber}`} />
          <span className={styles.hudLabelSm}>AUX.PWR</span>
        </div>
        <div className={styles.hudRow}>
          <div className={`${styles.statusDot} ${styles.dotGreen}`} />
          <span className={styles.hudLabelSm}>LIFE.SUP</span>
        </div>
      </div>

      <div className={styles.hudBottomCenter}>
        <span className={styles.designation}>NBS SEEDER // REG NB-7741</span>
      </div>

      <div className={styles.ticksLeft}>
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={`${styles.tick} ${styles.tickLong}`} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
      </div>
      <div className={styles.ticksRight}>
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={`${styles.tick} ${styles.tickLong}`} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
        <div className={styles.tick} />
      </div>

      <div className={styles.scanLines} />
      <div className={styles.vignette} />
    </div>
  );
};

export default CockpitFrame;
