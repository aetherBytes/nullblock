import React, { useState, useEffect } from 'react';
import { TaskCreationRequest, TaskType, TaskPriority, TaskCategory, SubTaskRequest } from '../../types/tasks';
import { Agent } from '../../types/agents';
import { agentService } from '../../common/services/agent-service';
import styles from './hud.module.scss';

interface TaskCreationFormProps {
  onCreateTask: (request: TaskCreationRequest) => Promise<boolean>;
  isLoading: boolean;
  onCancel: () => void;
  variant?: 'default' | 'embedded' | 'fullscreen';
  availableModels?: any[];
}

const TaskCreationForm: React.FC<TaskCreationFormProps> = ({
  onCreateTask,
  isLoading,
  onCancel,
  variant = 'default',
  availableModels = []
}) => {
  const [formData, setFormData] = useState<TaskCreationRequest>({
    name: '',
    description: '',
    task_type: 'system',
    priority: 'medium',
    category: 'user_assigned',
    auto_start: true,
    user_approval_required: false,
    assigned_agent_id: undefined,
    parameters: {
      preferred_model: undefined,
      temperature: 0.7,
      max_tokens: 2000,
      timeout_ms: 300000,
    },
    dependencies: [],
    sub_tasks: []
  });

  const [subTasks, setSubTasks] = useState<SubTaskRequest[]>([]);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Debug: Log button state
  useEffect(() => {
    console.log('🔘 Button state:', {
      isLoading,
      hasName: !!formData.name.trim(),
      hasDescription: !!formData.description.trim(),
      isDisabled: isLoading || !formData.name.trim() || !formData.description.trim()
    });
  }, [isLoading, formData.name, formData.description]);

  // Fetch agents on mount
  useEffect(() => {
    const fetchAgents = async () => {
      try {
        const response = await agentService.getAgents();
        if (response.success && response.data) {
          setAgents(response.data.agents);
        }
      } catch (err) {
        console.warn('Failed to load agents:', err);
      }
    };
    fetchAgents();
  }, []);

  const taskTypes: { value: TaskType; label: string }[] = [
    { value: 'system', label: 'User Generated' }
  ];

  const priorities: { value: TaskPriority; label: string; color: string }[] = [
    { value: 'urgent', label: 'Urgent', color: '#ff3333' },
    { value: 'high', label: 'High', color: '#ff6b47' },
    { value: 'medium', label: 'Medium', color: '#4ecdc4' },
    { value: 'low', label: 'Low', color: '#95a5a6' }
  ];

  const categories: { value: TaskCategory; label: string }[] = [
    { value: 'user_assigned', label: 'User Generated' }
  ];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!formData.name.trim()) {
      setError('Task name is required');
      return;
    }

    if (!formData.description.trim()) {
      setError('Task description is required');
      return;
    }

    console.log('📋 Submitting task creation request:', formData);

    try {
      const requestData = {
        ...formData,
        sub_tasks: subTasks.filter(st => st.name.trim() && st.description.trim())
      };
      console.log('📤 Calling onCreateTask with:', requestData);
      const success = await onCreateTask(requestData);
      console.log('📤 Task creation result:', success);
      if (success) {
        console.log('✅ Task created successfully, closing form');
        onCancel();
        setFormData({
          name: '',
          description: '',
          task_type: 'system',
          priority: 'medium',
          category: 'user_assigned',
          auto_start: true,
          user_approval_required: false,
          assigned_agent_id: undefined,
          parameters: {
            preferred_model: undefined,
            temperature: 0.7,
            max_tokens: 2000,
            timeout_ms: 300000,
          },
          dependencies: [],
          sub_tasks: []
        });
        setSubTasks([]);
      } else {
        console.error('❌ Task creation failed');
        setError('Failed to create task. Please check the console for details.');
      }
    } catch (err) {
      console.error('❌ Task creation error:', err);
      setError((err as Error).message);
    }
  };

  const handleInputChange = (field: keyof TaskCreationRequest, value: any) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const addSubTask = () => {
    setSubTasks([...subTasks, { name: '', description: '', assigned_agent_id: undefined }]);
  };

  const removeSubTask = (index: number) => {
    setSubTasks(subTasks.filter((_, i) => i !== index));
  };

  const updateSubTask = (index: number, field: keyof SubTaskRequest, value: any) => {
    const updated = [...subTasks];
    updated[index] = { ...updated[index], [field]: value };
    setSubTasks(updated);
  };

  const updateParameter = (key: string, value: any) => {
    setFormData({
      ...formData,
      parameters: { ...formData.parameters, [key]: value }
    });
  };

  return (
    <div className={`${styles.taskCreationForm} ${
      variant === 'embedded' ? styles.embeddedForm : 
      variant === 'fullscreen' ? styles.fullscreenForm : ''
    }`}>
      {variant !== 'fullscreen' && (
        <div className={styles.formHeader}>
          <h5>📋 Create New Task</h5>
          <button
            type="button"
            onClick={onCancel}
            className={styles.closeButton}
            disabled={isLoading}
          >
            ✕
          </button>
        </div>
      )}

      <form onSubmit={handleSubmit}>
        <div className={styles.formGroup}>
          <label htmlFor="taskName" className={styles.formLabel}>
            Task Name *
          </label>
          <input
            id="taskName"
            type="text"
            value={formData.name}
            onChange={(e) => handleInputChange('name', e.target.value)}
            placeholder="Enter task name..."
            className={styles.formInput}
            disabled={isLoading}
            maxLength={100}
            required
          />
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="taskDescription" className={styles.formLabel}>
            Description *
          </label>
          <textarea
            id="taskDescription"
            value={formData.description}
            onChange={(e) => handleInputChange('description', e.target.value)}
            placeholder="Describe what this task should do..."
            className={styles.formTextarea}
            disabled={isLoading}
            maxLength={500}
            rows={3}
            required
          />
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="assignedAgent" className={styles.formLabel}>
            Assigned Agent
          </label>
          <select
            id="assignedAgent"
            value={formData.assigned_agent_id || ''}
            onChange={(e) => handleInputChange('assigned_agent_id', e.target.value || undefined)}
            className={styles.formSelect}
            disabled={isLoading}
          >
            <option value="">Auto (Hecate Orchestrator)</option>
            {agents.map(agent => (
              <option key={agent.name} value={agent.name}>
                {agent.name} - {agent.capabilities?.slice(0, 2).join(', ')}
              </option>
            ))}
          </select>
        </div>

        <div className={styles.formGroup}>
          <label className={styles.formLabel}>Sub-tasks</label>
          {subTasks.map((st, idx) => (
            <div key={idx} className={styles.subTaskRow}>
              <input
                type="text"
                placeholder="Sub-task name"
                value={st.name}
                onChange={(e) => updateSubTask(idx, 'name', e.target.value)}
                className={styles.formInput}
                disabled={isLoading}
              />
              <input
                type="text"
                placeholder="Description"
                value={st.description}
                onChange={(e) => updateSubTask(idx, 'description', e.target.value)}
                className={styles.formInput}
                disabled={isLoading}
              />
              <select
                value={st.assigned_agent_id || ''}
                onChange={(e) => updateSubTask(idx, 'assigned_agent_id', e.target.value || undefined)}
                className={styles.formSelect}
                disabled={isLoading}
              >
                <option value="">Auto</option>
                {agents.map(a => <option key={a.name} value={a.name}>{a.name}</option>)}
              </select>
              <button
                type="button"
                onClick={() => removeSubTask(idx)}
                className={styles.removeSubTaskBtn}
                disabled={isLoading}
              >
                ✕
              </button>
            </div>
          ))}
          <button
            type="button"
            onClick={addSubTask}
            className={styles.addSubTaskBtn}
            disabled={isLoading}
          >
            + Add Sub-task
          </button>
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="preferredModel" className={styles.formLabel}>
            Preferred Model
          </label>
          <select
            id="preferredModel"
            value={formData.parameters?.preferred_model || ''}
            onChange={(e) => updateParameter('preferred_model', e.target.value || undefined)}
            className={styles.formSelect}
            disabled={isLoading}
          >
            <option value="">Use Agent Default</option>
            {availableModels.slice(0, 10).map(model => (
              <option key={model.name} value={model.name}>
                {model.display_name} ({model.tier})
              </option>
            ))}
          </select>
        </div>

        <div className={styles.formGroup}>
          <button
            type="button"
            onClick={() => setShowAdvanced(!showAdvanced)}
            className={styles.advancedToggle}
            disabled={isLoading}
          >
            ⚙️ Advanced Configuration {showAdvanced ? '▼' : '▶'}
          </button>

          {showAdvanced && (
            <div className={styles.advancedPanel}>
              <label className={styles.advancedLabel}>
                Temperature: {formData.parameters?.temperature?.toFixed(1)}
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  value={formData.parameters?.temperature || 0.7}
                  onChange={(e) => updateParameter('temperature', parseFloat(e.target.value))}
                  disabled={isLoading}
                />
              </label>

              <label className={styles.advancedLabel}>
                Max Tokens:
                <input
                  type="number"
                  value={formData.parameters?.max_tokens || 2000}
                  onChange={(e) => updateParameter('max_tokens', parseInt(e.target.value))}
                  className={styles.formInput}
                  disabled={isLoading}
                />
              </label>

              <label className={styles.advancedLabel}>
                Timeout (ms):
                <input
                  type="number"
                  value={formData.parameters?.timeout_ms || 300000}
                  onChange={(e) => updateParameter('timeout_ms', parseInt(e.target.value))}
                  className={styles.formInput}
                  disabled={isLoading}
                />
              </label>
            </div>
          )}
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="taskPriority" className={styles.formLabel}>
            Priority
          </label>
          <select
            id="taskPriority"
            value={formData.priority}
            onChange={(e) => handleInputChange('priority', e.target.value as TaskPriority)}
            className={styles.formSelect}
            disabled={isLoading}
          >
            {priorities.map(priority => (
              <option key={priority.value} value={priority.value}>
                {priority.label}
              </option>
            ))}
          </select>
        </div>

        <div className={styles.formCheckboxes}>
          <label className={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={formData.auto_start}
              onChange={(e) => handleInputChange('auto_start', e.target.checked)}
              disabled={isLoading}
            />
            <span className={styles.checkboxText}>Auto-start task immediately</span>
          </label>

          <label className={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={formData.user_approval_required}
              onChange={(e) => handleInputChange('user_approval_required', e.target.checked)}
              disabled={isLoading}
            />
            <span className={styles.checkboxText}>Require user approval before execution</span>
          </label>
        </div>

        {error && (
          <div className={styles.formError}>
            ⚠️ {error}
          </div>
        )}

        <div className={styles.formActions}>
          <button
            type="button"
            onClick={onCancel}
            className={styles.cancelButton}
            disabled={isLoading}
          >
            Cancel
          </button>
          <button
            type="submit"
            className={styles.submitButton}
            disabled={isLoading || !formData.name.trim() || !formData.description.trim()}
          >
            {isLoading ? '⏳ Creating...' : '✅ Create Task'}
          </button>
        </div>
      </form>
    </div>
  );
};

export default TaskCreationForm;