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
  type PanelFill = {
    points: string;
    outerX: number;
    innerX: number;
    midY: number;
  };
  type BackingFill = {
    points: string;
    gx1: number; gy1: number;
    gx2: number; gy2: number;
  };
  type WindshieldFill = {
    points: string;
    midX: number;
    topY: number;
    bottomY: number;
    topW: number;
    bottomW: number;
  };
  type ConnectorData = {
    w: number; h: number;
    lines: Array<{x1: number; y1: number; x2: number; y2: number}>;
    backLines: Array<{x1: number; y1: number; x2: number; y2: number}>;
    panels: PanelFill[];
    backings: BackingFill[];
    windshields: WindshieldFill[];
  };

  const mfdRowRef = useRef<HTMLDivElement>(null);
  const [svgData, setSvgData] = useState<ConnectorData>({ w: 0, h: 0, lines: [], backLines: [], panels: [], backings: [], windshields: [] });

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

    const leftTL = corner(styles.mfdLeft, true, true);
    const leftTR = corner(styles.mfdLeft, true, false);
    const leftBL = corner(styles.mfdLeft, false, true);
    const leftBR = corner(styles.mfdLeft, false, false);
    const centerTL = corner(styles.mfdCenter, true, true);
    const centerTR = corner(styles.mfdCenter, true, false);
    const centerBL = corner(styles.mfdCenter, false, true);
    const centerBR = corner(styles.mfdCenter, false, false);
    const rightTL = corner(styles.mfdRight, true, true);
    const rightTR = corner(styles.mfdRight, true, false);
    const rightBL = corner(styles.mfdRight, false, true);
    const rightBR = corner(styles.mfdRight, false, false);

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

    const backLines: ConnectorData['backLines'] = [];
    const backings: ConnectorData['backings'] = [];
    if (leftTL && leftBL) {
      const bTL = { x: leftTL.x - 22, y: leftTL.y + 18 };
      const bBL = { x: leftBL.x - 14, y: leftBL.y + 6 };
      const midY = (leftTL.y + leftBL.y + bTL.y + bBL.y) / 4;
      backings.push({
        points: `${leftTL.x},${leftTL.y} ${bTL.x},${bTL.y} ${bBL.x},${bBL.y} ${leftBL.x},${leftBL.y}`,
        gx1: Math.max(leftTL.x, leftBL.x), gy1: midY,
        gx2: Math.min(bTL.x, bBL.x), gy2: midY,
      });
      backLines.push({ x1: leftTL.x, y1: leftTL.y, x2: bTL.x, y2: bTL.y });
      backLines.push({ x1: leftBL.x, y1: leftBL.y, x2: bBL.x, y2: bBL.y });
      backLines.push({ x1: bTL.x, y1: bTL.y, x2: bBL.x, y2: bBL.y });
    }
    if (rightTR && rightBR) {
      const bTR = { x: rightTR.x + 22, y: rightTR.y + 18 };
      const bBR = { x: rightBR.x + 14, y: rightBR.y + 6 };
      const midY = (rightTR.y + rightBR.y + bTR.y + bBR.y) / 4;
      backings.push({
        points: `${rightTR.x},${rightTR.y} ${bTR.x},${bTR.y} ${bBR.x},${bBR.y} ${rightBR.x},${rightBR.y}`,
        gx1: Math.min(rightTR.x, rightBR.x), gy1: midY,
        gx2: Math.max(bTR.x, bBR.x), gy2: midY,
      });
      backLines.push({ x1: rightTR.x, y1: rightTR.y, x2: bTR.x, y2: bTR.y });
      backLines.push({ x1: rightBR.x, y1: rightBR.y, x2: bBR.x, y2: bBR.y });
      backLines.push({ x1: bTR.x, y1: bTR.y, x2: bBR.x, y2: bBR.y });
    }
    if (centerBL && centerBR) {
      const bCBL = { x: centerBL.x + 6, y: centerBL.y + 14 };
      const bCBR = { x: centerBR.x - 6, y: centerBR.y + 14 };
      const midX = (centerBL.x + centerBR.x) / 2;
      backings.push({
        points: `${centerBL.x},${centerBL.y} ${centerBR.x},${centerBR.y} ${bCBR.x},${bCBR.y} ${bCBL.x},${bCBL.y}`,
        gx1: midX, gy1: centerBL.y,
        gx2: midX, gy2: bCBL.y,
      });
      backLines.push({ x1: centerBL.x, y1: centerBL.y, x2: bCBL.x, y2: bCBL.y });
      backLines.push({ x1: centerBR.x, y1: centerBR.y, x2: bCBR.x, y2: bCBR.y });
      backLines.push({ x1: bCBL.x, y1: bCBL.y, x2: bCBR.x, y2: bCBR.y });
    }

    const windshields: ConnectorData['windshields'] = [];
    const wsHeight = 390;
    const wsTopRatio = 0.25;
    let leftWsInnerTop: Point | null = null;
    let rightWsInnerTop: Point | null = null;

    if (leftTR && centerTL) {
      const midX = (leftTR.x + centerTL.x) / 2;
      const midBottomY = (leftTR.y + centerTL.y) / 2;
      const bottomW = centerTL.x - leftTR.x;
      const topW = bottomW * wsTopRatio;
      const topY = midBottomY - wsHeight;
      const tl = { x: midX - topW / 2, y: topY };
      const tr = { x: midX + topW / 2, y: topY };
      leftWsInnerTop = tr;
      windshields.push({
        points: `${leftTR.x},${leftTR.y} ${centerTL.x},${centerTL.y} ${tr.x},${tr.y} ${tl.x},${tl.y}`,
        midX, topY, bottomY: midBottomY, topW: topW, bottomW,
      });
      backLines.push({ x1: leftTR.x, y1: leftTR.y, x2: tl.x, y2: tl.y });
      backLines.push({ x1: centerTL.x, y1: centerTL.y, x2: tr.x, y2: tr.y });
      backLines.push({ x1: tl.x, y1: tl.y, x2: tr.x, y2: tr.y });
    }
    if (centerTR && rightTL) {
      const midX = (centerTR.x + rightTL.x) / 2;
      const midBottomY = (centerTR.y + rightTL.y) / 2;
      const bottomW = rightTL.x - centerTR.x;
      const topW = bottomW * wsTopRatio;
      const topY = midBottomY - wsHeight;
      const tl = { x: midX - topW / 2, y: topY };
      const tr = { x: midX + topW / 2, y: topY };
      rightWsInnerTop = tl;
      windshields.push({
        points: `${centerTR.x},${centerTR.y} ${rightTL.x},${rightTL.y} ${tr.x},${tr.y} ${tl.x},${tl.y}`,
        midX, topY, bottomY: midBottomY, topW: topW, bottomW,
      });
      backLines.push({ x1: centerTR.x, y1: centerTR.y, x2: tl.x, y2: tl.y });
      backLines.push({ x1: rightTL.x, y1: rightTL.y, x2: tr.x, y2: tr.y });
      backLines.push({ x1: tl.x, y1: tl.y, x2: tr.x, y2: tr.y });
    }
    if (centerTL && centerTR && leftWsInnerTop && rightWsInnerTop) {
      const midX = (leftWsInnerTop.x + rightWsInnerTop.x) / 2;
      const topY = leftWsInnerTop.y;
      const bottomY = (centerTL.y + centerTR.y) / 2;
      const cTopW = rightWsInnerTop.x - leftWsInnerTop.x;
      const cBottomW = centerTR.x - centerTL.x;
      windshields.push({
        points: `${centerTL.x},${centerTL.y} ${centerTR.x},${centerTR.y} ${rightWsInnerTop.x},${rightWsInnerTop.y} ${leftWsInnerTop.x},${leftWsInnerTop.y}`,
        midX, topY, bottomY, topW: cTopW, bottomW: cBottomW,
      });
      backLines.push({ x1: leftWsInnerTop.x, y1: leftWsInnerTop.y, x2: rightWsInnerTop.x, y2: rightWsInnerTop.y });
    }

    setSvgData({ w: rowRect.width, h: rowRect.height, lines, backLines, panels, backings, windshields });
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
        {(svgData.lines.length > 0 || svgData.panels.length > 0 || svgData.backings.length > 0) && (
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
              {svgData.backings.map((b, i) => (
                <linearGradient
                  key={`bgrad-${i}`}
                  id={`backingGrad${i}`}
                  gradientUnits="userSpaceOnUse"
                  x1={b.gx1} y1={b.gy1}
                  x2={b.gx2} y2={b.gy2}
                >
                  <stop offset="0%" stopColor="rgb(12, 10, 24)" stopOpacity="0.85" />
                  <stop offset="60%" stopColor="rgb(6, 4, 14)" stopOpacity="0.92" />
                  <stop offset="100%" stopColor="rgb(2, 2, 6)" stopOpacity="0.95" />
                </linearGradient>
              ))}
              {svgData.windshields.map((ws, i) => {
                const cy = (ws.topY + ws.bottomY) / 2;
                const avgHalfW = (ws.topW + ws.bottomW) / 4;
                const halfH = (ws.bottomY - ws.topY) / 2;
                const rx = avgHalfW * 0.45;
                const ry = halfH * 0.4;
                return (
                  <radialGradient
                    key={`wsgrad-${i}`}
                    id={`wsGrad${i}`}
                    gradientUnits="userSpaceOnUse"
                    cx={ws.midX} cy={cy}
                    rx={rx} ry={ry}
                  >
                    <stop offset="0%" stopColor="rgb(200, 200, 220)" stopOpacity="0" />
                    <stop offset="30%" stopColor="rgb(180, 180, 210)" stopOpacity="0.04" />
                    <stop offset="55%" stopColor="rgb(160, 160, 200)" stopOpacity="0.12" />
                    <stop offset="72%" stopColor="rgb(140, 140, 180)" stopOpacity="0.22" />
                    <stop offset="85%" stopColor="rgb(120, 120, 170)" stopOpacity="0.35" />
                    <stop offset="100%" stopColor="rgb(100, 100, 160)" stopOpacity="0.5" />
                  </radialGradient>
                );
              })}
            </defs>
            {svgData.windshields.map((ws, i) => (
              <polygon
                key={`ws-${i}`}
                points={ws.points}
                fill={`url(#wsGrad${i})`}
                stroke="none"
              />
            ))}
            {svgData.backings.map((b, i) => (
              <polygon
                key={`backing-${i}`}
                points={b.points}
                fill={`url(#backingGrad${i})`}
                stroke="none"
              />
            ))}
            {svgData.panels.map((p, i) => (
              <polygon
                key={`panel-${i}`}
                points={p.points}
                fill={`url(#panelGrad${i})`}
                stroke="none"
              />
            ))}
            {svgData.backLines.map((l, i) => (
              <line
                key={`bl-${i}`}
                x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
                stroke="rgba(154, 123, 255, 0.15)"
                strokeWidth="0.6"
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
