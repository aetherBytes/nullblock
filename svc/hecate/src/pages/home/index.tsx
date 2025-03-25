import React, { useState, useEffect } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import Echo from '@components/echo/echo';
import SystemChat from '@components/system-chat/system-chat';

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

  const automaticResponses = [
    {
      alert: "Error: Invalid input pattern.",
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
      alert: "Error: Security mismatch.",
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
    setCurrentRoom(room);
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
            text: "System: Wallet detected.",
            type: "message"
          },
          {
            id: 3,
            text: "System Update: Ready to connect.",
            type: "update"
          },
          {
            id: 4,
            text: "Connect Wallet",
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
            text: "Error: No wallet found.",
            type: "critical"
          },
          {
            id: 3,
            text: "System: Wallet required for connection.",
            type: "message"
          },
          {
            id: 4,
            text: "Install Phantom",
            type: "action",
            action: () => window.open('https://phantom.app/', '_blank'),
            actionText: "Install Wallet"
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
          // First try to connect
          const resp = await provider.connect();
          const walletPubKey = resp.publicKey.toString();
          
          // Request signature for new connections
          await requestSignature(provider, walletPubKey);
          
          // If we get here, both connection and signature were successful
          setPublicKey(walletPubKey);
          setWalletConnected(true);
          setShowEcho(true);
          localStorage.setItem('walletPublickey', walletPubKey);
          localStorage.setItem('chatCollapsedState', 'true');
          updateAuthTime();
          
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
            text: "Error: Authentication failed. Please retry.",
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
            text: "System: Disconnected from neural interface.",
            type: "message"
          }, {
            id: 2,
            text: "System Alert: Session terminated. Re-authentication required for next connection.",
            type: "alert"
          }]);
          setMessageIndex(0);
        } catch (error) {
          console.error('Error disconnecting from Phantom:', error);
        }
      }
    }
  };

  return (
    <>
      <div className={styles.backgroundImage} />
      <StarsCanvas />
      <div className={styles.scene}>
        <div className={styles.fire}></div>
      </div>
      <SystemChat 
        messages={messages} 
        isEchoActive={showEcho} 
        onUserInput={handleUserInput}
        currentRoom={currentRoom}
        onRoomChange={handleRoomChange}
      />
      {showEcho && <Echo publicKey={publicKey} onDisconnect={handleDisconnect} />}
    </>
  );
};

export default Home;
