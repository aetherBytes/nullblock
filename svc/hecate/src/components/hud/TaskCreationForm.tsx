import React, { useState } from 'react';
import { TaskCreationRequest, TaskType, TaskPriority, TaskCategory } from '../../types/tasks';
import styles from './hud.module.scss';

interface TaskCreationFormProps {
  onCreateTask: (request: TaskCreationRequest) => Promise<boolean>;
  isLoading: boolean;
  onCancel: () => void;
  variant?: 'default' | 'embedded' | 'fullscreen';
}

const TaskCreationForm: React.FC<TaskCreationFormProps> = ({
  onCreateTask,
  isLoading,
  onCancel,
  variant = 'default'
}) => {
  const [formData, setFormData] = useState<TaskCreationRequest>({
    name: '',
    description: '',
    task_type: 'system',
    priority: 'medium',
    category: 'user_assigned',
    auto_start: false,
    user_approval_required: false,
    parameters: {},
    dependencies: []
  });

  const [error, setError] = useState<string | null>(null);

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

    try {
      const success = await onCreateTask(formData);
      if (success) {
        onCancel();
        setFormData({
          name: '',
          description: '',
          task_type: 'system',
          priority: 'medium',
          category: 'user_assigned',
          auto_start: false,
          user_approval_required: false,
          parameters: {},
          dependencies: []
        });
      }
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const handleInputChange = (field: keyof TaskCreationRequest, value: any) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }));
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