import React, { useState, useCallback } from 'react';
import { useChat } from '../../hooks/useChat';
import { useModelManagement } from '../../hooks/useModelManagement';
import VoidChatHUD from './chat/VoidChatHUD';

interface VoidHUDProps {
  publicKey: string | null;
  isActive?: boolean;
  loginAnimationPhase?: string;
  onAgentResponseReceived?: (messageId: string) => void;
  onFirstMessage?: () => void;
  glowActive?: boolean;
  hecatePanelOpen?: boolean;
  onHecatePanelChange?: (open: boolean) => void;
  hasOverlappingPanels?: boolean;
}

const VoidHUD: React.FC<VoidHUDProps> = ({
  publicKey,
  isActive = true,
  loginAnimationPhase,
  onAgentResponseReceived,
  onFirstMessage,
  glowActive = false,
  hecatePanelOpen = false,
  hasOverlappingPanels = false,
}) => {
  // Use chat hook for agent management
  const { activeAgent, setActiveAgent, getImagesForMessage } = useChat(publicKey);

  // Use model management hook for current model display
  const { currentSelectedModel, agentHealthStatus } = useModelManagement(publicKey, activeAgent);

  if (!isActive || loginAnimationPhase !== 'complete') {
    return null;
  }

  return (
    <VoidChatHUD
      publicKey={publicKey}
      isActive={isActive}
      onFirstMessage={onFirstMessage}
      onAgentResponseReceived={onAgentResponseReceived}
      glowActive={glowActive}
      currentModel={currentSelectedModel}
      activeAgent={activeAgent}
      setActiveAgent={setActiveAgent}
      agentHealthStatus={agentHealthStatus}
      getImagesForMessage={getImagesForMessage}
      showHistory={hecatePanelOpen}
      hasOverlappingPanels={hasOverlappingPanels}
    />
  );
};

export default VoidHUD;
