import { useState, useEffect, useRef } from 'react';

export const useModelManagement = (publicKey: string | null, activeAgent: 'hecate' | 'siren' = 'hecate') => {
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [hecateModel, setHecateModel] = useState<string | null>(null);
  const [sirenModel, setSirenModel] = useState<string | null>(null);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [defaultModelReady, setDefaultModelReady] = useState(false);
  const [defaultModelLoaded, setDefaultModelLoaded] = useState(false);
  const [modelsCached, setModelsCached] = useState(false);
  const [lastStatusMessageModel, setLastStatusMessageModel] = useState<string | null>(null);
  const [isModelChanging, setIsModelChanging] = useState(false);
  const [agentHealthStatus, setAgentHealthStatus] = useState<'healthy' | 'unhealthy' | 'unknown'>('unknown');
  const [sessionStartTime] = useState<Date>(new Date());

  const isLoadingModelsRef = useRef(false);
  const defaultModelLoadingRef = useRef(false);

  // Computed property for current selected model based on active agent
  const currentSelectedModel = activeAgent === 'hecate' ? hecateModel : sirenModel;
  const setCurrentSelectedModel = activeAgent === 'hecate' ? setHecateModel : setSirenModel;

  const loadDefaultModel = async () => {
    if (defaultModelReady || !publicKey || defaultModelLoadingRef.current) {
      if (defaultModelReady && currentSelectedModel) {
        return;
      }
      return;
    }

    defaultModelLoadingRef.current = true;

    try {
      console.log('🚀 Loading default model immediately...');

      const { hecateAgent } = await import('../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        console.warn('Failed to connect to Hecate agent for default model');
        return;
      }

      const status = await hecateAgent.getModelStatus();

      // Check agent health status - prioritize health.overall_status over status
      const healthStatus = status.health?.overall_status || status.status;
      console.log('🔍 Agent health check:', {
        rawHealthStatus: status.health?.overall_status,
        rawStatus: status.status,
        finalHealthStatus: healthStatus,
        modelsAvailable: status.health?.models_available || status.models_available
      });

      setAgentHealthStatus(healthStatus === 'healthy' ? 'healthy' : 'unhealthy');

      if (status.current_model && healthStatus === 'healthy') {
        console.log('✅ Model already validated on backend:', status.current_model);
        setCurrentSelectedModel(status.current_model);
        setDefaultModelReady(true);

        if (lastStatusMessageModel !== status.current_model) {
          setLastStatusMessageModel(status.current_model);
        }

        return;
      } else if (healthStatus !== 'healthy') {
        console.warn('⚠️ Agent is unhealthy, models not available');
        setDefaultModelReady(false);
        setCurrentSelectedModel(null);
        return;
      }

      console.warn('⚠️ No validated model from backend - backend should validate model at startup');
      console.warn('💡 Model selection will use router auto-selection per request');
      setCurrentSelectedModel(null);
      setDefaultModelReady(false);
    } catch (error) {
      console.error('Error loading default model:', error);
    } finally {
      defaultModelLoadingRef.current = false;
    }
  };

  const loadAvailableModels = async () => {
    if (isLoadingModelsRef.current) {
      console.log('Model loading already in progress (ref guard), skipping duplicate call');
      return;
    }

    if (modelsCached && availableModels.length > 0) {
      console.log('Models already cached for this session, skipping API call');
      return;
    }

    try {
      console.log('=== LOADING MODELS START ===');

      isLoadingModelsRef.current = true;
      setIsLoadingModels(true);

      const { hecateAgent } = await import('../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        console.warn('Failed to connect to Hecate agent for model loading');
        return;
      }

      const modelsData = await hecateAgent.getAvailableModels();
      setAvailableModels(modelsData.models || []);

      console.log('Available models loaded:', modelsData.models?.length || 0);
      console.log('Current model from backend:', modelsData.current_model);

      if (modelsData.current_model) {
        if (!currentSelectedModel) {
          setCurrentSelectedModel(modelsData.current_model);
        }
        setDefaultModelLoaded(true);

        console.log('=== LOADING MODELS END (model already set) ===');
        return;
      }

      console.log('=== LOADING MODELS END ===');

      setModelsCached(true);
      console.log('Models successfully cached for session started at:', sessionStartTime.toISOString());

    } catch (error) {
      console.error('Error loading available models:', error);
      setDefaultModelLoaded(false);
    } finally {
      isLoadingModelsRef.current = false;
      setIsLoadingModels(false);
    }
  };

  const handleModelSelection = async (modelName: string) => {
    if (isModelChanging) return;

    if (currentSelectedModel === modelName) {
      console.log(`Already using model: ${modelName}`);
      return;
    }

    try {
      setIsModelChanging(true);

      console.log(`=== MODEL SWITCH START (${activeAgent}): ${currentSelectedModel} -> ${modelName} ===`);

      const { agentService } = await import('../common/services/agent-service');

      const connected = await agentService.connect();
      if (!connected) {
        throw new Error(`Failed to connect to ${activeAgent} agent`);
      }

      console.log(`Loading new model for ${activeAgent}: ${modelName}`);

      const response = await agentService.setAgentModel(activeAgent, modelName);

      if (!response.success) {
        throw new Error(`Failed to switch to model: ${modelName}`);
      }

      console.log(`Successfully switched ${activeAgent} to model: ${modelName}`);
      console.log(`=== MODEL SWITCH COMPLETE ===`);

      setCurrentSelectedModel(modelName);

    } catch (error) {
      console.error(`Error setting model for ${activeAgent}:`, error);
    } finally {
      setIsModelChanging(false);
    }
  };

  // Effect to sync model state when active agent changes
  useEffect(() => {
    const syncAgentModel = async () => {
      if (!publicKey) return;

      try {
        console.log(`🔄 Active agent changed to ${activeAgent}, syncing model state...`);

        const { agentService } = await import('../common/services/agent-service');
        const connected = await agentService.connect();

        if (!connected) {
          console.warn(`Failed to connect to get ${activeAgent} model status`);
          return;
        }

        // Query the agent's health endpoint to get current model
        const response = await agentService.getAgentHealth(activeAgent);

        if (response.success && response.data?.current_model) {
          console.log(`✅ ${activeAgent} current model: ${response.data.current_model}`);

          // Update the appropriate model state
          if (activeAgent === 'hecate') {
            setHecateModel(response.data.current_model);
          } else {
            setSirenModel(response.data.current_model);
          }
        } else {
          console.log(`ℹ️ ${activeAgent} has no current model set yet`);
        }
      } catch (error) {
        console.error(`Error syncing ${activeAgent} model:`, error);
      }
    };

    syncAgentModel();
  }, [activeAgent, publicKey]);

  return {
    availableModels,
    setAvailableModels,
    currentSelectedModel,
    setCurrentSelectedModel,
    isLoadingModels,
    setIsLoadingModels,
    defaultModelReady,
    setDefaultModelReady,
    defaultModelLoaded,
    setDefaultModelLoaded,
    modelsCached,
    setModelsCached,
    lastStatusMessageModel,
    setLastStatusMessageModel,
    isModelChanging,
    setIsModelChanging,
    agentHealthStatus,
    setAgentHealthStatus,
    sessionStartTime,
    loadDefaultModel,
    loadAvailableModels,
    handleModelSelection,
    isLoadingModelsRef,
    defaultModelLoadingRef
  };
};