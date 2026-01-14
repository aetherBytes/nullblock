import { useState, useEffect } from 'react';
import {
  isAuthenticated,
  restoreSession,
  createAuthChallenge,
  verifyAuthChallenge,
  checkMCPHealth,
} from '../common/services/mcp-api';

export const useAuthentication = () => {
  const [mcpAuthenticated, setMcpAuthenticated] = useState<boolean>(false);
  const [mcpHealthStatus, setMcpHealthStatus] = useState<any>(null);

  useEffect(() => {
    const initializeMCP = async () => {
      try {
        const hasSession = restoreSession();

        setMcpAuthenticated(hasSession && isAuthenticated());

        const health = await checkMCPHealth();

        setMcpHealthStatus(health);
      } catch (error) {
        console.error('Failed to initialize MCP:', error);
        setMcpHealthStatus(null);
      }
    };

    initializeMCP();
  }, []);

  const handleMCPAuthentication = async (publicKey: string | null) => {
    if (!publicKey) {
      alert('Please connect your wallet first');

      return;
    }

    try {
      const challenge = await createAuthChallenge(publicKey);

      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;

        if (provider) {
          const message = new TextEncoder().encode(challenge.message);
          const signedMessage = await provider.signMessage(message, 'utf8');
          const signature = Array.from(signedMessage.signature);

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

  return {
    mcpAuthenticated,
    setMcpAuthenticated,
    mcpHealthStatus,
    setMcpHealthStatus,
    handleMCPAuthentication,
  };
};
