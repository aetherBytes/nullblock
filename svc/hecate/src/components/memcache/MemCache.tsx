import React, { useState, useEffect, useCallback } from 'react';
import { agentService } from '../../common/services/agent-service';
import type { Agent } from '../../types/agents';
import type { Engram, EngramType } from '../../types/engrams';
import type { Task, TaskState } from '../../types/tasks';
import MarkdownRenderer from '../common/MarkdownRenderer';
import TaskCreationForm from '../hud/TaskCreationForm';
import EngramsShelf from './EngramsShelf';
import { ArbFarmDashboard } from './arbfarm';
import type { ArbFarmView } from './arbfarm';
import { ConsensusView } from './consensus';
import { StashView } from './stash';
import { ContentView } from './content';
import styles from './memcache.module.scss';

export type MemCacheSection =
  | 'engrams'
  | 'stash'
  | 'agents'
  | 'model'
  | 'arbfarm'
  | 'consensus'
  | 'content';

type TaskCategory = 'todo' | 'running' | 'completed';

interface TaskManagement {
  tasks: Task[];
  isLoading: boolean;
  createTask: (request: any) => Promise<boolean>;
  startTask: (id: string) => Promise<boolean>;
  pauseTask: (id: string) => Promise<boolean>;
  resumeTask: (id: string) => Promise<boolean>;
  cancelTask: (id: string) => Promise<boolean>;
  retryTask: (id: string) => Promise<boolean>;
  processTask: (id: string) => Promise<boolean>;
  deleteTask: (id: string) => Promise<boolean>;
}

interface ModelManagement {
  isLoadingModelInfo: boolean;
  currentSelectedModel: string | null;
  availableModels: any[];
  showModelSelection: boolean;
  setShowModelSelection: (show: boolean) => void;
  handleModelSelection: (modelName: string) => void;
  loadAvailableModels: () => Promise<void>;
  getFreeModels: (models: any[], limit?: number) => any[];
  getFastModels: (models: any[], limit?: number) => any[];
  getThinkerModels: (models: any[], limit?: number) => any[];
  getImageModels: (models: any[], limit?: number) => any[];
}

interface MemCacheProps {
  publicKey: string | null;
  activeSection?: MemCacheSection;
  taskManagement?: TaskManagement;
  modelManagement?: ModelManagement;
  availableModels?: any[];
  activeAgent?: 'hecate' | 'siren';
  setActiveAgent?: (agent: 'hecate' | 'siren') => void;
  hasApiKey?: boolean;
}

// Task helpers
const getStatusIcon = (status: TaskState): string => {
  switch (status) {
    case 'working':
      return '⚡';
    case 'input-required':
      return '⏸️';
    case 'completed':
      return '✅';
    case 'failed':
      return '❌';
    case 'rejected':
      return '🚫';
    case 'canceled':
      return '🚫';
    case 'submitted':
      return '⏳';
    case 'auth-required':
      return '🔐';
    case 'unknown':
      return '❓';
    default:
      return '❓';
  }
};

const getStatusClass = (status: TaskState): string => {
  switch (status) {
    case 'working':
      return styles.statusWorking;
    case 'input-required':
      return styles.statusPaused;
    case 'completed':
      return styles.statusCompleted;
    case 'failed':
      return styles.statusFailed;
    case 'rejected':
      return styles.statusFailed;
    case 'canceled':
      return styles.statusCanceled;
    case 'submitted':
      return styles.statusSubmitted;
    case 'auth-required':
      return styles.statusPaused;
    case 'unknown':
      return styles.statusSubmitted;
    default:
      return styles.statusSubmitted;
  }
};

const getCategoryIcon = (category: string): string => {
  switch (category) {
    case 'user':
      return '👤 User';
    case 'agent':
      return '🤖 Agent';
    case 'api':
      return '🔗 API';
    case 'system':
      return '⚙️ System';
    case 'scheduled':
      return '⏰ Scheduled';
    case 'automated':
      return '🤖 Automated';
    case 'manual':
      return '👤 Manual';
    case 'webhook':
      return '🔗 Webhook';
    case 'cron':
      return '⏰ Cron';
    case 'user_assigned':
      return '👤 User';
    default:
      return `📋 ${category}`;
  }
};

const EREBUS_BASE_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

const MemCache: React.FC<MemCacheProps> = ({
  publicKey,
  activeSection = 'engrams',
  taskManagement,
  modelManagement,
  // @ts-ignore
  availableModels = [],
  // @ts-ignore
  activeAgent,
  setActiveAgent,
  hasApiKey = false,
}) => {
  // Engram state
  const [engrams, setEngrams] = useState<Engram[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedType, setSelectedType] = useState<EngramType | 'all'>('all');
  const [showCreateModal, setShowCreateModal] = useState(false);

  // Task state
  const [activeTaskCategory, setActiveTaskCategory] = useState<TaskCategory>('todo');
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  // @ts-ignore: task rendering reserved for future use
  const [showTaskForm, setShowTaskForm] = useState(false);

  // Agent state
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoadingAgents, setIsLoadingAgents] = useState(false);
  const [agentsError, setAgentsError] = useState<string | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  // Model state
  const [showModelList, setShowModelList] = useState(false);
  const [activeModelCategory, setActiveModelCategory] = useState<string | null>(null);

  // ArbFarm state
  const [arbFarmView, setArbFarmView] = useState<ArbFarmView>('dashboard');

  // Helper to check if a model is locked
  const isModelLocked = (model: any): boolean => {
    if (hasApiKey) {
      return false;
    }

    const isFree =
      model.tier === 'economical' ||
      model.cost_per_1k_tokens === 0 ||
      model.pricing?.prompt === '0' ||
      model.id?.includes(':free') ||
      model.name?.includes(':free');

    return !isFree;
  };

  // Fetch agents when agents section is selected
  useEffect(() => {
    if (activeSection === 'agents' && agents.length === 0) {
      fetchAgents();
    }
  }, [activeSection]);

  // Reset sub-states when section changes
  useEffect(() => {
    setSelectedTaskId(null);
    setShowTaskForm(false);
    setSelectedAgentId(null);
    setShowModelList(false);
    setArbFarmView('dashboard');
    setActiveModelCategory(null);
  }, [activeSection]);

  const fetchAgents = async () => {
    setIsLoadingAgents(true);
    setAgentsError(null);
    try {
      const response = await agentService.getAgents();

      if (response.success && response.data) {
        setAgents(response.data.agents);
      } else {
        setAgentsError(response.error || 'Failed to load agents');
      }
    } catch (error) {
      setAgentsError((error as Error).message);
    } finally {
      setIsLoadingAgents(false);
    }
  };

  const fetchEngrams = useCallback(async () => {
    if (!publicKey) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const url = `${EREBUS_BASE_URL}/api/engrams/wallet/${publicKey}`;
      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch engrams: ${response.status}`);
      }

      const data = await response.json();

      setEngrams(data.data || data || []);
    } catch (err) {
      console.error('Error fetching engrams:', err);
      setError(err instanceof Error ? err.message : 'Failed to load engrams');
    } finally {
      setIsLoading(false);
    }
  }, [publicKey]);

  useEffect(() => {
    fetchEngrams();
  }, [fetchEngrams]);

  const handleDeleteEngram = async (engramId: string) => {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/engrams/${engramId}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        throw new Error('Failed to delete engram');
      }

      setEngrams((prev) => prev.filter((e) => e.id !== engramId));
    } catch (err) {
      console.error('Error deleting engram:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete engram');
    }
  };

  const handleCreateEngram = async (newEngram: {
    engram_type: EngramType;
    key: string;
    content: string;
    metadata?: Record<string, unknown>;
  }) => {
    if (!publicKey) {
      return;
    }

    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/engrams`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: publicKey,
          ...newEngram,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to create engram');
      }

      const data = await response.json();

      setEngrams((prev) => [data.data || data, ...prev]);
      setShowCreateModal(false);
    } catch (err) {
      console.error('Error creating engram:', err);
      setError(err instanceof Error ? err.message : 'Failed to create engram');
    }
  };

  const filteredEngrams =
    selectedType === 'all' ? engrams : engrams.filter((e) => e.engram_type === selectedType);

  // ============================================
  // TASKS RENDER FUNCTIONS
  // ============================================

  // @ts-ignore: task rendering reserved for future use
  const _renderTaskList = () => {
    if (!taskManagement) {
      return (
        <div className={styles.emptyState}>
          <p>Task management not available</p>
        </div>
      );
    }

    const { tasks, isLoading: tasksLoading } = taskManagement;

    const categorizedTasks = {
      todo: tasks.filter((t) => t.status.state === 'submitted'),
      running: tasks.filter((t) =>
        ['working', 'input-required', 'auth-required'].includes(t.status.state),
      ),
      completed: tasks.filter((t) =>
        ['completed', 'failed', 'rejected', 'canceled'].includes(t.status.state),
      ),
    };

    const currentTasks = categorizedTasks[activeTaskCategory];

    return (
      <div className={styles.taskScope}>
        <div className={styles.taskTabs}>
          <button
            className={`${styles.taskTab} ${activeTaskCategory === 'todo' ? styles.activeTab : ''}`}
            onClick={() => setActiveTaskCategory('todo')}
          >
            To Do ({categorizedTasks.todo.length})
          </button>
          <button
            className={`${styles.taskTab} ${activeTaskCategory === 'running' ? styles.activeTab : ''}`}
            onClick={() => setActiveTaskCategory('running')}
          >
            Running ({categorizedTasks.running.length})
          </button>
          <button
            className={`${styles.taskTab} ${activeTaskCategory === 'completed' ? styles.activeTab : ''}`}
            onClick={() => setActiveTaskCategory('completed')}
          >
            Done ({categorizedTasks.completed.length})
          </button>
        </div>

        <button className={styles.createTaskButton} onClick={() => setShowTaskForm(true)}>
          + Create Task
        </button>

        {tasksLoading ? (
          <div className={styles.emptyState}>
            <p>Loading tasks...</p>
          </div>
        ) : currentTasks.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No {activeTaskCategory} tasks</p>
            <p className={styles.emptyHint}>
              {activeTaskCategory === 'todo' && 'Create a task to get started'}
              {activeTaskCategory === 'running' && 'Start a task to see it here'}
              {activeTaskCategory === 'completed' && 'Completed tasks will appear here'}
            </p>
          </div>
        ) : (
          <div className={styles.taskList}>
            {currentTasks.map((task) => (
              <div
                key={task.id}
                className={`${styles.taskItem} ${getStatusClass(task.status.state)}`}
                onClick={() => setSelectedTaskId(task.id)}
              >
                <div className={styles.taskItemHeader}>
                  <span className={styles.taskName}>
                    {task.status.message || task.name || 'Unnamed task'}
                  </span>
                  <span className={styles.taskStatus}>{getStatusIcon(task.status.state)}</span>
                </div>
                <div className={styles.taskItemMeta}>
                  <span className={styles.taskTime}>
                    {new Date(task.created_at).toLocaleDateString()}
                  </span>
                  <span className={styles.taskCategory}>{getCategoryIcon(task.category)}</span>
                </div>
                {activeTaskCategory === 'todo' && (
                  <div className={styles.taskItemActions}>
                    <button
                      className={styles.taskQuickAction}
                      onClick={(e) => {
                        e.stopPropagation();
                        taskManagement.startTask(task.id);
                      }}
                    >
                      ▶ Start
                    </button>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    );
  };

  // @ts-ignore: task rendering reserved for future use
  const _renderTaskDetails = () => {
    if (!taskManagement || !selectedTaskId) {
      return null;
    }

    const task = taskManagement.tasks.find((t) => t.id === selectedTaskId);

    if (!task) {
      return null;
    }

    return (
      <div className={styles.taskDetails}>
        <div className={styles.taskDetailsHeader}>
          <button className={styles.backButton} onClick={() => setSelectedTaskId(null)}>
            ← Back
          </button>
          <span className={styles.taskDetailsTitle}>
            {task.status.message || task.name || 'Task Details'}
          </span>
        </div>
        <div className={styles.taskDetailsBody}>
          <div className={styles.taskSection}>
            <h5>Status</h5>
            <div className={styles.taskField}>
              <label>State:</label>
              <span className={`${styles.statusBadge} ${getStatusClass(task.status.state)}`}>
                {getStatusIcon(task.status.state)} {task.status.state}
              </span>
            </div>
            <div className={styles.taskField}>
              <label>Category:</label>
              <span>{getCategoryIcon(task.category)}</span>
            </div>
            <div className={styles.taskField}>
              <label>Priority:</label>
              <span className={styles.priorityBadge}>{task.priority}</span>
            </div>
            {task.progress !== undefined && (
              <div className={styles.taskField}>
                <label>Progress:</label>
                <div className={styles.progressContainer}>
                  <div className={styles.progressBar}>
                    <div className={styles.progressFill} style={{ width: `${task.progress}%` }} />
                  </div>
                  <span>{task.progress}%</span>
                </div>
              </div>
            )}
          </div>

          <div className={styles.taskSection}>
            <h5>Timing</h5>
            <div className={styles.taskField}>
              <label>Created:</label>
              <span>{new Date(task.created_at).toLocaleString()}</span>
            </div>
            {task.started_at && (
              <div className={styles.taskField}>
                <label>Started:</label>
                <span>{new Date(task.started_at).toLocaleString()}</span>
              </div>
            )}
            {task.completed_at && (
              <div className={styles.taskField}>
                <label>Completed:</label>
                <span>{new Date(task.completed_at).toLocaleString()}</span>
              </div>
            )}
          </div>

          {task.artifacts && task.artifacts.length > 0 && (
            <div className={styles.taskSection}>
              <h5>Result</h5>
              <div className={styles.taskResultBox}>
                <MarkdownRenderer content={task.artifacts[0]?.parts?.[0]?.text || 'No result'} />
              </div>
            </div>
          )}

          <div className={styles.taskActions}>
            {task.status.state === 'submitted' && (
              <button
                className={styles.actionButton}
                onClick={() => taskManagement.startTask(task.id)}
              >
                ▶ Start
              </button>
            )}
            {task.status.state === 'working' && (
              <button
                className={styles.actionButton}
                onClick={() => taskManagement.pauseTask(task.id)}
              >
                ⏸ Pause
              </button>
            )}
            {task.status.state === 'input-required' && (
              <button
                className={styles.actionButton}
                onClick={() => taskManagement.resumeTask(task.id)}
              >
                ▶ Resume
              </button>
            )}
            {task.status.state === 'failed' && (
              <button
                className={styles.actionButton}
                onClick={() => taskManagement.retryTask(task.id)}
              >
                🔄 Retry
              </button>
            )}
            {!['completed', 'canceled', 'rejected'].includes(task.status.state) && (
              <button
                className={`${styles.actionButton} ${styles.dangerButton}`}
                onClick={() => taskManagement.cancelTask(task.id)}
              >
                ✕ Cancel
              </button>
            )}
            <button
              className={`${styles.actionButton} ${styles.dangerButton}`}
              onClick={() => {
                taskManagement.deleteTask(task.id);
                setSelectedTaskId(null);
              }}
            >
              🗑 Delete
            </button>
          </div>
        </div>
      </div>
    );
  };

  // @ts-ignore: task rendering reserved for future use
  const _renderTaskForm = () => {
    if (!taskManagement) {
      return null;
    }

    return (
      <div className={styles.taskFormWrapper}>
        <div className={styles.taskFormHeader}>
          <button className={styles.backButton} onClick={() => setShowTaskForm(false)}>
            ← Back
          </button>
          <span>Create New Task</span>
        </div>
        <TaskCreationForm
          onCreateTask={async (task: any) => {
            const success = await taskManagement.createTask(task);
            if (success) {
              setShowTaskForm(false);
            }
            return success;
          }}
          isLoading={taskManagement.isLoading}
          onCancel={() => setShowTaskForm(false)}
        />
      </div>
    );
  };

  // ============================================
  // AGENTS RENDER FUNCTIONS
  // ============================================

  const renderAgentList = () => {
    if (isLoadingAgents) {
      return (
        <div className={styles.emptyState}>
          <p>Loading agents...</p>
        </div>
      );
    }

    if (agentsError) {
      return (
        <div className={styles.emptyState}>
          <p>Error: {agentsError}</p>
          <button className={styles.retryButton} onClick={fetchAgents}>
            Retry
          </button>
        </div>
      );
    }

    return (
      <div className={styles.agentScope}>
        <div className={styles.agentHeader}>
          <span className={styles.agentCount}>
            {agents.length} agent{agents.length !== 1 ? 's' : ''} available
          </span>
          <button className={styles.refreshButton} onClick={fetchAgents}>
            🔄
          </button>
        </div>

        {agents.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No agents found</p>
            <p className={styles.emptyHint}>Agents will appear here when available</p>
          </div>
        ) : (
          <div className={styles.agentList}>
            {agents.map((agent) => (
              <div
                key={agent.name}
                className={styles.agentItem}
                onClick={() => setSelectedAgentId(agent.name)}
              >
                <div className={styles.agentItemHeader}>
                  <span className={styles.agentName}>{agent.name}</span>
                  <span
                    className={`${styles.agentItemStatus} ${agent.status === 'healthy' ? styles.healthy : styles.unhealthy}`}
                  >
                    {agent.status === 'healthy' ? '●' : '○'}
                  </span>
                </div>
                <div className={styles.agentItemType}>{agent.type || 'Agent'}</div>
                {agent.capabilities && (
                  <div className={styles.agentItemCapabilities}>
                    {agent.capabilities.slice(0, 3).map((_cap, i) => (
                      <span key={i} className={styles.capabilityTag}>
                        🔧
                      </span>
                    ))}
                    {agent.capabilities.length > 3 && (
                      <span className={styles.capabilityMore}>
                        +{agent.capabilities.length - 3}
                      </span>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    );
  };

  const renderAgentDetails = () => {
    const agent = agents.find((a) => a.name === selectedAgentId);

    if (!agent) {
      return null;
    }

    return (
      <div className={styles.agentDetails}>
        <div className={styles.agentDetailsHeader}>
          <button className={styles.backButton} onClick={() => setSelectedAgentId(null)}>
            ← Back
          </button>
          <span className={styles.agentDetailsTitle}>{agent.name}</span>
          {setActiveAgent && (
            <div className={styles.agentHeaderActions}>
              <button
                className={styles.agentChatButton}
                onClick={() => setActiveAgent(agent.name === 'siren' ? 'siren' : 'hecate')}
                disabled={agent.status !== 'healthy'}
              >
                Chat with {agent.name}
              </button>
            </div>
          )}
        </div>
        <div className={styles.agentDetailsBody}>
          <div className={styles.agentSection}>
            <h5>Status</h5>
            <div className={styles.agentField}>
              <label>Health:</label>
              <span
                className={`${styles.agentStatusBadge} ${agent.status === 'healthy' ? styles.healthy : styles.unhealthy}`}
              >
                {agent.status === 'healthy' ? '● Healthy' : '○ Unhealthy'}
              </span>
            </div>
            <div className={styles.agentField}>
              <label>Type:</label>
              <span>{agent.type || 'Agent'}</span>
            </div>
            {agent.endpoint && (
              <div className={styles.agentField}>
                <label>Endpoint:</label>
                <span className={styles.agentEndpoint}>{agent.endpoint}</span>
              </div>
            )}
          </div>

          {agent.description && (
            <div className={styles.agentSection}>
              <h5>Description</h5>
              <p className={styles.agentDescription}>{agent.description}</p>
            </div>
          )}

          {(agent as any).mission && (
            <div className={styles.agentSection}>
              <h5>Mission</h5>
              <p className={styles.agentMission}>{(agent as any).mission}</p>
            </div>
          )}

          {agent.capabilities && agent.capabilities.length > 0 && (
            <div className={styles.agentSection}>
              <h5>Capabilities</h5>
              <div className={styles.capabilitiesGrid}>
                {agent.capabilities.map((cap, i) => (
                  <span key={i} className={styles.capabilityItem}>
                    {cap}
                  </span>
                ))}
              </div>
            </div>
          )}

          {agent.metrics && (
            <div className={styles.agentSection}>
              <h5>Metrics</h5>
              <div className={styles.metricsGrid}>
                {Object.entries(agent.metrics).map(([key, value]) => (
                  <span key={key} className={styles.metricItem}>
                    {key}: {String(value)}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    );
  };

  // ============================================
  // MODEL RENDER FUNCTIONS
  // ============================================

  const renderModelInfo = () => {
    if (!modelManagement) {
      return (
        <div className={styles.emptyState}>
          <p>Model management not available</p>
        </div>
      );
    }

    const { isLoadingModelInfo, currentSelectedModel, availableModels: models } = modelManagement;

    const currentModel = models.find(
      (m: any) => m.id === currentSelectedModel || m.name === currentSelectedModel,
    );
    const modelName = currentModel?.name || currentSelectedModel || 'No model selected';
    const provider = currentModel?.owned_by || currentModel?.provider || 'Unknown';

    const freeModels = modelManagement.getFreeModels(models);
    const fastModels = modelManagement.getFastModels(models);
    const thinkerModels = modelManagement.getThinkerModels(models);
    const imageModels = modelManagement.getImageModels(models);

    return (
      <div className={styles.modelScope}>
        <div className={styles.modelSection}>
          <h5>Current Model</h5>
          <div className={styles.modelCurrent}>
            <div className={styles.modelCurrentHeader}>
              <span className={styles.modelIcon}>🤖</span>
              <div className={styles.modelCurrentInfo}>
                <span className={styles.modelCurrentLabel}>Active Model</span>
                <span className={styles.modelCurrentName}>{modelName}</span>
                <span className={styles.modelCurrentProvider}>{provider}</span>
              </div>
              <button className={styles.switchModelButton} onClick={() => setShowModelList(true)}>
                Switch
              </button>
            </div>

            <div className={styles.modelCategories}>
              <button
                className={`${styles.modelCategoryButton} ${activeModelCategory === 'free' ? styles.activeCategory : ''}`}
                onClick={() => {
                  setActiveModelCategory('free');
                  setShowModelList(true);
                }}
              >
                Free <span className={styles.categoryCount}>{freeModels.length}</span>
              </button>
              <button
                className={`${styles.modelCategoryButton} ${activeModelCategory === 'fast' ? styles.activeCategory : ''} ${!hasApiKey ? styles.categoryLocked : ''}`}
                onClick={() => {
                  setActiveModelCategory('fast');
                  setShowModelList(true);
                }}
              >
                Fast <span className={styles.categoryCount}>{fastModels.length}</span>
                {!hasApiKey && <span className={styles.lockBadge}>🔒</span>}
              </button>
              <button
                className={`${styles.modelCategoryButton} ${activeModelCategory === 'thinker' ? styles.activeCategory : ''} ${!hasApiKey ? styles.categoryLocked : ''}`}
                onClick={() => {
                  setActiveModelCategory('thinker');
                  setShowModelList(true);
                }}
              >
                Thinker <span className={styles.categoryCount}>{thinkerModels.length}</span>
                {!hasApiKey && <span className={styles.lockBadge}>🔒</span>}
              </button>
              <button
                className={`${styles.modelCategoryButton} ${activeModelCategory === 'image' ? styles.activeCategory : ''} ${!hasApiKey ? styles.categoryLocked : ''}`}
                onClick={() => {
                  setActiveModelCategory('image');
                  setShowModelList(true);
                }}
              >
                Image <span className={styles.categoryCount}>{imageModels.length}</span>
                {!hasApiKey && <span className={styles.lockBadge}>🔒</span>}
              </button>
            </div>

            <div className={styles.modelStats}>
              <div className={styles.modelStat}>
                Total: <span>{models.length}</span>
              </div>
              {currentModel?.tier && (
                <div
                  className={`${styles.tierBadge} ${currentModel.tier === 'economical' ? styles.tierFree : styles.tierPaid}`}
                >
                  {currentModel.tier === 'economical' ? 'Free' : 'Paid'}
                </div>
              )}
            </div>
          </div>
        </div>

        {isLoadingModelInfo && (
          <div className={styles.modelLoading}>
            <div className={styles.loadingSpinner}></div>
            <span>Loading model info...</span>
          </div>
        )}

        <button
          className={styles.refreshModelsButton}
          onClick={() => modelManagement.loadAvailableModels()}
        >
          🔄 Refresh Models
        </button>
      </div>
    );
  };

  const renderModelSelection = () => {
    if (!modelManagement) {
      return null;
    }

    const { availableModels: models, currentSelectedModel, handleModelSelection } = modelManagement;

    let filteredModels = models;

    if (activeModelCategory === 'free') {
      filteredModels = modelManagement.getFreeModels(models);
    } else if (activeModelCategory === 'fast') {
      filteredModels = modelManagement.getFastModels(models);
    } else if (activeModelCategory === 'thinker') {
      filteredModels = modelManagement.getThinkerModels(models);
    } else if (activeModelCategory === 'image') {
      filteredModels = modelManagement.getImageModels(models);
    }

    return (
      <div className={styles.modelSelection}>
        <div className={styles.modelSelectionHeader}>
          <button
            className={styles.backButton}
            onClick={() => {
              setShowModelList(false);
              setActiveModelCategory(null);
            }}
          >
            ← Back
          </button>
          <span className={styles.modelSelectionTitle}>
            {activeModelCategory
              ? `${activeModelCategory.charAt(0).toUpperCase() + activeModelCategory.slice(1)} Models`
              : 'Select Model'}
          </span>
        </div>

        {!hasApiKey && activeModelCategory !== 'free' && (
          <div className={styles.apiKeyWarning}>
            <span className={styles.warningIcon}>⚠️</span>
            <span>Add an API key in Settings to unlock paid models</span>
          </div>
        )}

        <div className={styles.modelCategoryTabs}>
          <button
            className={`${styles.modelCategoryTab} ${activeModelCategory === null ? styles.activeTab : ''}`}
            onClick={() => setActiveModelCategory(null)}
          >
            All
          </button>
          <button
            className={`${styles.modelCategoryTab} ${activeModelCategory === 'free' ? styles.activeTab : ''}`}
            onClick={() => setActiveModelCategory('free')}
          >
            Free
          </button>
          <button
            className={`${styles.modelCategoryTab} ${activeModelCategory === 'fast' ? styles.activeTab : ''}`}
            onClick={() => setActiveModelCategory('fast')}
          >
            Fast
          </button>
          <button
            className={`${styles.modelCategoryTab} ${activeModelCategory === 'thinker' ? styles.activeTab : ''}`}
            onClick={() => setActiveModelCategory('thinker')}
          >
            Thinker
          </button>
          <button
            className={`${styles.modelCategoryTab} ${activeModelCategory === 'image' ? styles.activeTab : ''}`}
            onClick={() => setActiveModelCategory('image')}
          >
            Image
          </button>
        </div>

        <div className={styles.scopeContent}>
          {filteredModels.length === 0 ? (
            <div className={styles.modelListEmpty}>
              <p>No models in this category</p>
              <p className={styles.noModelsHint}>Try a different category</p>
            </div>
          ) : (
            <div className={styles.modelList}>
              {filteredModels.map((model: any) => {
                const isSelected =
                  model.id === currentSelectedModel || model.name === currentSelectedModel;
                const locked = isModelLocked(model);

                return (
                  <div
                    key={model.id || model.name}
                    className={`${styles.modelItem} ${isSelected ? styles.selectedModel : ''} ${locked ? styles.modelLocked : ''}`}
                    onClick={() => !locked && handleModelSelection(model.id || model.name)}
                  >
                    <span className={styles.modelItemIcon}>
                      {locked ? <span className={styles.lockedIcon}>🔒</span> : '🤖'}
                    </span>
                    <div className={styles.modelItemInfo}>
                      <span className={styles.modelItemName}>{model.name || model.id}</span>
                      <span className={styles.modelItemProvider}>
                        {model.owned_by || model.provider || 'Unknown'}
                      </span>
                    </div>
                    <div className={styles.modelItemMeta}>
                      <span
                        className={`${styles.modelTier} ${model.tier === 'economical' ? styles.free : styles.paid}`}
                      >
                        {model.tier === 'economical' ? 'Free' : 'Paid'}
                      </span>
                      {isSelected && <span className={styles.selectedBadge}>✓ Active</span>}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    );
  };

  // ============================================
  // MAIN RENDER SECTION CONTENT
  // ============================================

  const renderSectionContent = () => {
    switch (activeSection) {
      case 'engrams':
        return (
          <>
            <div className={styles.memcacheHeader}>
              <div className={styles.headerLeft}>
                <h1 className={styles.title}>Engrams</h1>
                <p className={styles.tagline}>Your memories persist. The void remembers.</p>
              </div>
            </div>

            <button className={styles.createButtonFixed} onClick={() => setShowCreateModal(true)}>
              + New Engram
            </button>

            <div className={styles.filterBar}>
              <button
                className={`${styles.filterChip} ${selectedType === 'all' ? styles.active : ''}`}
                onClick={() => setSelectedType('all')}
              >
                All ({engrams.length})
              </button>
              {(
                ['persona', 'preference', 'strategy', 'knowledge', 'compliance'] as EngramType[]
              ).map((type) => {
                const count = engrams.filter((e) => e.engram_type === type).length;

                return (
                  <button
                    key={type}
                    className={`${styles.filterChip} ${styles[type]} ${selectedType === type ? styles.active : ''}`}
                    onClick={() => setSelectedType(type)}
                  >
                    {type.charAt(0).toUpperCase() + type.slice(1)} ({count})
                  </button>
                );
              })}
            </div>

            {error && (
              <div className={styles.errorBanner}>
                <span>{error}</span>
                <button onClick={() => setError(null)}>Dismiss</button>
              </div>
            )}

            <EngramsShelf
              engrams={filteredEngrams}
              isLoading={isLoading}
              onDelete={handleDeleteEngram}
              onRefresh={fetchEngrams}
            />
          </>
        );
      case 'stash':
        return <StashView walletAddress={publicKey} />;
      case 'agents':
        if (selectedAgentId) {
          return renderAgentDetails();
        }

        return renderAgentList();
      case 'model':
        if (showModelList) {
          return renderModelSelection();
        }

        return renderModelInfo();
      case 'arbfarm':
        return <ArbFarmDashboard activeView={arbFarmView} onViewChange={setArbFarmView} />;
      case 'consensus':
        return <ConsensusView />;
      case 'content':
        return <ContentView />;
      default:
        return null;
    }
  };

  if (!publicKey) {
    return (
      <div className={styles.memcacheContainer}>
        <div className={styles.disconnectedState}>
          <div className={styles.disconnectedIcon}>🧠</div>
          <h2>The Mem Cache</h2>
          <p className={styles.tagline}>Your memories persist. The void remembers.</p>
          <p className={styles.connectPrompt}>Connect your wallet to access your engrams</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.memcacheLayout}>
      <main className={styles.memcacheMain}>
        <div className={styles.memcacheContentWrapper}>{renderSectionContent()}</div>
      </main>

      {showCreateModal && (
        <CreateEngramModal
          onClose={() => setShowCreateModal(false)}
          onCreate={handleCreateEngram}
        />
      )}
    </div>
  );
};

interface CreateEngramModalProps {
  onClose: () => void;
  onCreate: (engram: {
    engram_type: EngramType;
    key: string;
    content: string;
    metadata?: Record<string, unknown>;
  }) => void;
}

const CreateEngramModal: React.FC<CreateEngramModalProps> = ({ onClose, onCreate }) => {
  const [engramType, setEngramType] = useState<EngramType>('knowledge');
  const [key, setKey] = useState('');
  const [content, setContent] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!key.trim() || !content.trim()) {
      return;
    }

    onCreate({ engram_type: engramType, key: key.trim(), content: content.trim() });
  };

  return (
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.modalHeader}>
          <h2>Create New Engram</h2>
          <button className={styles.closeButton} onClick={onClose}>
            ×
          </button>
        </div>
        <form onSubmit={handleSubmit} className={styles.modalForm}>
          <div className={styles.formGroup}>
            <label>Type</label>
            <select
              value={engramType}
              onChange={(e) => setEngramType(e.target.value as EngramType)}
            >
              <option value="persona">Persona</option>
              <option value="preference">Preference</option>
              <option value="strategy">Strategy</option>
              <option value="knowledge">Knowledge</option>
              <option value="compliance">Compliance</option>
            </select>
          </div>
          <div className={styles.formGroup}>
            <label>Key</label>
            <input
              type="text"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder="e.g., trading_style, risk_tolerance"
            />
          </div>
          <div className={styles.formGroup}>
            <label>Content</label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Enter the engram content..."
              rows={5}
            />
          </div>
          <div className={styles.modalActions}>
            <button type="button" className={styles.cancelButton} onClick={onClose}>
              Cancel
            </button>
            <button
              type="submit"
              className={styles.submitButton}
              disabled={!key.trim() || !content.trim()}
            >
              Create Engram
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default MemCache;
