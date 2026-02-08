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
  activeTab?: 'crossroads' | 'memcache' | null;
}

const PAGE_INFO: Record<string, { title: string; subtitle: string }> = {
  crossroads: {
    title: 'Picks and shovels for the new age.',
    subtitle: 'Agents are the new users. Own the tools that own the future.',
  },
  memcache: {
    title: 'MemCache',
    subtitle: 'Memory, context, and operational intelligence.',
  },
  void: {
    title: 'Picks and shovels for the new age.',
    subtitle: 'Agents are the new users. Own the tools that own the future.',
  },
};

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
  activeTab,
}) => {
  const showActions = !isLoggedIn && (loginAnimationPhase === 'navbar' || loginAnimationPhase === 'complete');
  const pageInfo = PAGE_INFO[activeTab || 'void'] || PAGE_INFO.void;

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
  const mfdRowTopRef = useRef<HTMLDivElement>(null);
  const [svgData, setSvgData] = useState<ConnectorData>({ w: 0, h: 0, lines: [], backLines: [], panels: [], backings: [], windshields: [] });
  const [topSvgData, setTopSvgData] = useState<{ w: number; h: number; lines: ConnectorData['lines']; backLines: ConnectorData['backLines']; panels: PanelFill[]; backings: BackingFill[] }>({ w: 0, h: 0, lines: [], backLines: [], panels: [], backings: [] });

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
      const bTL = { x: leftTL.x - 50, y: leftTL.y + 40 };
      const bBL = { x: leftBL.x - 30, y: leftBL.y + 14 };
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
      const bTR = { x: rightTR.x + 50, y: rightTR.y + 40 };
      const bBR = { x: rightBR.x + 30, y: rightBR.y + 14 };
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
      const bCBL = { x: centerBL.x + 14, y: centerBL.y + 35 };
      const bCBR = { x: centerBR.x - 14, y: centerBR.y + 35 };
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
    const topRow = mfdRowTopRef.current;
    let topLeftBR: Point | null = null;
    let topCenterBL: Point | null = null;
    let topCenterBR: Point | null = null;
    let topRightBL: Point | null = null;

    if (topRow) {
      const tc = (panelCls: string, left: boolean): Point | null => {
        const panel = topRow.getElementsByClassName(panelCls)[0] as HTMLElement | undefined;
        const bezel = panel?.firstElementChild as HTMLElement | undefined;
        if (!bezel) return null;
        const prev = bezel.style.position;
        if (!prev) bezel.style.position = 'relative';
        const m = document.createElement('div');
        m.style.cssText = `position:absolute;width:0;height:0;pointer-events:none;bottom:0;${left ? 'left:0' : 'right:0'}`;
        bezel.appendChild(m);
        const r = m.getBoundingClientRect();
        bezel.removeChild(m);
        if (!prev) bezel.style.position = '';
        return { x: r.left - rowRect.left, y: r.top - rowRect.top };
      };
      topLeftBR = tc(styles.mfdTopLeft, false);
      topCenterBL = tc(styles.mfdTopCenter, true);
      topCenterBR = tc(styles.mfdTopCenter, false);
      topRightBL = tc(styles.mfdTopRight, true);
    }

    if (leftTR && centerTL && topLeftBR && topCenterBL) {
      const midBottomY = (leftTR.y + centerTL.y) / 2;
      const bottomW = centerTL.x - leftTR.x;
      const midTopY = (topLeftBR.y + topCenterBL.y) / 2;
      const topW = Math.abs(topCenterBL.x - topLeftBR.x);
      const midX = (leftTR.x + centerTL.x + topLeftBR.x + topCenterBL.x) / 4;
      windshields.push({
        points: `${leftTR.x},${leftTR.y} ${centerTL.x},${centerTL.y} ${topCenterBL.x},${topCenterBL.y} ${topLeftBR.x},${topLeftBR.y}`,
        midX, topY: midTopY, bottomY: midBottomY, topW, bottomW,
      });
      backLines.push({ x1: leftTR.x, y1: leftTR.y, x2: topLeftBR.x, y2: topLeftBR.y });
      backLines.push({ x1: centerTL.x, y1: centerTL.y, x2: topCenterBL.x, y2: topCenterBL.y });
    }
    if (centerTR && rightTL && topCenterBR && topRightBL) {
      const midBottomY = (centerTR.y + rightTL.y) / 2;
      const bottomW = rightTL.x - centerTR.x;
      const midTopY = (topCenterBR.y + topRightBL.y) / 2;
      const topW = Math.abs(topRightBL.x - topCenterBR.x);
      const midX = (centerTR.x + rightTL.x + topCenterBR.x + topRightBL.x) / 4;
      windshields.push({
        points: `${centerTR.x},${centerTR.y} ${rightTL.x},${rightTL.y} ${topRightBL.x},${topRightBL.y} ${topCenterBR.x},${topCenterBR.y}`,
        midX, topY: midTopY, bottomY: midBottomY, topW, bottomW,
      });
      backLines.push({ x1: centerTR.x, y1: centerTR.y, x2: topCenterBR.x, y2: topCenterBR.y });
      backLines.push({ x1: rightTL.x, y1: rightTL.y, x2: topRightBL.x, y2: topRightBL.y });
    }
    if (centerTL && centerTR && topCenterBL && topCenterBR) {
      const midBottomY = (centerTL.y + centerTR.y) / 2;
      const bottomW = centerTR.x - centerTL.x;
      const midTopY = (topCenterBL.y + topCenterBR.y) / 2;
      const topW = topCenterBR.x - topCenterBL.x;
      const midX = (centerTL.x + centerTR.x + topCenterBL.x + topCenterBR.x) / 4;
      windshields.push({
        points: `${centerTL.x},${centerTL.y} ${centerTR.x},${centerTR.y} ${topCenterBR.x},${topCenterBR.y} ${topCenterBL.x},${topCenterBL.y}`,
        midX, topY: midTopY, bottomY: midBottomY, topW, bottomW,
      });
    }

    setSvgData({ w: rowRect.width, h: rowRect.height, lines, backLines, panels, backings, windshields });
  }, []);

  const updateVisorSkew = useCallback(() => {
    const row = mfdRowTopRef.current;
    if (!row) return;
    const vw = window.innerWidth;
    const t = Math.max(0, Math.min(1, (vw - 768) / (1920 - 768)));
    const skew = Math.round(t * 22 * 10) / 10;
    const margin = Math.round(-6 - t * 18);
    row.style.setProperty('--visor-skew', `${skew}deg`);
    row.style.setProperty('--visor-margin', `${margin}px`);
    row.offsetHeight; // force reflow
  }, []);

  const measureTopConnectors = useCallback(() => {
    const row = mfdRowTopRef.current;
    if (!row) return;
    const rowRect = row.getBoundingClientRect();

    const c = (panelCls: string, top: boolean, left: boolean): Point | null => {
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

    const lTL = c(styles.mfdTopLeft, true, true);
    const lTR = c(styles.mfdTopLeft, true, false);
    const lBL = c(styles.mfdTopLeft, false, true);
    const lBR = c(styles.mfdTopLeft, false, false);
    const cTL = c(styles.mfdTopCenter, true, true);
    const cTR = c(styles.mfdTopCenter, true, false);
    const cBL = c(styles.mfdTopCenter, false, true);
    const cBR = c(styles.mfdTopCenter, false, false);
    const rTL = c(styles.mfdTopRight, true, true);
    const rTR = c(styles.mfdTopRight, true, false);
    const rBL = c(styles.mfdTopRight, false, true);
    const rBR = c(styles.mfdTopRight, false, false);

    const tLines: ConnectorData['lines'] = [];
    if (lTR && cTL) tLines.push({ x1: lTR.x, y1: lTR.y, x2: cTL.x, y2: cTL.y });
    if (cTR && rTL) tLines.push({ x1: cTR.x, y1: cTR.y, x2: rTL.x, y2: rTL.y });
    if (lBR && cBL) tLines.push({ x1: lBR.x, y1: lBR.y, x2: cBL.x, y2: cBL.y });
    if (cBR && rBL) tLines.push({ x1: cBR.x, y1: cBR.y, x2: rBL.x, y2: rBL.y });

    const tPanels: PanelFill[] = [];
    if (lTR && cTL && cBL && lBR) {
      const midY = (lTR.y + cTL.y + cBL.y + lBR.y) / 4;
      tPanels.push({
        points: `${lTR.x},${lTR.y} ${cTL.x},${cTL.y} ${cBL.x},${cBL.y} ${lBR.x},${lBR.y}`,
        outerX: Math.min(lTR.x, lBR.x), innerX: Math.max(cTL.x, cBL.x), midY,
      });
    }
    if (cTR && rTL && rBL && cBR) {
      const midY = (cTR.y + rTL.y + rBL.y + cBR.y) / 4;
      tPanels.push({
        points: `${cTR.x},${cTR.y} ${rTL.x},${rTL.y} ${rBL.x},${rBL.y} ${cBR.x},${cBR.y}`,
        outerX: Math.max(rTL.x, rBL.x), innerX: Math.min(cTR.x, cBR.x), midY,
      });
    }

    const tBackLines: ConnectorData['backLines'] = [];
    const tBackings: BackingFill[] = [];
    if (lTL && lBL) {
      const bTL = { x: lTL.x - 22, y: lTL.y - 18 };
      const bBL = { x: lBL.x - 14, y: lBL.y - 6 };
      const midY = (lTL.y + lBL.y + bTL.y + bBL.y) / 4;
      tBackings.push({
        points: `${lTL.x},${lTL.y} ${bTL.x},${bTL.y} ${bBL.x},${bBL.y} ${lBL.x},${lBL.y}`,
        gx1: Math.max(lTL.x, lBL.x), gy1: midY,
        gx2: Math.min(bTL.x, bBL.x), gy2: midY,
      });
      tBackLines.push({ x1: lTL.x, y1: lTL.y, x2: bTL.x, y2: bTL.y });
      tBackLines.push({ x1: lBL.x, y1: lBL.y, x2: bBL.x, y2: bBL.y });
      tBackLines.push({ x1: bTL.x, y1: bTL.y, x2: bBL.x, y2: bBL.y });
    }
    if (rTR && rBR) {
      const bTR = { x: rTR.x + 22, y: rTR.y - 18 };
      const bBR = { x: rBR.x + 14, y: rBR.y - 6 };
      const midY = (rTR.y + rBR.y + bTR.y + bBR.y) / 4;
      tBackings.push({
        points: `${rTR.x},${rTR.y} ${bTR.x},${bTR.y} ${bBR.x},${bBR.y} ${rBR.x},${rBR.y}`,
        gx1: Math.min(rTR.x, rBR.x), gy1: midY,
        gx2: Math.max(bTR.x, bBR.x), gy2: midY,
      });
      tBackLines.push({ x1: rTR.x, y1: rTR.y, x2: bTR.x, y2: bTR.y });
      tBackLines.push({ x1: rBR.x, y1: rBR.y, x2: bBR.x, y2: bBR.y });
      tBackLines.push({ x1: bTR.x, y1: bTR.y, x2: bBR.x, y2: bBR.y });
    }
    if (cTL && cTR) {
      const bCTL = { x: cTL.x + 6, y: cTL.y - 14 };
      const bCTR = { x: cTR.x - 6, y: cTR.y - 14 };
      const midX = (cTL.x + cTR.x) / 2;
      tBackings.push({
        points: `${cTL.x},${cTL.y} ${cTR.x},${cTR.y} ${bCTR.x},${bCTR.y} ${bCTL.x},${bCTL.y}`,
        gx1: midX, gy1: cTL.y,
        gx2: midX, gy2: bCTL.y,
      });
      tBackLines.push({ x1: cTL.x, y1: cTL.y, x2: bCTL.x, y2: bCTL.y });
      tBackLines.push({ x1: cTR.x, y1: cTR.y, x2: bCTR.x, y2: bCTR.y });
      tBackLines.push({ x1: bCTL.x, y1: bCTL.y, x2: bCTR.x, y2: bCTR.y });
    }

    setTopSvgData({ w: rowRect.width, h: rowRect.height, lines: tLines, backLines: tBackLines, panels: tPanels, backings: tBackings });
  }, []);

  useEffect(() => {
    if (!visible) return;
    const measureAll = () => { updateVisorSkew(); measureConnectors(); measureTopConnectors(); };
    const id = requestAnimationFrame(() => requestAnimationFrame(measureAll));
    window.addEventListener('resize', measureAll);
    return () => { cancelAnimationFrame(id); window.removeEventListener('resize', measureAll); };
  }, [visible, updateVisorSkew, measureConnectors, measureTopConnectors]);

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

      {/* Top MFD row — overhead panels (inverse of bottom, thinner) */}
      <div ref={mfdRowTopRef} className={styles.mfdRowTop}>
        {(topSvgData.lines.length > 0 || topSvgData.panels.length > 0 || topSvgData.backings.length > 0) && (
          <svg
            className={styles.connectorSvg}
            viewBox={`0 0 ${topSvgData.w} ${topSvgData.h}`}
          >
            <defs>
              <filter id="topConnGlow">
                <feGaussianBlur stdDeviation="3" result="b" />
                <feMerge>
                  <feMergeNode in="b" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
              {topSvgData.panels.map((p, i) => (
                <linearGradient
                  key={`tgrad-${i}`}
                  id={`topPanelGrad${i}`}
                  gradientUnits="userSpaceOnUse"
                  x1={p.outerX} y1={p.midY}
                  x2={p.innerX} y2={p.midY}
                >
                  <stop offset="0%" stopColor="rgb(8, 6, 18)" stopOpacity="0.95" />
                  <stop offset="50%" stopColor="rgb(14, 12, 30)" stopOpacity="0.85" />
                  <stop offset="100%" stopColor="rgb(20, 16, 40)" stopOpacity="0.7" />
                </linearGradient>
              ))}
              {topSvgData.backings.map((b, i) => (
                <linearGradient
                  key={`tbgrad-${i}`}
                  id={`topBackingGrad${i}`}
                  gradientUnits="userSpaceOnUse"
                  x1={b.gx1} y1={b.gy1}
                  x2={b.gx2} y2={b.gy2}
                >
                  <stop offset="0%" stopColor="rgb(12, 10, 24)" stopOpacity="0.85" />
                  <stop offset="60%" stopColor="rgb(6, 4, 14)" stopOpacity="0.92" />
                  <stop offset="100%" stopColor="rgb(2, 2, 6)" stopOpacity="0.95" />
                </linearGradient>
              ))}
            </defs>
            {topSvgData.backings.map((b, i) => (
              <polygon
                key={`tbacking-${i}`}
                points={b.points}
                fill={`url(#topBackingGrad${i})`}
                stroke="none"
              />
            ))}
            {topSvgData.panels.map((p, i) => (
              <polygon
                key={`tp-${i}`}
                points={p.points}
                fill={`url(#topPanelGrad${i})`}
                stroke="none"
              />
            ))}
            {topSvgData.backLines.map((l, i) => (
              <line
                key={`tbl-${i}`}
                x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
                stroke="rgba(154, 123, 255, 0.15)"
                strokeWidth="0.6"
              />
            ))}
            {topSvgData.lines.map((l, i) => (
              <React.Fragment key={`tl-${i}`}>
                <line
                  x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
                  stroke="rgba(154, 123, 255, 0.06)"
                  strokeWidth="6"
                  filter="url(#topConnGlow)"
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
        <PipBoyScreen className={styles.mfdTopLeft} isLoggedIn={isLoggedIn}>
          <div className={styles.visorTitleWrap} key={activeTab || 'void'}>
            <div className={styles.visorTitle}>{pageInfo.title}</div>
            <div className={styles.visorSubtitle}>{pageInfo.subtitle}</div>
            <div className={styles.visorCursor}>&gt;_</div>
          </div>
        </PipBoyScreen>
        <MFDScreen
          title="VISOR"
          statusColor="green"
          className={styles.mfdTopCenter}
        >
          <div className={styles.visorContent}>
            <div className={styles.visorPinSlot}>
              <div className={styles.visorPinDot} />
              <span>AGENTIC TOOLS</span>
            </div>
            <div className={styles.visorPinSlot}>
              <div className={styles.visorPinDot} />
              <span>CROSSROADS</span>
            </div>
          </div>
        </MFDScreen>
        <PipBoyScreen className={styles.mfdTopRight} isLoggedIn={isLoggedIn} tabs={['SYS', 'NET']}>
          <div className={styles.visorPinSlot}>
            <div className={styles.visorPinDot} />
            <span>NB-7741</span>
          </div>
        </PipBoyScreen>
      </div>

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
