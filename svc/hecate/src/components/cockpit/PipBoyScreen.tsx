import React, { useEffect, useRef, useState } from 'react';
import styles from './PipBoyScreen.module.scss';

interface AlertItem {
  id: string;
  text: string;
  priority?: 'info' | 'warn' | 'critical';
}

interface PipBoyScreenProps {
  children: React.ReactNode;
  className?: string;
  activeTab?: string;
  tabs?: string[];
  onTabChange?: (tab: string) => void;
  alerts?: AlertItem[];
  isLoggedIn?: boolean;
}

const DEFAULT_ALERTS: AlertItem[] = [
  { id: 'welcome', text: 'HECATE COMMS ARRAY \u2014 Signal locked \u2014 Awaiting transmission', priority: 'info' },
  { id: 'status', text: 'ALL SYSTEMS NOMINAL \u2014 Standing by for operator input', priority: 'info' },
];

const PipBoyScreen: React.FC<PipBoyScreenProps> = ({
  children,
  className,
  activeTab = 'COMMS',
  tabs = ['COMMS', 'DATA', 'LOG'],
  onTabChange,
  alerts,
  isLoggedIn: _isLoggedIn = false,
}) => {
  const displayAlerts = alerts || DEFAULT_ALERTS;
  const [currentAlert, setCurrentAlert] = useState(0);
  const tickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (displayAlerts.length <= 1) return;
    const interval = setInterval(() => {
      setCurrentAlert((prev) => (prev + 1) % displayAlerts.length);
    }, 6000);
    return () => clearInterval(interval);
  }, [displayAlerts.length]);

  return (
    <div className={`${styles.pipboy} ${className || ''}`}>
      <div className={styles.bezel}>
        <div className={styles.powerLed} />
        <div className={styles.bezelTop}>
          <div className={styles.tabs}>
            {tabs.map((tab) => (
              <button
                key={tab}
                className={`${styles.tab} ${tab === activeTab ? styles.tabActive : ''}`}
                onClick={() => onTabChange?.(tab)}
                type="button"
              >
                {tab}
              </button>
            ))}
          </div>
          <div className={styles.bezelLabel}>RobCo</div>
        </div>

        <div className={styles.crtFrame}>
          <div className={styles.crtScreen}>
            <div className={styles.screenContent}>
              {children}
            </div>
            <div className={styles.scanLines} />
            <div className={styles.screenGlow} />
            <div className={styles.flicker} />
          </div>
        </div>

        <div className={styles.alertStrip}>
          <div className={styles.alertIndicator}>
            <div className={`${styles.alertDot} ${styles[displayAlerts[currentAlert]?.priority || 'info']}`} />
          </div>
          <div className={styles.alertTicker} ref={tickerRef}>
            <div
              className={styles.alertText}
              key={currentAlert}
            >
              {displayAlerts[currentAlert]?.text}
            </div>
          </div>
        </div>

        <div className={styles.bezelBottom}>
          <div className={styles.knob} />
          <div className={styles.screws}>
            <div className={styles.screw} />
            <div className={styles.screw} />
            <div className={styles.screw} />
          </div>
          <div className={styles.knob} />
        </div>
      </div>
    </div>
  );
};

export default PipBoyScreen;
