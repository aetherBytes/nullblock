import React, { useState, useEffect } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import Echo from '@components/echo/echo';
import DigitizingText from '../../components/digitizing-text';
import powerOn from '@assets/images/echo_bot_night.png';
import powerOff from '@assets/images/echo_bot_white.png';
import nyxImage from '@assets/images/night_wolf_1.png';

type MessageType = 'message' | 'alert' | 'critical' | 'update' | 'action' | 'user' | 'welcome' | 'system';

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
  const [isDigitizing, setIsDigitizing] = useState<boolean>(false);
  const [showWelcomeText, setShowWelcomeText] = useState<boolean>(true);
  const [echoScreenSelected, setEchoScreenSelected] = useState<boolean>(false);
  const [currentTheme, setCurrentTheme] = useState<'null' | 'light'>('light');
  const [showConnectButton, setShowConnectButton] = useState<boolean>(false);
  const [textComplete, setTextComplete] = useState<boolean>(false);
  const [showNyxPopup, setShowNyxPopup] = useState<boolean>(false);
  const [nyxMessages, setNyxMessages] = useState<ChatMessage[]>([]);

  // Initialize state from localStorage on component mount
  useEffect(() => {
    // Check if we have a saved wallet connection
    const savedPublicKey = localStorage.getItem('walletPublickey');
    const lastAuth = localStorage.getItem('lastAuthTime');
    const hasSeenEcho = localStorage.getItem('hasSeenEcho');
    const savedTheme = localStorage.getItem('currentTheme');
    
    // Set initial states based on localStorage
    if (savedPublicKey && lastAuth && isSessionValid()) {
      setPublicKey(savedPublicKey);
      setWalletConnected(true);
      setShowEcho(true);
      setShowWelcomeText(false);
    } else {
      // Ensure welcome text is shown for new users
      setShowWelcomeText(true);
      setWalletConnected(false);
      setShowEcho(false);
    }
    
    if (savedTheme) {
      setCurrentTheme(savedTheme as 'null' | 'light');
    }
  }, []);

  // Hide welcome text when ECHO screen is open
  useEffect(() => {
    if (showEcho) {
      setShowWelcomeText(false);
    }
  }, [showEcho]);

  // Show connect button when digitized text is complete
  useEffect(() => {
    console.log('Text complete effect triggered:', { textComplete, walletConnected, showEcho });
    if (textComplete && !walletConnected && !showEcho) {
      console.log('Setting showConnectButton to true');
      setShowConnectButton(true);
    }
  }, [textComplete, walletConnected, showEcho]);

  // Add a fallback timer to show the connect button after a reasonable time
  useEffect(() => {
    if (!walletConnected && !showEcho && showWelcomeText) {
      const fallbackTimer = setTimeout(() => {
        console.log('Fallback timer triggered - showing connect button');
        setShowConnectButton(true);
      }, 5000); // 5 seconds fallback
      
      return () => clearTimeout(fallbackTimer);
    }
  }, [walletConnected, showEcho, showWelcomeText]);

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
            text: "System: Interface detected.",
            type: "message"
          },
          {
            id: 3,
            text: "System Update: Awaiting connection.",
            type: "update"
          },
          {
            id: 4,
            text: "Connect",
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
            text: "Error: Interface not found.",
            type: "critical"
          },
          {
            id: 3,
            text: "System: Interface required for access.",
            type: "message"
          },
          {
            id: 4,
            text: "Acquire Interface",
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
            setShowWelcomeText(false);
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
          localStorage.setItem('walletPublickey', walletPubKey);
          localStorage.setItem('hasSeenEcho', 'true');
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
          localStorage.removeItem('hasSeenEcho');
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
          setMessages([{
            id: 1,
            text: "System: Interface disconnected.",
            type: "message"
          }, {
            id: 2,
            text: "System Alert: Session terminated. Re-authentication required.",
            type: "alert"
          }]);
          setMessageIndex(0);
          
          // Show welcome text and reset text completion state
          setShowWelcomeText(true);
          setTextComplete(false);
          setShowConnectButton(false);
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  // Handle digitized text completion
  const handleTextComplete = () => {
    console.log('Text complete callback triggered');
    setTextComplete(true);
    // Hide the welcome text after animation completes
    setTimeout(() => {
      setShowWelcomeText(false);
    }, 3000);
    // Directly set showConnectButton to true if conditions are met
    if (!walletConnected && !showEcho) {
      console.log('Setting showConnectButton to true directly from callback');
      setShowConnectButton(true);
    }
  };

  // Add this after other useEffect hooks
  useEffect(() => {
    // Initialize Nyx messages
    const initialNyxMessages: ChatMessage[] = [
      {
        id: 1,
        text: "Welcome to Nullblock. The digital void awaits your presence.",
        type: "welcome"
      },
      {
        id: 2,
        text: "System: Nyx interface initialized. Echo chamber active.",
        type: "system"
      },
      {
        id: 3,
        text: "This is a read-only view of the system's communication history. Connect your wallet to access full functionality.",
        type: "message"
      },
      {
        id: 4,
        text: "Alert: Translation matrix inactive. Core systems locked.",
        type: "alert"
      },
      {
        id: 5,
        text: "System: Memory Card storage unavailable. Connect wallet to enable persistent data storage.",
        type: "system"
      }
    ];
    setNyxMessages(initialNyxMessages);
  }, []);

  const handleNyxClick = () => {
    setShowNyxPopup(true);
  };

  const handleCloseNyxPopup = () => {
    setShowNyxPopup(false);
  };

  return (
    <div className={`${styles.appContainer} ${styles[`theme-${currentTheme}`]}`}>
      <div className={styles.backgroundImage} />
      <StarsCanvas theme={currentTheme} />
      <a href="https://twitter.com/nullblock_io" target="_blank" rel="noopener noreferrer" className={styles.socialLink} />
      <div className={`${styles.scene} ${showEcho ? styles.echoActive : ''}`}>
        <div className={styles.fire} onClick={manualConnect}></div>
        <div className={styles.campParts}></div>
        <div className={styles.nyx} onClick={handleNyxClick}></div>
        <div className={styles.campForeTree}></div>
      </div>
      {showWelcomeText && !showEcho && (
        <DigitizingText 
          text="Welcome to Nullblock." 
          duration={3000}
          theme={currentTheme === 'null' ? 'null-dark' : 'light'}
          onComplete={handleTextComplete}
        />
      )}
      {showEcho && <Echo 
        publicKey={publicKey} 
        onDisconnect={handleDisconnect}
        theme={currentTheme}
        onClose={() => {
          setShowEcho(false);
          // Don't reset wallet connection when just closing the Echo component
          // Only reset the welcome text and connect button state
          setShowWelcomeText(true);
          setTextComplete(false);
          setShowConnectButton(false);
        }}
        onThemeChange={(theme) => {
          if (theme === 'cyber') {
            setCurrentTheme('null');
          } else {
            setCurrentTheme(theme as 'null' | 'light');
          }
        }}
        messages={messages}
        onUserInput={handleUserInput}
        currentRoom={currentRoom}
        onRoomChange={handleRoomChange}
      />}

      {showNyxPopup && (
        <>
          <div className={styles.nyxPopupOverlay} onClick={handleCloseNyxPopup} />
          <div className={styles.nyxPopup}>
            <div className={styles.nyxPopupHeader}>
              <h2>Nyx Echo Chamber</h2>
              <button className={styles.closeButton} onClick={handleCloseNyxPopup}>×</button>
            </div>
            <div className={styles.nyxPopupContent}>
              {nyxMessages.map((message) => (
                <div 
                  key={message.id} 
                  className={`${styles.nyxMessage} ${
                    message.type === 'alert' ? styles.alert : 
                    message.type === 'system' ? styles.system :
                    message.type === 'welcome' ? styles.welcome : ''
                  }`}
                >
                  {message.text}
                </div>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default Home;