import React from 'react';
import MFDScreen from './MFDScreen';
import HecateHologram from './HecateHologram';
import styles from './CockpitDash.module.scss';

interface CockpitDashProps {
  visible: boolean;
  chatMFD?: React.ReactNode;
  agentState?: 'idle' | 'thinking' | 'responding';
  agentName?: string;
  currentModel?: string;
  agentHealthStatus?: string;
  sessionMessageCount?: number;
  isLoggedIn?: boolean;
  loginAnimationPhase?: string;
  onConnectWallet?: () => void;
  onEnterCrossroads?: () => void;
  pendingCrossroadsTransition?: boolean;
  autopilot?: boolean;
  onToggleAutopilot?: () => void;
}

const CockpitDash: React.FC<CockpitDashProps> = ({
  visible,
  chatMFD,
  agentState = 'idle',
  agentName = 'hecate',
  currentModel = '',
  agentHealthStatus = 'unknown',
  sessionMessageCount = 0,
  isLoggedIn = false,
  loginAnimationPhase = 'idle',
  onConnectWallet,
  onEnterCrossroads,
  pendingCrossroadsTransition = false,
  autopilot = true,
  onToggleAutopilot,
}) => {
  const showActions = !isLoggedIn && (loginAnimationPhase === 'navbar' || loginAnimationPhase === 'complete');
  return (
    <div className={`${styles.cockpitDash} ${visible ? styles.visible : ''}`}>
      {/* Frame chrome — pointer-events: none overlay */}
      <div className={styles.frameChrome}>
        <div className={styles.frameTop} />
        <div className={styles.frameBottom} />
        <div className={styles.frameLeft} />
        <div className={styles.frameRight} />
        <div className={styles.strutTL} />
        <div className={styles.strutTR} />
        <div className={styles.strutBL} />
        <div className={styles.strutBR} />
        <div className={styles.scanLines} />
        <div className={styles.vignette} />
      </div>

      {/* Center reticle */}
      <div className={styles.reticle}>
        <div className={styles.ringOuter} />
        <div className={styles.ringInner} />
        <div className={styles.ringCore} />
        <div className={styles.reticleCross} />
      </div>

      {/* Left panel — SYS/NAV/COMMS/HULL */}
      <div className={styles.leftPanel}>
        <div className={styles.sidePanelInner}>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>SYS</span>
            <span className={styles.hudValGreen}>NOMINAL</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>NAV</span>
            <span className={styles.hudValGreen}>ONLINE</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>COMMS</span>
            <span className={styles.hudValGreen}>ACTIVE</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>HULL</span>
            <span className={styles.hudValCyan}>100%</span>
          </div>
        </div>
        <div className={styles.ticks}>
          {[...Array(11)].map((_, i) => (
            <div key={i} className={`${styles.tick} ${i === 5 ? styles.tickLong : ''}`} />
          ))}
        </div>
      </div>

      {/* Right panel — SECTOR/RANGE/RATE/DRIFT */}
      <div className={styles.rightPanel}>
        <div className={styles.sidePanelInner}>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>SECTOR</span>
            <span className={styles.hudValGreen}>7G</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>RANGE</span>
            <span className={styles.hudValCyan}>202.6 m</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>RATE</span>
            <span className={styles.hudValCyan}>0.038 m/s</span>
          </div>
          <div className={styles.hudRow}>
            <span className={styles.hudLabel}>DRIFT</span>
            <span className={styles.hudValAmber}>0.2°/s</span>
          </div>
        </div>
        <div className={styles.ticks}>
          {[...Array(11)].map((_, i) => (
            <div key={i} className={`${styles.tick} ${i === 5 ? styles.tickLong : ''}`} />
          ))}
        </div>
      </div>

      {/* Hecate hologram — floating in viewport */}
      <HecateHologram
        agentState={agentState}
        agentName={agentName}
        currentModel={currentModel}
        healthStatus={agentHealthStatus}
        sessionMessageCount={sessionMessageCount}
      />

      {/* MFD row */}
      <div className={styles.mfdRow}>
        <div className={styles.strutLeft} />
        <MFDScreen title="COMMS" statusColor="cyan" className={styles.mfdLeft}>
          <div className={styles.mfdChatWrap}>
            {chatMFD || (
              <div className={styles.mfdPlaceholder}>
                <span>HECATE COMMS</span>
                <span className={styles.mfdPlaceholderSub}>Awaiting connection...</span>
              </div>
            )}
          </div>
        </MFDScreen>
        <MFDScreen
          title={showActions ? 'COMMAND' : 'INSTRUMENTS'}
          statusColor={showActions ? 'cyan' : 'green'}
          className={styles.mfdCenter}
        >
          {showActions ? (
            <div className={styles.actionPanel}>
              <button
                className={styles.cockpitAction}
                onClick={onConnectWallet}
                type="button"
              >
                <div className={styles.actionIndicator}>
                  <div className={styles.actionDot} />
                </div>
                <div className={styles.actionContent}>
                  <span className={styles.actionLabel}>CONNECT</span>
                  <span className={styles.actionSub}>FULL INTERFACE</span>
                </div>
                <div className={styles.actionArrow}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </div>
              </button>
              <div className={styles.actionDivider} />
              <button
                className={`${styles.cockpitAction} ${styles.cockpitActionAlt} ${pendingCrossroadsTransition ? styles.actionTransitioning : ''}`}
                onClick={onEnterCrossroads}
                disabled={pendingCrossroadsTransition}
                type="button"
              >
                <div className={styles.actionIndicator}>
                  <div className={`${styles.actionDot} ${styles.actionDotPurple}`} />
                </div>
                <div className={styles.actionContent}>
                  <span className={styles.actionLabel}>
                    {pendingCrossroadsTransition ? 'ALIGNING' : 'ENTER'}
                  </span>
                  <span className={styles.actionSub}>CROSSROADS</span>
                </div>
                <div className={styles.actionArrow}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </div>
              </button>
            </div>
          ) : (
            <>
              <div className={styles.gaugeGrid}>
                <div className={styles.gaugeItem}>
                  <div className={styles.gaugeBar}>
                    <div className={`${styles.gaugeFill} ${styles.fillHigh}`} />
                  </div>
                  <span className={styles.gaugeLabel}>PWR</span>
                </div>
                <div className={styles.gaugeItem}>
                  <div className={styles.gaugeBar}>
                    <div className={`${styles.gaugeFill} ${styles.fillMid}`} />
                  </div>
                  <span className={styles.gaugeLabel}>FUL</span>
                </div>
                <div className={styles.gaugeItem}>
                  <div className={styles.gaugeBar}>
                    <div className={`${styles.gaugeFill} ${styles.fillHigh}`} />
                  </div>
                  <span className={styles.gaugeLabel}>O2</span>
                </div>
                <div className={styles.gaugeItem}>
                  <div className={styles.gaugeBar}>
                    <div className={`${styles.gaugeFill} ${styles.fillLow}`} />
                  </div>
                  <span className={styles.gaugeLabel}>THR</span>
                </div>
              </div>
              <div className={styles.compassRow}>
                <span className={styles.compassLabel}>HDG</span>
                <div className={styles.compassTrack}>
                  <div className={styles.compassNeedle} />
                </div>
                <span className={styles.compassVal}>042°</span>
              </div>
            </>
          )}
        </MFDScreen>
        <MFDScreen title="STATUS" statusColor="green" className={styles.mfdRight}>
          <div className={styles.statusList}>
            <div className={styles.statusRow}>
              <div className={`${styles.sDot} ${styles.sGreen}`} />
              <span>ENGN.01</span>
            </div>
            <div className={styles.statusRow}>
              <div className={`${styles.sDot} ${styles.sGreen}`} />
              <span>ENGN.02</span>
            </div>
            <div className={styles.statusRow}>
              <div className={`${styles.sDot} ${styles.sAmber}`} />
              <span>AUX.PWR</span>
            </div>
            <div className={styles.statusRow}>
              <div className={`${styles.sDot} ${styles.sGreen}`} />
              <span>LIFE.SUP</span>
            </div>
            <div className={styles.statusDivider} />
            <button
              className={`${styles.autopilotToggle} ${autopilot ? styles.autopilotOn : styles.autopilotOff}`}
              onClick={onToggleAutopilot}
              type="button"
            >
              <div className={`${styles.sDot} ${autopilot ? styles.sGreen : styles.sAmber}`} />
              <span className={styles.autopilotLabel}>AUTO.PLT</span>
              <span className={styles.autopilotState}>
                {autopilot ? 'ENGAGED' : 'MANUAL'}
              </span>
            </button>
          </div>
        </MFDScreen>
        <div className={styles.strutRight} />
      </div>

      {/* Footer strip */}
      <div className={styles.footer}>
        <span className={styles.designation}>NBS SEEDER // REG NB-7741</span>
        <div className={styles.footerGauges}>
          <div className={styles.footerGauge}>
            <span className={styles.fgLabel}>PWR</span>
            <div className={styles.fgTrack}><div className={`${styles.fgFill} ${styles.fgHigh}`} /></div>
          </div>
          <div className={styles.footerGauge}>
            <span className={styles.fgLabel}>FUL</span>
            <div className={styles.fgTrack}><div className={`${styles.fgFill} ${styles.fgMid}`} /></div>
          </div>
          <div className={styles.footerGauge}>
            <span className={styles.fgLabel}>O2</span>
            <div className={styles.fgTrack}><div className={`${styles.fgFill} ${styles.fgHigh}`} /></div>
          </div>
          <div className={styles.footerGauge}>
            <span className={styles.fgLabel}>THR</span>
            <div className={styles.fgTrack}><div className={`${styles.fgFill} ${styles.fgLow}`} /></div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CockpitDash;
