import React, { useState, useEffect } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';
import styles from './index.module.scss';
import StarsCanvas from '@components/stars/stars';
import Echo from '@components/echo/echo';
import SystemChat from '@components/system-chat/system-chat';

type MessageType = 'message' | 'alert' | 'critical' | 'upgrade' | 'action';

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

  const addMessage = (message: ChatMessage) => {
    setMessages(prev => [...prev, message]);
  };

  useEffect(() => {
    const phantomExists = 'phantom' in window && (window as any).phantom?.solana;
    setHasPhantom(!!phantomExists);

    const getInitialMessages = (): ChatMessage[] => {
      const baseMessages: ChatMessage[] = [
        {
          id: 1,
          text: "System: Initializing biological interface scan...",
          type: "message"
        }
      ];

      if (phantomExists) {
        return [
          ...baseMessages,
          {
            id: 2,
            text: "System: Web3 interface detected. Compatibility check in progress...",
            type: "message"
          },
          {
            id: 3,
            text: "System Upgrade: Neural interface ready for synchronization.",
            type: "upgrade"
          },
          {
            id: 4,
            text: "Connect Wallet",
            type: "action",
            action: manualConnect,
            actionText: "Initialize Neural Link"
          }
        ];
      } else {
        return [
          ...baseMessages,
          {
            id: 2,
            text: "System Critical Alert: No neural interface detected.",
            type: "critical"
          },
          {
            id: 3,
            text: "System: Scanning for alternative connection protocols...",
            type: "message"
          },
          {
            id: 4,
            text: "System Alert: Web3 capability required for neural synchronization.",
            type: "alert"
          },
          {
            id: 5,
            text: "System: Phantom neural interface recommended for optimal compatibility.",
            type: "message"
          },
          {
            id: 6,
            text: "Install Phantom",
            type: "action",
            action: () => window.open('https://phantom.app/', '_blank'),
            actionText: "Download Neural Interface"
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

  useEffect(() => {
    const checkWalletConnection = async () => {
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;
        if (provider) {
          if (provider.isConnected) {
            try {
              const connectedPublicKey = await provider.getPublicKey();
              setPublicKey(connectedPublicKey.toString());
              setWalletConnected(true);
              setShowEcho(true);
              localStorage.setItem('walletPublickey', connectedPublicKey.toString());
            } catch (error) {
              console.error('Failed to get public key:', error);
              localStorage.removeItem('walletPublickey');
            }
          } else {
            const savedPublicKey = localStorage.getItem('walletPublickey');
            if (savedPublicKey) {
              try {
                await provider.connect();
                setPublicKey(savedPublicKey);
                setWalletConnected(true);
                setShowEcho(true);
              } catch (error) {
                console.error('Failed to auto-reconnect:', error);
                localStorage.removeItem('walletPublickey');
              }
            }
          }
        }
      }
    };

    checkWalletConnection();
  }, []);

  const manualConnect = async () => {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider) {
        try {
          const { publicKey } = await provider.connect();
          setPublicKey(publicKey.toString());
          setWalletConnected(true);
          setShowEcho(true);
          localStorage.setItem('walletPublickey', publicKey.toString());
          addMessage({
            id: messages.length + 1,
            text: "System Upgrade: Neural link established. Initializing enhanced interface...",
            type: "upgrade"
          });
        } catch (error) {
          console.error('Manual connect error:', error);
          addMessage({
            id: messages.length + 1,
            text: "System Critical Alert: Neural link failed. Please retry connection.",
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
          localStorage.removeItem('walletPublickey');
          setMessages([]);
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
      {!walletConnected && (
        <SystemChat messages={messages} />
      )}
      {showEcho && <Echo publicKey={publicKey} onDisconnect={handleDisconnect} />}
    </>
  );
};

export default Home;
