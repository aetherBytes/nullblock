import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
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
const DEV_UNLOCK_CONNECT = false;

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
  const [chatHistoryVisible, setChatHistoryVisible] = useState(true);
  const [lockedWarning, setLockedWarning] = useState(false);
  const lockedWarningTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [hasUnreadMessages, setHasUnreadMessages] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [activeModelName, setActiveModelName] = useState<string | null>(null);
  const chatInputRef = useRef<HTMLTextAreaElement>(null);
  const historyEndRef = useRef<HTMLDivElement>(null);
  const layerRef = useRef<HTMLDivElement>(null);
  const settingsRef = useRef<HTMLDivElement>(null);
  const memcacheRef = useRef<HTMLDivElement>(null);
  const mobileMenuRef = useRef<HTMLDivElement>(null);
  const hamburgerRef = useRef<HTMLButtonElement>(null);

  const { unlockedTabs } = useWalletTools(publicKey || null, { autoFetch: true });
  const {
    filterCommands,
    getHelpText,
    getToolListText,
    getMcpStatusText,
  } = useCommands(undefined, false);

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
    agentService.getAgentCapabilities('hecate').then((res) => {
      if (res.success && res.data?.model_name) {
        const name = res.data.model_name as string;
        const short = name.includes('/') ? name.split('/').pop()! : name;
        setActiveModelName(short.replace(/:free$/, ''));
      }
    }).catch(() => {});
  }, []);

  useEffect(() => {
    setWelcomeVisible(showWelcome);
  }, [showWelcome]);

  useEffect(() => {
    if (historyEndRef.current && chatMessages.length > 0) {
      historyEndRef.current.scrollIntoView({ behavior: 'smooth', block: 'end' });
    }
  }, [chatMessages.length]);

  // Flag unread when history is hidden and a new agent message arrives
  useEffect(() => {
    if (chatMessages.length === 0) return;
    const last = chatMessages[chatMessages.length - 1];
    if (!chatHistoryVisible && last.sender === 'hecate') {
      setHasUnreadMessages(true);
    }
  }, [chatMessages.length, chatHistoryVisible]);

  // Snap to bottom when history is toggled on
  useEffect(() => {
    if (chatHistoryVisible && historyEndRef.current && chatMessages.length > 0) {
      historyEndRef.current.scrollIntoView({ behavior: 'instant', block: 'end' });
    }
  }, [chatHistoryVisible, chatMessages.length]);

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

  useEffect(() => {
    if (!mobileMenuOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (
        mobileMenuRef.current && !mobileMenuRef.current.contains(e.target as Node) &&
        (!hamburgerRef.current || !hamburgerRef.current.contains(e.target as Node))
      ) {
        setMobileMenuOpen(false);
      }
    };
    const handleResize = () => {
      if (window.innerWidth > 768) setMobileMenuOpen(false);
    };
    document.addEventListener('mousedown', handleClickOutside);
    window.addEventListener('resize', handleResize);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      window.removeEventListener('resize', handleResize);
    };
  }, [mobileMenuOpen]);

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

  const handleLockedClick = useCallback(() => {
    setLockedWarning(true);
    if (lockedWarningTimer.current) clearTimeout(lockedWarningTimer.current);
    lockedWarningTimer.current = setTimeout(() => setLockedWarning(false), 2500);
  }, []);

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
            <nav className={`${styles.voidNav} ${styles.voidNavSingle}`}>
              <button
                className={`${styles.navItem} ${styles.navItemActive}`}
                onClick={onEnterCrossroads}
                disabled={pendingCrossroadsTransition}
              >
                {pendingCrossroadsTransition ? 'Aligning...' : 'Crossroads'}
              </button>
            </nav>
            {lockedWarning && (
              <div className={styles.lockedWarning}>Coming soon</div>
            )}
            <button
              className={`${styles.connectButton} ${!DEV_UNLOCK_CONNECT ? styles.connectButtonLocked : ''}`}
              onClick={DEV_UNLOCK_CONNECT ? onConnectWallet : handleLockedClick}
              title={DEV_UNLOCK_CONNECT ? 'Connect Wallet' : 'Coming soon'}
            >
              Connect
            </button>
            {activeTab === 'crossroads' && (
              <button
                ref={hamburgerRef}
                className={styles.hamburgerButton}
                onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
                aria-label="Toggle menu"
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                  <path d="M3 6h18M3 12h18M3 18h18" />
                </svg>
              </button>
            )}
          </>
        )}
      </div>

      {/* Crossroads submenu bar (pre-login, desktop) */}
      {!publicKey && activeTab === 'crossroads' && (
        <div className={styles.crossroadsSubmenuBar}>
          {CROSSROADS_ITEMS.map((item, index) => {
            const isHype = item.id === 'hype';
            return (
              <button
                key={item.id}
                className={`${styles.submenuBarItem} ${isHype && crossroadsSection === item.id ? styles.submenuBarItemActive : ''} ${!isHype ? styles.submenuBarItemLocked : ''}`}
                onClick={isHype ? () => onCrossroadsSectionChange?.(item.id) : handleLockedClick}
                style={{ animationDelay: `${index * 0.04}s` }}
              >
                {item.label}
              </button>
            );
          })}
        </div>
      )}

      {/* Crossroads mobile dropdown (pre-login) */}
      {!publicKey && activeTab === 'crossroads' && mobileMenuOpen && (
        <div className={styles.mobileMenuDropdown} ref={mobileMenuRef}>
          {CROSSROADS_ITEMS.map((item) => {
            const isHype = item.id === 'hype';
            return (
              <button
                key={item.id}
                className={`${styles.mobileMenuItem} ${isHype && crossroadsSection === item.id ? styles.mobileMenuItemActive : ''} ${!isHype ? styles.mobileMenuItemLocked : ''}`}
                onClick={() => {
                  if (isHype) {
                    onCrossroadsSectionChange?.(item.id);
                    setMobileMenuOpen(false);
                  } else {
                    handleLockedClick();
                  }
                }}
              >
                {item.label}
              </button>
            );
          })}
        </div>
      )}

      {/* Chat History */}
      {chatHistoryVisible && chatMessages.length > 0 && (
        <div className={styles.chatHistoryLayer} ref={layerRef}>
          <div className={styles.chatHistoryTrack}>
            {chatMessages.slice(-MAX_VISIBLE_MESSAGES).map((msg) => (
              <div
                key={msg.id}
                className={`${styles.chatBubble} ${msg.sender === 'user' ? styles.chatBubbleUser : styles.chatBubbleAgent}`}
              >
                {msg.sender === 'hecate' && <span className={styles.chatBubbleSender}>HEXY</span>}
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

      {/* Chat Input */}
      {(
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
            <span className={`${styles.chatLabel} ${hasUnreadMessages && !chatHistoryVisible ? styles.chatLabelAlert : ''}`}>{isProcessing ? 'THINKING' : `HEXY${activeModelName ? `: ${activeModelName}` : ''}`}</span>
            <div className={styles.chatInputWrapper}>
              <textarea
                ref={chatInputRef}
                className={styles.chatInput}
                placeholder={isProcessing ? 'Hexy is thinking...' : 'Talk to Hexy... (/ for commands)'}
                value={chatInput}
                onChange={handleChatInputChange}
                onKeyDown={handleChatKeyDown}
                onFocus={() => setChatFocused(true)}
                onBlur={() => setChatFocused(false)}
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
            {chatMessages.length > 0 && (
              <button
                className={`${styles.chatHistoryToggle} ${chatHistoryVisible ? styles.chatHistoryToggleActive : ''} ${hasUnreadMessages && !chatHistoryVisible ? styles.chatHistoryToggleAlert : ''}`}
                onClick={() => {
                  setChatHistoryVisible((v) => !v);
                  if (!chatHistoryVisible) setHasUnreadMessages(false);
                }}
                title={chatHistoryVisible ? 'Hide chat history' : 'Show chat history'}
                aria-label={chatHistoryVisible ? 'Hide chat history' : 'Show chat history'}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  {chatHistoryVisible ? (
                    <>
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                      <circle cx="12" cy="12" r="3" />
                    </>
                  ) : (
                    <>
                      <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94" />
                      <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19" />
                      <line x1="1" y1="1" x2="23" y2="23" />
                    </>
                  )}
                </svg>
              </button>
            )}
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
