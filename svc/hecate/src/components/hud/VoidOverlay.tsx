import React, { useState, useEffect, useLayoutEffect, useRef, useMemo, useCallback } from 'react';
import { agentService } from '../../common/services/agent-service';
import { useWalletTools } from '../../common/hooks/useWalletTools';
import { useCommands } from '../../hooks/useCommands';
import type { SlashCommand } from '../../hooks/useCommands';
import { NULLBLOCK_SERVICE_COWS } from '../../constants/nullblock';
import type { MemCacheSection } from '../memcache';
import type { CrossroadsSection } from './hud';
import MarkdownRenderer from '../common/MarkdownRenderer';
import CommandDropdown from '../void-experience/chat/CommandDropdown';
import NullblockLogo from './NullblockLogo';
import styles from './VoidOverlay.module.scss';

const DEV_SHOW_ALL_COW_TABS = true;

interface PreLoginMessage {
  id: string;
  text: string;
  sender: 'user' | 'hecate';
}

const MAX_VISIBLE_MESSAGES = 20;

const BASE_MEMCACHE_ITEMS: { id: MemCacheSection; icon: string; label: string }[] = [
  { id: 'engrams', icon: '◈', label: 'Engrams' },
  { id: 'stash', icon: '⬡', label: 'Stash' },
  { id: 'agents', icon: '◉', label: 'Agents' },
  { id: 'consensus', icon: '⚖', label: 'Consensus' },
  { id: 'model', icon: '◎', label: 'Model' },
];

const CROSSROADS_ITEMS: { id: CrossroadsSection; label: string }[] = [
  { id: 'hype', label: 'Hype' },
  { id: 'marketplace', label: 'Marketplace' },
  { id: 'agents', label: 'Agents' },
  { id: 'tools', label: 'Tools' },
  { id: 'cows', label: 'COWs' },
];

interface VoidOverlayProps {
  onOpenSynapse: () => void;
  onTabSelect: (tab: 'crossroads' | 'memcache') => void;
  onDisconnect: () => void;
  onConnectWallet?: () => void;
  onResetToVoid?: () => void;
  showWelcome?: boolean;
  onDismissWelcome?: () => void;
  publicKey?: string | null;
  activeTab?: 'crossroads' | 'memcache' | null;
  memcacheSection?: MemCacheSection;
  onMemcacheSectionChange?: (section: MemCacheSection) => void;
  crossroadsSection?: CrossroadsSection;
  onCrossroadsSectionChange?: (section: CrossroadsSection) => void;
  onEnterCrossroads?: () => void;
  pendingCrossroadsTransition?: boolean;
}

const VoidOverlay: React.FC<VoidOverlayProps> = ({
  onOpenSynapse,
  onTabSelect,
  onDisconnect,
  onConnectWallet,
  onResetToVoid,
  showWelcome = false,
  onDismissWelcome,
  publicKey,
  activeTab,
  memcacheSection = 'engrams',
  onMemcacheSectionChange,
  crossroadsSection = 'hype',
  onCrossroadsSectionChange,
  onEnterCrossroads,
  pendingCrossroadsTransition = false,
}) => {
  const [welcomeVisible, setWelcomeVisible] = useState(showWelcome);
  const [welcomeFading, setWelcomeFading] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [chatInput, setChatInput] = useState('');
  const [chatFocused, setChatFocused] = useState(false);
  const [chatMessages, setChatMessages] = useState<PreLoginMessage[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [showCommandDropdown, setShowCommandDropdown] = useState(false);
  const [commandSelectedIndex, setCommandSelectedIndex] = useState(0);
  const chatInputRef = useRef<HTMLTextAreaElement>(null);
  const historyEndRef = useRef<HTMLDivElement>(null);
  const layerRef = useRef<HTMLDivElement>(null);
  const trackRef = useRef<HTMLDivElement>(null);
  const settingsRef = useRef<HTMLDivElement>(null);
  const memcacheRef = useRef<HTMLDivElement>(null);

  const { unlockedTabs } = useWalletTools(publicKey || null, { autoFetch: true });
  const {
    filterCommands,
    getHelpText,
    getToolListText,
    getMcpStatusText,
  } = useCommands('http://localhost:3000', !!publicKey);

  const filteredCommands = chatInput.startsWith('/') ? filterCommands(chatInput) : [];

  const MEMCACHE_ITEMS = useMemo(() => {
    const items = [...BASE_MEMCACHE_ITEMS];
    const insertIndex = 3;

    NULLBLOCK_SERVICE_COWS.forEach((cow) => {
      const isUnlocked = unlockedTabs.includes(cow.id) || DEV_SHOW_ALL_COW_TABS;

      if (isUnlocked) {
        items.splice(insertIndex, 0, {
          id: cow.id as MemCacheSection,
          icon: cow.menuIcon,
          label: cow.name,
        });
      }
    });

    return items;
  }, [unlockedTabs]);

  useEffect(() => {
    setWelcomeVisible(showWelcome);
  }, [showWelcome]);

  useEffect(() => {
    if (historyEndRef.current && chatMessages.length > 0) {
      historyEndRef.current.scrollIntoView({ behavior: 'smooth', block: 'end' });
    }
  }, [chatMessages.length]);

  // Scroll-aware 3D: style based on visual distance from viewport bottom
  const updateBubbleStyles = useCallback(() => {
    const layer = layerRef.current;
    const track = trackRef.current;
    if (!layer || !track) return;

    const scrollBottom = layer.scrollTop + layer.clientHeight;
    const effectRange = layer.clientHeight * 0.5;

    const children = track.querySelectorAll<HTMLElement>('[data-bubble]');
    children.forEach((el) => {
      const elBottom = el.offsetTop + el.offsetHeight;
      const dist = scrollBottom - elBottom;
      const t = Math.min(Math.max(dist / effectRange, 0), 1);

      const depth = t * 1000;
      // Flat at bottom (readable), tilt ramps after 15% of the range
      const tiltT = Math.max(t - 0.15, 0) / 0.85;
      const angle = tiltT * 55;

      el.style.transform = `translateZ(${-depth}px) rotateX(${angle}deg)`;
      el.style.opacity = `${Math.max(1 - t * t, 0)}`;
      el.style.filter = t > 0.3 ? `blur(${(t - 0.3) * 8}px)` : '';
    });
  }, []);

  useEffect(() => {
    const layer = layerRef.current;
    if (!layer) return;

    let rafId: number;
    const onUpdate = () => {
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(updateBubbleStyles);
    };

    layer.addEventListener('scroll', onUpdate, { passive: true });
    window.addEventListener('resize', onUpdate, { passive: true });
    return () => {
      layer.removeEventListener('scroll', onUpdate);
      window.removeEventListener('resize', onUpdate);
      cancelAnimationFrame(rafId);
    };
  }, [updateBubbleStyles]);

  // Apply styles before first paint when messages change
  useLayoutEffect(() => {
    updateBubbleStyles();
  }, [chatMessages.length, updateBubbleStyles]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (settingsRef.current && !settingsRef.current.contains(event.target as Node)) {
        setSettingsOpen(false);
      }
    };

    if (settingsOpen) {
      document.addEventListener('mousedown', handleClickOutside);

      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [settingsOpen]);

  const handleDismissWelcome = () => {
    setWelcomeFading(true);
    setTimeout(() => {
      setWelcomeVisible(false);
      setWelcomeFading(false);
      onDismissWelcome?.();
    }, 500);
  };

  const handleCommandSelect = useCallback((command: SlashCommand) => {
    setShowCommandDropdown(false);
    setChatInput('');
    setCommandSelectedIndex(0);

    if (command.action === 'execute') {
      let responseText = '';
      switch (command.name) {
        case '/help':
          responseText = getHelpText();
          break;
        case '/list-tools':
          responseText = getToolListText();
          break;
        case '/mcp':
        case '/status':
          responseText = getMcpStatusText();
          break;
        case '/clear':
          setChatMessages([]);
          return;
        default:
          responseText = `Command ${command.name} not implemented yet.`;
      }
      const agentMsg: PreLoginMessage = {
        id: `cmd-${Date.now()}`,
        text: responseText,
        sender: 'hecate',
      };
      setChatMessages((prev) => [...prev, agentMsg]);
    } else if (command.action === 'tools') {
      const toolListText = getToolListText();
      const agentMsg: PreLoginMessage = {
        id: `tools-${Date.now()}`,
        text: toolListText,
        sender: 'hecate',
      };
      setChatMessages((prev) => [...prev, agentMsg]);
    } else {
      const commandMsg = `Use the ${command.name.replace('/', '')} tool to help me.`;
      setChatInput(commandMsg);
      chatInputRef.current?.focus();
    }
  }, [getHelpText, getToolListText, getMcpStatusText]);

  const handleChatSubmit = useCallback(async () => {
    if (!chatInput.trim() || isProcessing) return;

    if (showCommandDropdown && filteredCommands.length > 0) {
      handleCommandSelect(filteredCommands[commandSelectedIndex]);
      return;
    }

    const message = chatInput.trim();
    const userMsg: PreLoginMessage = {
      id: `u-${Date.now()}`,
      text: message,
      sender: 'user',
    };
    setChatMessages((prev) => [...prev, userMsg]);
    setChatInput('');
    setShowCommandDropdown(false);
    if (chatInputRef.current) {
      chatInputRef.current.style.height = 'auto';
    }

    setIsProcessing(true);
    try {
      const response = await agentService.chatWithAgent('hecate', message);
      if (response.success && response.data) {
        const agentMsg: PreLoginMessage = {
          id: `h-${Date.now()}`,
          text: response.data.content || response.data.message || 'No response.',
          sender: 'hecate',
        };
        setChatMessages((prev) => [...prev, agentMsg]);
      } else {
        const agentMsg: PreLoginMessage = {
          id: `h-${Date.now()}`,
          text: response.error || 'Agent unavailable. Try again later.',
          sender: 'hecate',
        };
        setChatMessages((prev) => [...prev, agentMsg]);
      }
    } catch {
      const agentMsg: PreLoginMessage = {
        id: `h-${Date.now()}`,
        text: 'Could not reach Hecate. Check that services are running.',
        sender: 'hecate',
      };
      setChatMessages((prev) => [...prev, agentMsg]);
    } finally {
      setIsProcessing(false);
      chatInputRef.current?.focus();
    }
  }, [chatInput, isProcessing, showCommandDropdown, filteredCommands, commandSelectedIndex, handleCommandSelect]);

  const handleChatKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (showCommandDropdown && filteredCommands.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setCommandSelectedIndex((prev) => Math.min(prev + 1, filteredCommands.length - 1));
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setCommandSelectedIndex((prev) => Math.max(prev - 1, 0));
        return;
      }
      if (e.key === 'Tab') {
        e.preventDefault();
        const cmd = filteredCommands[commandSelectedIndex];
        if (cmd) setChatInput(cmd.name + ' ');
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        setShowCommandDropdown(false);
        return;
      }
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleChatSubmit();
    }
  }, [handleChatSubmit, showCommandDropdown, filteredCommands, commandSelectedIndex]);

  const handleChatInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setChatInput(value);
    const textarea = e.target;
    textarea.style.height = 'auto';
    textarea.style.height = `${Math.min(textarea.scrollHeight, 120)}px`;

    if (value.startsWith('/')) {
      setShowCommandDropdown(true);
      setCommandSelectedIndex(0);
    } else {
      setShowCommandDropdown(false);
    }
  }, []);

  const handleSettingsClick = () => {
    setSettingsOpen(false);
    onOpenSynapse();
  };

  const handleDisconnectClick = () => {
    setSettingsOpen(false);
    onDisconnect();
  };

  return (
    <>
      {/* Full-width navbar border */}
      <div className={styles.navbarBorder} />

      {/* Top-left: Logo and branding */}
      <div className={styles.logoContainer}>
        <NullblockLogo
          state="base"
          theme="dark"
          size="medium"
          variant="color"
          onClick={onResetToVoid}
          title="Return to Void"
        />
        <div className={styles.nullblockTextLogo} onClick={onResetToVoid} title="Return to Void">
          NULLBLOCK
        </div>
        {!publicKey && (
          <>
            <span className={styles.navbarDivider} />
            <span className={styles.navbarTagline}>Picks and shovels for the new age.</span>
          </>
        )}
      </div>

      {/* Top-right container: Nav + Settings */}
      <div className={styles.topRightContainer}>
        {publicKey && (
          <div className={styles.navWrapper} ref={memcacheRef}>
            {activeTab === 'memcache' && (
              <div className={styles.submenuExtra}>
                {MEMCACHE_ITEMS.slice(2)
                  .reverse()
                  .map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItemExtra} ${memcacheSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onMemcacheSectionChange?.(item.id)}
                        style={{ animationDelay: `${(MEMCACHE_ITEMS.length - 2 - index) * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < MEMCACHE_ITEMS.length - 3 && <span className={styles.navDivider} />}
                    </React.Fragment>
                  ))}
                <span className={styles.navDivider} />
              </div>
            )}

            {activeTab === 'crossroads' && (
              <div className={styles.submenuExtra}>
                {CROSSROADS_ITEMS.slice(2)
                  .reverse()
                  .map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItemExtra} ${crossroadsSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onCrossroadsSectionChange?.(item.id)}
                        style={{ animationDelay: `${(CROSSROADS_ITEMS.length - 2 - index) * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < CROSSROADS_ITEMS.length - 3 && <span className={styles.navDivider} />}
                    </React.Fragment>
                  ))}
                <span className={styles.navDivider} />
              </div>
            )}

            <nav className={styles.voidNav}>
              <button
                className={`${styles.navItem} ${activeTab === 'memcache' ? styles.navItemActive : ''}`}
                onClick={() => onTabSelect('memcache')}
              >
                Mem Cache
              </button>
              <span className={styles.navDivider} />
              <button
                className={`${styles.navItem} ${activeTab === 'crossroads' ? styles.navItemActive : ''}`}
                onClick={() => onTabSelect('crossroads')}
              >
                Crossroads
              </button>

              {activeTab === 'memcache' && (
                <>
                  {MEMCACHE_ITEMS.slice(0, 2).map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItem} ${memcacheSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onMemcacheSectionChange?.(item.id)}
                        style={{ animationDelay: `${index * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < 1 && <span className={styles.submenuDivider} />}
                    </React.Fragment>
                  ))}
                </>
              )}

              {activeTab === 'crossroads' && (
                <>
                  {CROSSROADS_ITEMS.slice(0, 2).map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItem} ${crossroadsSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onCrossroadsSectionChange?.(item.id)}
                        style={{ animationDelay: `${index * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < 1 && <span className={styles.submenuDivider} />}
                    </React.Fragment>
                  ))}
                </>
              )}
            </nav>
          </div>
        )}

        {/* Settings Menu or Connect Button */}
        {publicKey ? (
          <div className={styles.settingsContainer} ref={settingsRef}>
            <button
              className={styles.settingsButton}
              onClick={() => setSettingsOpen(!settingsOpen)}
              title="Settings"
              aria-label="Open settings menu"
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <circle cx="12" cy="12" r="3" />
                <path d="M12 2v4M12 18v4M2 12h4M18 12h4" />
                <path d="M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
              </svg>
            </button>

            {settingsOpen && (
              <div className={styles.settingsDropdown}>
                <button className={styles.settingsItem} onClick={handleSettingsClick}>
                  <span className={styles.settingsIcon}>⚙️</span>
                  <span>Settings</span>
                </button>
                <div className={styles.settingsDivider} />
                <button className={styles.settingsItem} onClick={handleDisconnectClick}>
                  <span className={styles.settingsIcon}>🔌</span>
                  <span>Disconnect</span>
                </button>
              </div>
            )}
          </div>
        ) : (
          <>
            <div className={styles.navWrapper}>
              <div className={styles.submenuExtra}>
                {CROSSROADS_ITEMS.slice(2)
                  .reverse()
                  .map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItemExtra} ${crossroadsSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onCrossroadsSectionChange?.(item.id)}
                        style={{ animationDelay: `${(CROSSROADS_ITEMS.length - 2 - index) * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < CROSSROADS_ITEMS.length - 3 && <span className={styles.navDivider} />}
                    </React.Fragment>
                  ))}
                <span className={styles.navDivider} />
              </div>

              <nav className={`${styles.voidNav} ${styles.voidNavSingle}`}>
                <button
                  className={`${styles.navItem} ${styles.navItemActive}`}
                  onClick={onEnterCrossroads}
                  disabled={pendingCrossroadsTransition}
                >
                  {pendingCrossroadsTransition ? 'Aligning...' : 'Crossroads'}
                </button>

                {CROSSROADS_ITEMS.slice(0, 2).map((item, index) => (
                  <React.Fragment key={item.id}>
                    <button
                      className={`${styles.submenuItem} ${crossroadsSection === item.id ? styles.submenuItemActive : ''}`}
                      onClick={() => onCrossroadsSectionChange?.(item.id)}
                      style={{ animationDelay: `${index * 0.03}s` }}
                    >
                      {item.label}
                    </button>
                    {index < 1 && <span className={styles.submenuDivider} />}
                  </React.Fragment>
                ))}
              </nav>
            </div>
            <button className={styles.connectButton} onClick={onConnectWallet} title="Connect Wallet">
              Connect
            </button>
          </>
        )}
      </div>

      {/* Pre-login Chat History */}
      {!publicKey && chatMessages.length > 0 && (
        <div className={styles.chatHistoryLayer} ref={layerRef}>
          <div className={styles.chatHistoryTrack} ref={trackRef}>
            {chatMessages.slice(-MAX_VISIBLE_MESSAGES).map((msg) => (
              <div
                key={msg.id}
                data-bubble
                className={`${styles.chatBubble} ${msg.sender === 'user' ? styles.chatBubbleUser : styles.chatBubbleAgent}`}
              >
                {msg.sender === 'hecate' && <span className={styles.chatBubbleSender}>HECATE</span>}
                {msg.sender === 'hecate' ? (
                  <MarkdownRenderer content={msg.text} className={styles.chatBubbleMarkdown} />
                ) : (
                  <span className={styles.chatBubbleText}>{msg.text}</span>
                )}
              </div>
            ))}
            <div ref={historyEndRef} />
          </div>
        </div>
      )}

      {/* Pre-login Chat Input */}
      {!publicKey && (
        <div className={`${styles.chatInputLayer} ${chatFocused ? styles.chatInputFocused : ''} ${isProcessing ? styles.chatInputProcessing : ''}`}>
          {showCommandDropdown && filteredCommands.length > 0 && (
            <div className={styles.commandDropdownAnchor}>
              <CommandDropdown
                commands={filteredCommands}
                selectedIndex={commandSelectedIndex}
                onSelect={handleCommandSelect}
                onClose={() => setShowCommandDropdown(false)}
                query={chatInput}
              />
            </div>
          )}
          <div className={styles.chatInputBar}>
            <span className={styles.chatLabel}>{isProcessing ? 'THINKING' : 'HECATE'}</span>
            <div className={styles.chatInputWrapper}>
              <textarea
                ref={chatInputRef}
                className={styles.chatInput}
                placeholder={isProcessing ? 'Hecate is thinking...' : 'Ask Hecate anything... (/ for commands)'}
                value={chatInput}
                onChange={handleChatInputChange}
                onKeyDown={handleChatKeyDown}
                onFocus={() => setChatFocused(true)}
                onBlur={() => setChatFocused(false)}
                disabled={isProcessing}
                rows={1}
              />
              <button
                className={styles.chatSendButton}
                onClick={handleChatSubmit}
                disabled={!chatInput.trim() || isProcessing}
                title="Send message"
                aria-label="Send message"
              >
                {isProcessing ? (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className={styles.spinnerIcon}>
                    <circle cx="12" cy="12" r="10" strokeDasharray="31.4" strokeDashoffset="10" />
                  </svg>
                ) : (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M22 2L11 13" />
                    <path d="M22 2L15 22L11 13L2 9L22 2Z" />
                  </svg>
                )}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Footer bar (pre-login) */}
      {!publicKey && (
        <div className={styles.footerBar}>
          <div className={styles.footerLeft}>
            <a
              href="https://aetherbytes.github.io/nullblock-sdk/"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.footerLink}
            >
              📚 Documentation
            </a>
            <a
              href="https://x.com/Nullblock_io"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.footerLink}
            >
              𝕏 Follow Updates
            </a>
          </div>
          <span className={styles.footerTagline}>
            Discover agents, tools, and workflows. Own the tools that own the future.
          </span>
        </div>
      )}

      {/* First-time Welcome Overlay */}
      {welcomeVisible && (
        <div
          className={`${styles.welcomeOverlay} ${welcomeFading ? styles.fading : ''}`}
          onClick={handleDismissWelcome}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => e.key === 'Enter' && handleDismissWelcome()}
          aria-label="Dismiss welcome message"
        >
          <div className={styles.welcomeContent}>
            <p className={styles.welcomeText}>You have awakened.</p>
            <p className={styles.welcomeHint}>Touch the lights or speak.</p>
            <div className={styles.welcomeDismiss}>
              <span>Click anywhere to begin</span>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default VoidOverlay;
