import React, { useState, useCallback, useMemo } from 'react';
import * as THREE from 'three';
import VoidScopes from './scopes/VoidScopes';
import VoidChatHUD from './chat/VoidChatHUD';
import { useTaskManagement } from '../../hooks/useTaskManagement';
import { useModelManagement } from '../../hooks/useModelManagement';
import { useChat } from '../../hooks/useChat';
import { useUserProfile } from '../../hooks/useUserProfile';
import { useApiKeyCheck } from '../../hooks/useApiKeyCheck';

interface VoidHUDProps {
  publicKey: string | null;
  isActive?: boolean;
  loginAnimationPhase?: string;
  onUserMessageSent?: (messageId: string) => void;
  onAgentResponseReceived?: (messageId: string) => void;
  onFirstMessage?: () => void;
  tendrilHit?: boolean;
  hecatePanelOpen?: boolean;
  onHecatePanelChange?: (open: boolean) => void;
}

const VoidHUD: React.FC<VoidHUDProps> = ({
  publicKey,
  isActive = true,
  loginAnimationPhase,
  onUserMessageSent,
  onAgentResponseReceived,
  onFirstMessage,
  tendrilHit = false,
  hecatePanelOpen = false,
  onHecatePanelChange,
}) => {
  // Panel state
  const [scopesOpen, setScopesOpen] = useState(false);
  const [chatHistoryOpen, setChatHistoryOpen] = useState(false);

  // Use chat hook for agent management
  const {
    activeAgent,
    setActiveAgent,
    getImagesForMessage,
    addTaskNotification,
  } = useChat(publicKey);

  // Get user profile for API key check
  const { userProfile } = useUserProfile(publicKey);
  const { hasApiKeys } = useApiKeyCheck(userProfile?.id || null);

  // Use model management hook
  const {
    availableModels,
    currentSelectedModel,
    agentHealthStatus,
    isLoadingModels,
    handleModelSelection,
    loadAvailableModels,
    getFreeModels,
    getFastModels,
    getThinkerModels,
    getImageModels,
  } = useModelManagement(publicKey, activeAgent);

  // Local state for model selection UI
  const [showModelSelection, setShowModelSelection] = useState(false);

  // Use task management hook
  const taskManagement = useTaskManagement(
    publicKey,
    {},
    true,
    addTaskNotification
  );

  // Create task management interface for VoidScopes
  const taskManagementInterface = useMemo(() => ({
    tasks: taskManagement.tasks,
    isLoading: taskManagement.isLoading,
    createTask: taskManagement.createTask,
    startTask: taskManagement.startTask,
    pauseTask: taskManagement.pauseTask,
    resumeTask: taskManagement.resumeTask,
    cancelTask: taskManagement.cancelTask,
    retryTask: taskManagement.retryTask,
    processTask: taskManagement.processTask,
    deleteTask: taskManagement.deleteTask,
  }), [taskManagement]);

  // Create model management interface for VoidScopes
  const modelManagementInterface = useMemo(() => ({
    isLoadingModelInfo: isLoadingModels,
    currentSelectedModel,
    availableModels,
    showModelSelection,
    setShowModelSelection,
    handleModelSelection,
    loadAvailableModels,
    getFreeModels,
    getFastModels,
    getThinkerModels,
    getImageModels,
  }), [
    isLoadingModels,
    currentSelectedModel,
    availableModels,
    showModelSelection,
    handleModelSelection,
    loadAvailableModels,
    getFreeModels,
    getFastModels,
    getThinkerModels,
    getImageModels,
  ]);

  // Handle hecate panel toggle (opens both scopes and chat history)
  const handleHecatePanelToggle = useCallback((open: boolean) => {
    setScopesOpen(open);
    setChatHistoryOpen(open);
    onHecatePanelChange?.(open);
  }, [onHecatePanelChange]);

  // Sync with external hecate panel state
  React.useEffect(() => {
    if (hecatePanelOpen !== (scopesOpen && chatHistoryOpen)) {
      setScopesOpen(hecatePanelOpen);
      setChatHistoryOpen(hecatePanelOpen);
    }
  }, [hecatePanelOpen]);

  // Handle scopes panel change
  const handleScopesChange = useCallback((open: boolean) => {
    setScopesOpen(open);
  }, []);

  if (!isActive || loginAnimationPhase !== 'complete') return null;

  return (
    <>
      {/* Scopes Panel (Bottom Left) - Only visible when Hecate panel is open */}
      {hecatePanelOpen && (
        <VoidScopes
          isActive={isActive}
          isOpen={scopesOpen}
          onOpenChange={handleScopesChange}
          taskManagement={taskManagementInterface}
          modelManagement={modelManagementInterface}
          availableModels={availableModels}
          activeAgent={activeAgent}
          setActiveAgent={setActiveAgent}
          hasApiKey={hasApiKeys === true}
        />
      )}

      {/* Chat HUD (Bottom Right) - Input always visible, history only when Hecate panel open */}
      <VoidChatHUD
        publicKey={publicKey}
        isActive={isActive}
        onFirstMessage={onFirstMessage}
        onUserMessageSent={onUserMessageSent}
        onAgentResponseReceived={onAgentResponseReceived}
        tendrilHit={tendrilHit}
        currentModel={currentSelectedModel}
        activeAgent={activeAgent}
        setActiveAgent={setActiveAgent}
        agentHealthStatus={agentHealthStatus}
        getImagesForMessage={getImagesForMessage}
        showHistory={hecatePanelOpen}
      />
    </>
  );
};

export default VoidHUD;
