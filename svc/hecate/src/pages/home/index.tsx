import React, { useState, useEffect } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import Echo from '@components/echo/echo';
import SystemChat from '@components/system-chat/system-chat';
import DigitizingText from '../../components/digitizing-text';

type MessageType = 'message' | 'alert' | 'critical' | 'update' | 'action' | 'user';

interface ChatMessage {
  id: number;
  text: string;
  type: MessageType;
  action?: () => void;
  actionText?: string;
}

const Home: React.FC = () => {
  const [walletConnected, setWalletConnected] = useState<boolean>(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [showEcho, setShowEcho] = useState<boolean>(false);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [messageIndex, setMessageIndex] = useState<number>(0);
  const [hasPhantom, setHasPhantom] = useState<boolean>(false);
  const [currentRoom, setCurrentRoom] = useState<string>('/logs');
  const [chatCollapsed, setChatCollapsed] = useState<boolean>(true);
  const [isDigitizing, setIsDigitizing] = useState<boolean>(false);
  const [showWelcomeText, setShowWelcomeText] = useState<boolean>(true);
  const [echoScreenSelected, setEchoScreenSelected] = useState<boolean>(false);
  const [currentTheme, setCurrentTheme] = useState<'null' | 'light'>('light');

  // Hide welcome text when ECHO screen is open
  useEffect(() => {
    if (showEcho) {
      setShowWelcomeText(false);
    }
  }, [showEcho]);

  const automaticResponses = [
    {
      alert: "Error: Invalid pattern detected.",
      message: "System: Recalibrating..."
    },
    {
      alert: "Error: Protocol mismatch.",
      message: "System: Searching alternatives..."
    },
    {
      alert: "Error: Connection unstable.",
      message: "System: Resyncing..."
    },
    {
      alert: "Error: Security breach.",
      message: "System: Realigning..."
    },
    {
      alert: "Error: Process failure.",
      message: "System: Rerouting..."
    },
    {
      alert: "Error: Parse failure.",
      message: "System: Recovering..."
    }
  ];

  const getRandomResponse = () => {
    const index = Math.floor(Math.random() * automaticResponses.length);
    return automaticResponses[index];
  };

  const addMessage = (message: ChatMessage) => {
    setMessages(prev => [...prev, message]);
  };

  const handleUserInput = async (input: string) => {
    addMessage({
      id: messages.length + 1,
      text: input,
      type: 'user'
    });

    try {
      const response = await fetch('http://localhost:8000/api/command', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          command: input,
          room: currentRoom 
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to process command');
      }

      const data = await response.json();
      
      // Special handling for /clear command
      if (input.toLowerCase() === '/clear') {
        setMessages([]);
        return;
      }

      // Add each message from the response with a delay
      data.messages.forEach((msg: any, index: number) => {
        setTimeout(() => {
          if (msg.type === 'action') {
            if (msg.action === 'connect_wallet') {
              addMessage({
                id: messages.length + 2 + index,
                text: msg.text,
                type: msg.type,
                action: manualConnect,
                actionText: "Connect"
              });
            } else if (msg.action === 'disconnect_wallet') {
              addMessage({
                id: messages.length + 2 + index,
                text: msg.text,
                type: msg.type,
                action: handleDisconnect,
                actionText: "Disconnect"
              });
            }
          } else {
            addMessage({
              id: messages.length + 2 + index,
              text: msg.text,
              type: msg.type as MessageType
            });
          }
        }, 500 * (index + 1));
      });
    } catch (error) {
      console.error('Error processing command:', error);
      addMessage({
        id: messages.length + 2,
        text: 'Error: Command processing failed',
        type: 'critical'
      });
    }
  };

  const handleRoomChange = (room: string) => {
    if (room.startsWith('/theme/')) {
      const themeId = room.split('/theme/')[1] as 'null' | 'light';
      setCurrentTheme(themeId);
      
      // Add a message about theme change
      const themeName = themeId === 'null' ? 'NULL' : 'LIGHT';
      
      addMessage({
        id: messages.length + 1,
        text: `System: Theme changed to ${themeName}`,
        type: 'update'
      });
      
      return;
    }
    setCurrentRoom(room);
    if (room.startsWith('/echo')) {
      setEchoScreenSelected(true);
      setShowWelcomeText(false);
    }
    addMessage({
      id: messages.length + 1,
      text: `System: Switched to ${room}`,
      type: 'update'
    });
  };

  useEffect(() => {
    const phantomExists = 'phantom' in window && (window as any).phantom?.solana;
    setHasPhantom(!!phantomExists);

    // Check wallet connection on mount
    if (phantomExists) {
      checkWalletConnection();
    }

    const getInitialMessages = (): ChatMessage[] => {
      const baseMessages: ChatMessage[] = [
        {
          id: 1,
          text: "System: Initializing...",
          type: "message"
        }
      ];

      if (phantomExists) {
        return [
          ...baseMessages,
          {
            id: 2,
            text: "System: Neural interface detected.",
            type: "message"
          },
          {
            id: 3,
            text: "System Update: Awaiting connection.",
            type: "update"
          },
          {
            id: 4,
            text: "Connect Neural Interface",
            type: "action",
            action: manualConnect,
            actionText: "Connect"
          }
        ];
      } else {
        return [
          ...baseMessages,
          {
            id: 2,
            text: "Error: Neural interface not found.",
            type: "critical"
          },
          {
            id: 3,
            text: "System: Interface required for access.",
            type: "message"
          },
          {
            id: 4,
            text: "Acquire Neural Interface",
            type: "action",
            action: () => window.open('https://phantom.app/', '_blank'),
            actionText: "Install Interface"
          }
        ];
      }
    };

    const initialMessages = getInitialMessages();

    const displayNextMessage = () => {
      if (messageIndex < initialMessages.length) {
        addMessage(initialMessages[messageIndex]);
        setMessageIndex(prev => prev + 1);
      }
    };

    if (!walletConnected && messageIndex < initialMessages.length) {
      const timer = setTimeout(displayNextMessage, messageIndex * 400);
      return () => clearTimeout(timer);
    }
  }, [messageIndex, walletConnected]);

  const requestSignature = async (provider: any, publicKey: string) => {
    try {
      const message = `Authenticate ECHO Interface\nTimestamp: ${Date.now()}`;
      const encodedMessage = new TextEncoder().encode(message);
      await provider.signMessage(encodedMessage, "utf8");
    } catch (error) {
      throw new Error('Authentication failed');
    }
  };

  const SESSION_TIMEOUT = 30 * 60 * 1000; // 30 minutes in milliseconds

  const isSessionValid = () => {
    const lastAuth = localStorage.getItem('lastAuthTime');
    if (!lastAuth) return false;
    
    const timeSinceAuth = Date.now() - parseInt(lastAuth);
    return timeSinceAuth < SESSION_TIMEOUT;
  };

  const updateAuthTime = () => {
    localStorage.setItem('lastAuthTime', Date.now().toString());
  };

  const checkWalletConnection = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        const savedPublicKey = localStorage.getItem('walletPublickey');
        const lastAuth = localStorage.getItem('lastAuthTime');
        
        if (savedPublicKey && lastAuth && isSessionValid()) {
          try {
            // Try to reconnect with existing session
            await provider.connect({ onlyIfTrusted: true });
            
            // If we get here, connection was successful
            setPublicKey(savedPublicKey);
            setWalletConnected(true);
            setShowEcho(true);
            return; // Exit early on successful reconnection
          } catch (error) {
            console.log('Auto-reconnect failed:', error);
          }
        }
        
        // Clear session data if we get here (either expired or failed)
        localStorage.removeItem('walletPublickey');
        localStorage.removeItem('lastAuthTime');
        localStorage.removeItem('hasSeenEcho');
        setWalletConnected(false);
        setPublicKey(null);
        setShowEcho(false);
      }
    }
  };

  const manualConnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          const resp = await provider.connect();
          const walletPubKey = resp.publicKey.toString();
          
          await requestSignature(provider, walletPubKey);
          
          setPublicKey(walletPubKey);
          setWalletConnected(true);
          setShowEcho(true);
          setChatCollapsed(true);
          localStorage.setItem('walletPublickey', walletPubKey);
          localStorage.setItem('chatCollapsedState', 'true');
          updateAuthTime();
          
          setShowWelcomeText(false);
          
          addMessage({
            id: messages.length + 1,
            text: "System: Connected. Loading interface...",
            type: "message"
          });
        } catch (error) {
          console.error('Connection error:', error);
          // Clear all session data on failure
          localStorage.removeItem('walletPublickey');
          localStorage.removeItem('lastAuthTime');
          setWalletConnected(false);
          setPublicKey(null);
          setShowEcho(false);
          addMessage({
            id: messages.length + 1,
            text: "Error: Authentication failed. Retry required.",
            type: "critical"
          });
        }
      }
    }
  };

  const handleDisconnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          await provider.disconnect();
          setWalletConnected(false);
          setPublicKey(null);
          setShowEcho(false);
          // Clear all session data
          localStorage.removeItem('walletPublickey');
          localStorage.removeItem('lastAuthTime');
          localStorage.removeItem('hasSeenEcho');
          localStorage.removeItem('chatCollapsedState');
          setMessages([{
            id: 1,
            text: "System: Neural interface disconnected.",
            type: "message"
          }, {
            id: 2,
            text: "System Alert: Session terminated. Re-authentication required.",
            type: "alert"
          }]);
          setMessageIndex(0);
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  useEffect(() => {
    const handleExpandChat = () => {
      setChatCollapsed(false);
      
      // Don't start digitizing animation if ECHO is shown
      if (showEcho) {
        return;
      }
      
      // Start digitizing animation
      setIsDigitizing(true);
      
      // Add digitizing welcome message
      addMessage({
        id: messages.length + 1,
        text: "INITIALIZING DIGITAL INTERFACE...",
        type: "alert"
      });
      
      // Add digitizing effect messages with delays
      setTimeout(() => {
        addMessage({
          id: messages.length + 2,
          text: "LOADING NEURAL NETWORK...",
          type: "message"
        });
      }, 1000);
      
      setTimeout(() => {
        addMessage({
          id: messages.length + 3,
          text: "CALIBRATING QUANTUM MATRIX...",
          type: "message"
        });
      }, 2000);
      
      setTimeout(() => {
        addMessage({
          id: messages.length + 4,
          text: "ESTABLISHING SECURE CONNECTION...",
          type: "message"
        });
      }, 3000);
      
      setTimeout(() => {
        addMessage({
          id: messages.length + 5,
          text: "ECHO INTERFACE READY",
          type: "update"
        });
        setIsDigitizing(false);
      }, 4000);
      
      // Add welcome message if no matrix is found
      if (!localStorage.getItem('hasMatrix')) {
        setTimeout(() => {
          addMessage({
            id: messages.length + 6,
            text: "System Alert: Matrix Integration Required",
            type: "alert"
          });
          addMessage({
            id: messages.length + 7,
            text: "Welcome to the void. Your journey begins with the Matrix.",
            type: "message"
          });
          addMessage({
            id: messages.length + 8,
            text: "The Matrix unlocks enhanced capabilities:",
            type: "message"
          });
          addMessage({
            id: messages.length + 9,
            text: "• Advanced algorithms\n• Real-time analysis\n• Strategy deployment\n• Priority access\n• Enhanced security",
            type: "message"
          });
          addMessage({
            id: messages.length + 10,
            text: "Matrix tiers determine your power level.",
            type: "message"
          });
          addMessage({
            id: messages.length + 11,
            text: "MARKETPLACE",
            type: "action",
            action: () => window.dispatchEvent(new CustomEvent('navigateToMarket')),
            actionText: "[ ACQUIRE MATRIX ]"
          });
        }, 4500);
      }
    };

    window.addEventListener('expandSystemChat', handleExpandChat);
    return () => window.removeEventListener('expandSystemChat', handleExpandChat);
  }, [messages, showEcho]);

  return (
    <div className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]}`}>
      <div className={styles.backgroundImage} />
      <StarsCanvas theme={currentTheme} />
      <div className={styles.scene}>
        <div className={styles.fire}></div>
      </div>
      {showWelcomeText && !showEcho && (
        <DigitizingText 
          text="Welcome to Nullblock. Interfaces for the new world." 
          duration={0}
          theme={currentTheme === 'null' ? 'null-dark' : 'light'}
        />
      )}
      <SystemChat 
        messages={messages} 
        isEchoActive={showEcho} 
        onUserInput={handleUserInput}
        currentRoom={currentRoom}
        onRoomChange={handleRoomChange}
        isCollapsed={chatCollapsed}
        onCollapsedChange={setChatCollapsed}
        isDigitizing={isDigitizing}
        theme={currentTheme}
      />
      {showEcho && <Echo 
        publicKey={publicKey} 
        onDisconnect={handleDisconnect}
        onExpandChat={() => {
          window.dispatchEvent(new CustomEvent('expandSystemChat'));
          setChatCollapsed(false);
        }}
        theme={currentTheme}
        onClose={() => setShowEcho(false)}
        onThemeChange={(theme) => {
          if (theme === 'cyber') {
            setCurrentTheme('null');
          } else {
            setCurrentTheme(theme as 'null' | 'light');
          }
        }}
      />}
    </div>
  );
};

export default Home;
