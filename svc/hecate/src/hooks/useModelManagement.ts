import { useState, useEffect, useRef } from 'react';

export const useModelManagement = (publicKey: string | null) => {
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [currentSelectedModel, setCurrentSelectedModel] = useState<string | null>(null);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [defaultModelReady, setDefaultModelReady] = useState(false);
  const [defaultModelLoaded, setDefaultModelLoaded] = useState(false);
  const [modelsCached, setModelsCached] = useState(false);
  const [lastStatusMessageModel, setLastStatusMessageModel] = useState<string | null>(null);
  const [isModelChanging, setIsModelChanging] = useState(false);
  const [sessionStartTime] = useState<Date>(new Date());

  const isLoadingModelsRef = useRef(false);
  const defaultModelLoadingRef = useRef(false);

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
      if (status.current_model) {
        console.log('✅ Model already loaded on backend:', status.current_model);
        setCurrentSelectedModel(status.current_model);
        setDefaultModelReady(true);

        if (lastStatusMessageModel !== status.current_model) {
          setLastStatusMessageModel(status.current_model);
        }

        return;
      }

      const defaultModelName = 'deepseek/deepseek-chat-v3.1:free';
      console.log('Loading default model:', defaultModelName);

      const success = await hecateAgent.setModel(defaultModelName);
      if (success) {
        setCurrentSelectedModel(defaultModelName);
        setDefaultModelReady(true);
        setLastStatusMessageModel(defaultModelName);

        console.log('✅ Default model loaded successfully');
      } else {
        console.warn('Failed to load default model');
      }
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

      console.log(`=== MODEL SWITCH START: ${currentSelectedModel} -> ${modelName} ===`);

      const { hecateAgent } = await import('../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        throw new Error('Failed to connect to Hecate agent');
      }

      console.log(`Loading new model: ${modelName}`);

      const success = await hecateAgent.setModel(modelName);

      if (!success) {
        throw new Error(`Failed to switch to model: ${modelName}`);
      }

      console.log(`Successfully switched to model: ${modelName}`);
      console.log(`=== MODEL SWITCH COMPLETE ===`);

      setCurrentSelectedModel(modelName);

    } catch (error) {
      console.error('Error setting model:', error);
    } finally {
      setIsModelChanging(false);
    }
  };

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
    sessionStartTime,
    loadDefaultModel,
    loadAvailableModels,
    handleModelSelection,
    isLoadingModelsRef,
    defaultModelLoadingRef
  };
};