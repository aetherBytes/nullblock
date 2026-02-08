import React, { useRef, useEffect, useState, useCallback } from 'react';
import MFDScreen from './MFDScreen';
import PipBoyScreen from './PipBoyScreen';
// import HecateHologram from './HecateHologram';
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
  agentState: _agentState = 'idle',
  agentName: _agentName = 'hecate',
  currentModel: _currentModel = '',
  agentHealthStatus: _agentHealthStatus = 'unknown',
  sessionMessageCount: _sessionMessageCount = 0,
  isLoggedIn = false,
  loginAnimationPhase = 'idle',
  onConnectWallet,
  onEnterCrossroads,
  pendingCrossroadsTransition = false,
  autopilot: _autopilot = true,
  onToggleAutopilot: _onToggleAutopilot,
}) => {
  const showActions = !isLoggedIn && (loginAnimationPhase === 'navbar' || loginAnimationPhase === 'complete');

  type Point = { x: number; y: number };
  type ConnectorData = {
    w: number; h: number;
    lines: Array<{x1: number; y1: number; x2: number; y2: number}>;
    panels: Array<{
      points: string;
      outerX: number;
      innerX: number;
      midY: number;
    }>;
  };

  const mfdRowRef = useRef<HTMLDivElement>(null);
  const [svgData, setSvgData] = useState<ConnectorData>({ w: 0, h: 0, lines: [], panels: [] });

  const measureConnectors = useCallback(() => {
    const row = mfdRowRef.current;
    if (!row) return;
    const rowRect = row.getBoundingClientRect();

    const corner = (panelCls: string, top: boolean, left: boolean): Point | null => {
      const panel = row.getElementsByClassName(panelCls)[0] as HTMLElement | undefined;
      const bezel = panel?.firstElementChild as HTMLElement | undefined;
      if (!bezel) return null;
      const prev = bezel.style.position;
      if (!prev) bezel.style.position = 'relative';
      const m = document.createElement('div');
      m.style.cssText = `position:absolute;width:0;height:0;pointer-events:none;${top ? 'top:0' : 'bottom:0'};${left ? 'left:0' : 'right:0'}`;
      bezel.appendChild(m);
      const r = m.getBoundingClientRect();
      bezel.removeChild(m);
      if (!prev) bezel.style.position = '';
      return { x: r.left - rowRect.left, y: r.top - rowRect.top };
    };

    const leftTR = corner(styles.mfdLeft, true, false);
    const leftBR = corner(styles.mfdLeft, false, false);
    const centerTL = corner(styles.mfdCenter, true, true);
    const centerTR = corner(styles.mfdCenter, true, false);
    const centerBL = corner(styles.mfdCenter, false, true);
    const centerBR = corner(styles.mfdCenter, false, false);
    const rightTL = corner(styles.mfdRight, true, true);
    const rightBL = corner(styles.mfdRight, false, true);

    const lines: ConnectorData['lines'] = [];
    if (leftTR && centerTL) lines.push({ x1: leftTR.x, y1: leftTR.y, x2: centerTL.x, y2: centerTL.y });
    if (centerTR && rightTL) lines.push({ x1: centerTR.x, y1: centerTR.y, x2: rightTL.x, y2: rightTL.y });
    if (leftBR && centerBL) lines.push({ x1: leftBR.x, y1: leftBR.y, x2: centerBL.x, y2: centerBL.y });
    if (centerBR && rightBL) lines.push({ x1: centerBR.x, y1: centerBR.y, x2: rightBL.x, y2: rightBL.y });

    const panels: ConnectorData['panels'] = [];
    if (leftTR && centerTL && centerBL && leftBR) {
      const midY = (leftTR.y + centerTL.y + centerBL.y + leftBR.y) / 4;
      panels.push({
        points: `${leftTR.x},${leftTR.y} ${centerTL.x},${centerTL.y} ${centerBL.x},${centerBL.y} ${leftBR.x},${leftBR.y}`,
        outerX: Math.min(leftTR.x, leftBR.x),
        innerX: Math.max(centerTL.x, centerBL.x),
        midY,
      });
    }
    if (centerTR && rightTL && rightBL && centerBR) {
      const midY = (centerTR.y + rightTL.y + rightBL.y + centerBR.y) / 4;
      panels.push({
        points: `${centerTR.x},${centerTR.y} ${rightTL.x},${rightTL.y} ${rightBL.x},${rightBL.y} ${centerBR.x},${centerBR.y}`,
        outerX: Math.max(rightTL.x, rightBL.x),
        innerX: Math.min(centerTR.x, centerBR.x),
        midY,
      });
    }

    setSvgData({ w: rowRect.width, h: rowRect.height, lines, panels });
  }, []);

  useEffect(() => {
    if (!visible) return;
    const id = requestAnimationFrame(() => requestAnimationFrame(measureConnectors));
    window.addEventListener('resize', measureConnectors);
    return () => { cancelAnimationFrame(id); window.removeEventListener('resize', measureConnectors); };
  }, [visible, measureConnectors]);

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

      {/* Center reticle — commented out while focusing on bottom 3 MFDs
      <div className={styles.reticle}>
        <div className={styles.ringOuter} />
        <div className={styles.ringInner} />
        <div className={styles.ringCore} />
        <div className={styles.reticleCross} />
      </div>
      */}

      {/* Right panel — commented out while focusing on bottom 3 MFDs
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
      */}

      {/* Hecate left-wall HUD panel — commented out while focusing on bottom 3 MFDs
      <div className={styles.hecateColumn}>
        <HecateHologram
          agentState={agentState}
          agentName={agentName}
          currentModel={currentModel}
          healthStatus={agentHealthStatus}
          sessionMessageCount={sessionMessageCount}
        />
      </div>
      */}

      {/* MFD row — 3 pane cockpit */}
      <div ref={mfdRowRef} className={styles.mfdRow}>
        {(svgData.lines.length > 0 || svgData.panels.length > 0) && (
          <svg
            className={styles.connectorSvg}
            viewBox={`0 0 ${svgData.w} ${svgData.h}`}
          >
            <defs>
              <filter id="connGlow">
                <feGaussianBlur stdDeviation="3" result="b" />
                <feMerge>
                  <feMergeNode in="b" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
              {svgData.panels.map((p, i) => (
                <linearGradient
                  key={`grad-${i}`}
                  id={`panelGrad${i}`}
                  gradientUnits="userSpaceOnUse"
                  x1={p.outerX} y1={p.midY}
                  x2={p.innerX} y2={p.midY}
                >
                  <stop offset="0%" stopColor="rgb(8, 6, 18)" stopOpacity="0.95" />
                  <stop offset="50%" stopColor="rgb(14, 12, 30)" stopOpacity="0.85" />
                  <stop offset="100%" stopColor="rgb(20, 16, 40)" stopOpacity="0.7" />
                </linearGradient>
              ))}
            </defs>
            {svgData.panels.map((p, i) => (
              <polygon
                key={`panel-${i}`}
                points={p.points}
                fill={`url(#panelGrad${i})`}
                stroke="none"
              />
            ))}
            {svgData.lines.map((l, i) => (
              <React.Fragment key={i}>
                <line
                  x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
                  stroke="rgba(154, 123, 255, 0.06)"
                  strokeWidth="6"
                  filter="url(#connGlow)"
                />
                <line
                  x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
                  stroke="rgba(154, 123, 255, 0.3)"
                  strokeWidth="1"
                />
              </React.Fragment>
            ))}
          </svg>
        )}
        <PipBoyScreen className={styles.mfdLeft} isLoggedIn={isLoggedIn}>
          <div className={styles.mfdChatWrap}>
            {chatMFD || (
              <div className={styles.pipboyPlaceholder}>
                <div className={styles.pipboyPrompt}>&gt;_</div>
                <span>HECATE COMMS</span>
                <span className={styles.pipboyPlaceholderSub}>Awaiting connection...</span>
              </div>
            )}
          </div>
        </PipBoyScreen>
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
        <div className={styles.landscapeWarning}>ROTATE DEVICE FOR FULL COCKPIT</div>
        <PipBoyScreen className={styles.mfdRight} isLoggedIn={isLoggedIn} tabs={['STATUS', 'SYS', 'NAV']}>
          <div className={styles.mfdStatusWrap}>
            <div className={styles.statusLinks}>
              <a
                href="https://aetherbytes.github.io/nullblock-sdk/"
                target="_blank"
                rel="noopener noreferrer"
                className={styles.statusLink}
              >
                <div className={styles.statusLinkDot} />
                <span>DOCS</span>
              </a>
              <a
                href="https://x.com/Nullblock_io"
                target="_blank"
                rel="noopener noreferrer"
                className={styles.statusLink}
              >
                <div className={styles.statusLinkDot} />
                <span>FOLLOW</span>
              </a>
            </div>
          </div>
        </PipBoyScreen>
      </div>

      {/* Cockpit floor — beneath MFD panels */}
      <div className={styles.cockpitFloor} />

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
