import React, { useState, useEffect, useRef } from 'react';
import xLogo from '../../assets/images/X_logo_black.png';
import nullviewLogo from '../../assets/images/nullview_logo.png';
import type { MissionData } from '../../common/services/api';
import {
  fetchWalletData,
  fetchUserProfile,
  fetchAscentLevel,
  fetchActiveMission,
} from '../../common/services/api';
import {
  isAuthenticated,
  restoreSession,
  createAuthChallenge,
  verifyAuthChallenge,
  checkMCPHealth,
} from '../../common/services/mcp-api';
// Removed separate dashboard imports - all functionality is now integrated into HUD tabs
import HecateHud from '../hecateHud';
import { hecateAgent, type ChatMessage as HecateChatMessage, type HecateResponse } from '../../common/services/hecate-agent';
import styles from './hud.module.scss';

type Theme = 'null' | 'light' | 'dark';

interface SystemStatus {
  hud: boolean;
  mcp: boolean;
  orchestration: boolean;
  agents: boolean;
  portfolio: boolean;
  defi: boolean;
  social: boolean;
  arbitrage: boolean;
  hecate: boolean;
  erebus: boolean;
}

interface HUDProps {
  publicKey: string | null;
  onDisconnect: () => void;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'cyber' | 'light' | 'dark') => void;
  systemStatus: SystemStatus;
}

interface UserProfile {
  id: string;
  ascent: number;
  nether: number | null;
  cacheValue: number;
  memories: number;
  matrix: {
    level: string;
    rarity: string;
    status: string;
  };
}

interface AscentLevel {
  level: number;
  title: string;
  description: string;
  progress: number;
  nextLevel: number;
  nextTitle: string;
  nextDescription: string;
  accolades: string[];
}







const HUD: React.FC<HUDProps> = ({
  publicKey,
  onDisconnect,
  onConnectWallet,
  theme = 'light',
  onClose,
  onThemeChange,
  systemStatus,
}) => {
  const [walletData, setWalletData] = useState<any>(null);
  const [userProfile, setUserProfile] = useState<UserProfile>({
    id: publicKey ? `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol` : '',
    ascent: 1,
    nether: null,
    cacheValue: 0,
    memories: 0,
    matrix: {
      level: 'NONE',
      rarity: 'NONE',
      status: 'N/A',
    },
  });
  const [ascentLevel, setAscentLevel] = useState<AscentLevel | null>(null);
  const [walletName, setWalletName] = useState<string | null>(null);
  const [activeMission, setActiveMission] = useState<MissionData | null>(null);
  // Remove separate dashboard overlays - everything will be integrated into HUD tabs
  const [mcpAuthenticated, setMcpAuthenticated] = useState<boolean>(false);
  const [mcpHealthStatus, setMcpHealthStatus] = useState<any>(null);
  const [nullviewState, setNulleyeState] = useState<
    | 'base'
    | 'response'
    | 'question'
    | 'thinking'
    | 'alert'
    | 'error'
    | 'warning'
    | 'success'
    | 'processing'
    | 'idle'
  >('base');
  const [showHecateHud, setShowHecateHud] = useState<boolean>(false);
  const [hecateHudActiveTab, setHecateHudActiveTab] = useState<
    'tasks' | 'mcp' | 'logs' | 'agents' | 'hecate'
  >('hecate');
  const [mainHudActiveTab, setMainHudActiveTab] = useState<
    'status' | 'tasks' | 'agents' | 'mcp' | 'logs' | 'hecate'
  >('status');







  // MCP initialization effect
  useEffect(() => {
    const initializeMCP = async () => {
      try {
        // Try to restore existing session
        const hasSession = restoreSession();

        setMcpAuthenticated(hasSession && isAuthenticated());

        // Check MCP health
        const health = await checkMCPHealth();

        setMcpHealthStatus(health);
      } catch (error) {
        console.error('Failed to initialize MCP:', error);
        setMcpHealthStatus(null);
      }
    };

    initializeMCP();
  }, []);

  useEffect(() => {
    const loadWalletData = async () => {
      if (publicKey) {
        try {
          // Skip old backend wallet data fetch for now - using Erebus for wallet ops
          console.log('Wallet connected:', publicKey);
          
          // Check for saved wallet name
          const savedName = localStorage.getItem(`walletName_${publicKey}`);
          if (savedName) {
            setWalletName(savedName);
          }
          
          // TODO: Update to use Erebus wallet data endpoints when available
          // const data = await fetchWalletData(publicKey);
          // setWalletData(data);

          // Skip old backend profile fetch for now - using Erebus for wallet ops
          // TODO: Update to use Erebus user profile endpoints when available
          // try {
          //   const profileData = await fetchUserProfile(publicKey);

          //   // Check if the wallet has Nectar tokens
          //   const hasNectarToken = profileData.active_tokens.includes('NECTAR');

          //   // Update user profile with wallet data and username if available
          //   setUserProfile((prev) => ({
          //     ...prev,
          //     nether: hasNectarToken ? data.balance : null,
          //     cacheValue: data.balance || 0, // Set cache value to wallet balance
          //     id: profileData.username
          //       ? `@${profileData.username}`
          //       : `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol`,
          //   }));

          //   // Log the profile data to debug
          //   console.log('Profile data received:', profileData);
          //   console.log('Username:', profileData.username);
          // } catch (profileError) {
          //   console.error('Failed to fetch user profile:', profileError);
          //   // Fallback to just updating with wallet data
          //   setUserProfile((prev) => ({
          //     ...prev,
          //     nether: null, // Set to null if we can't determine if Nectar exists
          //     cacheValue: data.balance || 0, // Set cache value to wallet balance
          //   }));
          // }

          // Skip old backend ascent fetch for now - using Erebus for wallet ops
          // TODO: Update to use Erebus ascent endpoints when available
          // try {
          //   const ascentData = await fetchAscentLevel(publicKey);
          //   // Convert AscentLevelData to AscentLevel
          //   setAscentLevel({
          //     level: ascentData.level,
          //     title: ascentData.name,
          //     description: ascentData.description,
          //     progress: ascentData.progress,
          //     nextLevel: ascentData.level + 1,
          //     nextTitle: `Level ${ascentData.level + 1}`,
          //     nextDescription: 'Next level description will be available soon.',
          //     accolades: ascentData.accolades,
          //   });
          //   // Update the ascent value in userProfile
          //   setUserProfile((prev) => ({
          //     ...prev,
          //     ascent: ascentData.level,
          //   }));
          // } catch (ascentError) {
          //   console.error('Failed to fetch ascent level:', ascentError);
          // }

          // Skip old backend mission fetch for now - using Erebus for wallet ops
          // TODO: Update to use Erebus mission endpoints when available
          // try {
          //   const missionData = await fetchActiveMission(publicKey);
          //   setActiveMission(missionData);
          // } catch (missionError) {
          //   console.error('Failed to fetch active mission:', missionError);
          // }
        } catch (error) {
          console.error('Failed to fetch wallet data:', error);
        }
      }
    };

    loadWalletData();
  }, [publicKey]);




  const handleMCPAuthentication = async () => {
    if (!publicKey) {
      alert('Please connect your wallet first');

      return;
    }

    try {
      // Create auth challenge
      const challenge = await createAuthChallenge(publicKey);

      // For Phantom wallet, sign the challenge
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;

        if (provider) {
          const message = new TextEncoder().encode(challenge.message);
          const signedMessage = await provider.signMessage(message, 'utf8');
          const signature = Array.from(signedMessage.signature);

          // Verify the signature with MCP
          const authResponse = await verifyAuthChallenge(
            publicKey,
            signature.toString(),
            'phantom',
          );

          if (authResponse.success) {
            setMcpAuthenticated(true);
            alert('Successfully authenticated with MCP!');
          } else {
            alert(`Authentication failed: ${authResponse.message}`);
          }
        }
      }
    } catch (error) {
      console.error('MCP authentication failed:', error);
      alert('Authentication failed. Please try again.');
    }
  };

  // All dashboard functionality will be integrated directly into existing tab system

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <div className={styles.nullblockTitle}>
        NULLBLOCK
      </div>

      {/* NULLVIEWS - Living system indicator */}
      <div
        className={`${styles.nullview} ${styles[nullviewState]}`}
        onClick={() => {
          if (!publicKey) {
            // Enhanced feedback for locked state
            setNulleyeState('error');
            setTimeout(() => setNulleyeState('base'), 1500);
            alert(
              '🔒 SECURE ACCESS REQUIRED\n\nConnect your Web3 wallet to unlock the NullView interface and access advanced features.',
            );

            return;
          }

          // Navigate to hecate tab in main HUD
          setMainHudActiveTab('hecate');
          setNulleyeState('processing');
        }}
        title={!publicKey ? '🔒 Connect wallet to unlock NullView' : '🔓 Access NullView Interface'}
      >
        <div className={styles.pulseRing}></div>
        <div className={styles.dataStream}>
          <div className={styles.streamLine}></div>
          <div className={styles.streamLine}></div>
          <div className={styles.streamLine}></div>
        </div>
        <div className={styles.lightningContainer}>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
          <div className={styles.lightningArc}></div>
        </div>
        <div className={styles.staticField}></div>
        <div className={styles.coreNode}></div>
      </div>

      <div className={styles.navbarButtons}>
        <button
          className={`${styles.walletMenuButton} ${publicKey ? styles.connected : ''}`}
          onClick={publicKey ? onDisconnect : () => onConnectWallet()}
          title={publicKey ? 'Disconnect Wallet' : 'Connect Wallet'}
        >
          <span className={styles.walletMenuText}>{publicKey ? 'Disconnect' : 'Connect'}</span>
        </button>
        <button
          className={styles.docsMenuButton}
          onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
          title="Documentation & Developer Resources"
        >
          <span className={styles.docsMenuText}>Docs</span>
        </button>
      </div>
    </nav>
  );





              {/* Duplicate items for seamless scrolling */}
              {leaderboardData.map((entry) => (
                <div
                  key={`${entry.id}-duplicate`}
                  className={styles.leaderboardItem}
                  onClick={() => {
                    setSelectedUplink({
                      id: entry.id,
                      name: entry.id,
                      status: 'active',
                      icon: '👤',
                      details: {
                        description: `Status profile for ${entry.id}`,
                        stats: [
                          {
                            label: 'Rank',
                            value: `#${entry.rank}`,
                          },
                          {
                            label: 'Ascent',
                            value: entry.ascent,





  // Add essential state for main HUD tabs functionality
  
  const renderHomeScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.innerHudMenuBar}>
        <button 
          className={`${styles.menuButton} ${mainHudActiveTab === 'status' ? styles.active : ''}`}
          onClick={() => setMainHudActiveTab('status')}
        >
          Status
        </button>
        
        {publicKey && (
          <>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'tasks' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('tasks')}
            >
              Tasks
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'agents' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('agents')}
            >
              Agents
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'mcp' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('mcp')}
            >
              MCP
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'logs' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('logs')}
            >
              Logs
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'hecate' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('hecate')}
            >
              Hecate
            </button>
          </>
        )}
      </div>
      <div className={styles.homeContent}>
        <div className={styles.landingContent}>
          <div className={styles.mainHudContent}>
            <div className={styles.placeholder}>
              <p>Main HUD tabs functionality will be implemented here.</p>
              <p>Currently showing: {mainHudActiveTab}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
