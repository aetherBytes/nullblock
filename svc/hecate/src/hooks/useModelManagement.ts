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
      const { hecateAgent } = await import('../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        return;
      }

      const status = await hecateAgent.getModelStatus();

      // Check agent health status - prioritize health.overall_status over status
      const healthStatus = status.health?.overall_status || status.status;

      setAgentHealthStatus(healthStatus === 'healthy' ? 'healthy' : 'unhealthy');

      if (status.current_model && healthStatus === 'healthy') {
        setCurrentSelectedModel(status.current_model);
        setDefaultModelReady(true);

        if (lastStatusMessageModel !== status.current_model) {
          setLastStatusMessageModel(status.current_model);
        }

        return;
      } else if (healthStatus !== 'healthy') {
        setDefaultModelReady(false);
        setCurrentSelectedModel(null);
        return;
      }

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
      return;
    }

    if (modelsCached && availableModels.length > 0) {
      return;
    }

    try {
      isLoadingModelsRef.current = true;
      setIsLoadingModels(true);

      const { hecateAgent } = await import('../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        return;
      }

      const modelsData = await hecateAgent.getAvailableModels();
      setAvailableModels(modelsData.models || []);

      if (modelsData.current_model) {
        if (!currentSelectedModel) {
          setCurrentSelectedModel(modelsData.current_model);
        }
        setDefaultModelLoaded(true);
        return;
      }

      setModelsCached(true);

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
      return;
    }

    try {
      setIsModelChanging(true);

      const { agentService } = await import('../common/services/agent-service');

      const connected = await agentService.connect();
      if (!connected) {
        throw new Error(`Failed to connect to ${activeAgent} agent`);
      }

      const response = await agentService.setAgentModel(activeAgent, modelName);

      if (!response.success) {
        throw new Error(`Failed to switch to model: ${modelName}`);
      }

      setCurrentSelectedModel(modelName);

    } catch (error) {
      console.error(`Error setting model for ${activeAgent}:`, error);
    } finally {
      setIsModelChanging(false);
    }
  };

  // Model filtering helper functions
  const getFreeModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => model.available && (model.tier === 'economical' || model.cost_per_1k_tokens === 0))
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getFastModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => model.available && model.tier === 'fast')
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getThinkerModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => {
        if (!model.available) return false;
        const name = (model.display_name || model.name).toLowerCase();
        return (model.capabilities && (model.capabilities.includes('reasoning') || model.capabilities.includes('reasoning_tokens'))) ||
               name.includes('reasoning') || name.includes('think') || name.includes('r1') || name.includes('o1');
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getImageModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => {
        if (!model.available) return false;
        return model.architecture?.output_modalities?.includes('image') ||
               (model.capabilities && model.capabilities.includes('image_generation'));
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  // Effect to sync model state when active agent changes
  useEffect(() => {
    const syncAgentModel = async () => {
      if (!publicKey) return;

      try {
        const { agentService } = await import('../common/services/agent-service');
        const connected = await agentService.connect();

        if (!connected) {
          return;
        }

        // Query the agent's health endpoint to get current model
        const response = await agentService.getAgentHealth(activeAgent);

        if (response.success && response.data?.current_model) {
          // Update the appropriate model state
          if (activeAgent === 'hecate') {
            setHecateModel(response.data.current_model);
          } else {
            setSirenModel(response.data.current_model);
          }
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
    defaultModelLoadingRef,
    // Model filtering functions
    getFreeModels,
    getFastModels,
    getThinkerModels,
    getImageModels
  };
};