import React, { useState, useRef, useEffect } from 'react';
import { Task, TaskState } from '../../../types/tasks';
import { Agent } from '../../../types/agents';
import { agentService } from '../../../common/services/agent-service';
import TaskCreationForm from '../../hud/TaskCreationForm';
import MarkdownRenderer from '../../common/MarkdownRenderer';
import styles from './voidScopes.module.scss';

type ScopeType = 'tasks' | 'agents' | 'model-info' | null;
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

interface VoidScopesProps {
  isActive?: boolean;
  isOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  taskManagement?: TaskManagement;
  modelManagement?: ModelManagement;
  availableModels?: any[];
  activeAgent?: 'hecate' | 'siren';
  setActiveAgent?: (agent: 'hecate' | 'siren') => void;
  hasApiKey?: boolean;
}

const SCOPE_OPTIONS: { id: ScopeType; label: string; icon: string }[] = [
  { id: 'tasks', label: 'Tasks', icon: '◈' },
  { id: 'agents', label: 'Agents', icon: '◉' },
  { id: 'model-info', label: 'Model Info', icon: '◎' },
];

// Task helpers
const getStatusIcon = (status: TaskState): string => {
  switch (status) {
    case 'working': return '⚡';
    case 'input-required': return '⏸️';
    case 'completed': return '✅';
    case 'failed': return '❌';
    case 'rejected': return '🚫';
    case 'canceled': return '🚫';
    case 'submitted': return '⏳';
    case 'auth-required': return '🔐';
    case 'unknown': return '❓';
    default: return '❓';
  }
};

const getStatusClass = (status: TaskState): string => {
  switch (status) {
    case 'working': return styles.statusWorking;
    case 'input-required': return styles.statusPaused;
    case 'completed': return styles.statusCompleted;
    case 'failed': return styles.statusFailed;
    case 'rejected': return styles.statusFailed;
    case 'canceled': return styles.statusCanceled;
    case 'submitted': return styles.statusSubmitted;
    case 'auth-required': return styles.statusPaused;
    case 'unknown': return styles.statusSubmitted;
    default: return styles.statusSubmitted;
  }
};

const getCategoryIcon = (category: string): string => {
  switch (category) {
    case 'user': return '👤 User';
    case 'agent': return '🤖 Agent';
    case 'api': return '🔗 API';
    case 'system': return '⚙️ System';
    case 'scheduled': return '⏰ Scheduled';
    case 'automated': return '🤖 Automated';
    case 'manual': return '👤 Manual';
    case 'webhook': return '🔗 Webhook';
    case 'cron': return '⏰ Cron';
    case 'user_assigned': return '👤 User';
    default: return `📋 ${category}`;
  }
};

const VoidScopes: React.FC<VoidScopesProps> = ({
  isActive = true,
  isOpen = false,
  onOpenChange,
  taskManagement,
  modelManagement,
  availableModels = [],
  activeAgent,
  setActiveAgent,
  hasApiKey = false,
}) => {
  // Helper to check if a model is locked (non-free models when user has no API key)
  const isModelLocked = (model: any): boolean => {
    if (hasApiKey) return false; // User has API key, nothing locked
    // Check if model is free
    const isFree = model.tier === 'economical' ||
                   model.cost_per_1k_tokens === 0 ||
                   model.pricing?.prompt === '0' ||
                   model.id?.includes(':free') ||
                   model.name?.includes(':free');
    return !isFree;
  };
  const [selectedScope, setSelectedScope] = useState<ScopeType>(null);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Task-specific state
  const [activeTaskCategory, setActiveTaskCategory] = useState<TaskCategory>('todo');
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [showTaskForm, setShowTaskForm] = useState(false);

  // Agent-specific state
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoadingAgents, setIsLoadingAgents] = useState(false);
  const [agentsError, setAgentsError] = useState<string | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  // Fetch agents when agents scope is selected
  useEffect(() => {
    if (selectedScope === 'agents' && agents.length === 0) {
      fetchAgents();
    }
  }, [selectedScope]);

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

  // Sync with external open state
  useEffect(() => {
    if (isOpen && !selectedScope) {
      setIsDropdownOpen(true);
    }
  }, [isOpen, selectedScope]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsDropdownOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleScopeSelect = (scopeId: ScopeType) => {
    setSelectedScope(scopeId);
    setIsDropdownOpen(false);
    onOpenChange?.(true);
  };

  const handleToggleDropdown = () => {
    // Toggle dropdown to show/hide scope options
    // Keep selected scope open - only close via X button or by selecting another scope
    setIsDropdownOpen(!isDropdownOpen);
  };

  const handleClose = () => {
    setSelectedScope(null);
    setIsDropdownOpen(false);
    setSelectedTaskId(null);
    setShowTaskForm(false);
    setSelectedAgentId(null);
    onOpenChange?.(false);
  };

  const getSelectedLabel = () => {
    if (!selectedScope) return 'SCOPES';
    const scope = SCOPE_OPTIONS.find(s => s.id === selectedScope);
    return scope ? scope.label.toUpperCase() : 'SCOPES';
  };

  // ============================================
  // TASKS SCOPE
  // ============================================

  const tasks = taskManagement?.tasks || [];
  const todoTasks = tasks.filter(task => task.status.state === 'submitted');
  const runningTasks = tasks.filter(task =>
    task.status.state === 'working' || task.status.state === 'input-required'
  );
  const completedTasks = tasks.filter(task =>
    task.status.state === 'completed' ||
    task.status.state === 'failed' ||
    task.status.state === 'canceled'
  );

  const getTasksForCategory = (category: TaskCategory): Task[] => {
    switch (category) {
      case 'todo': return todoTasks;
      case 'running': return runningTasks;
      case 'completed': return completedTasks;
      default: return [];
    }
  };

  const currentCategoryTasks = getTasksForCategory(activeTaskCategory);
  const selectedTask = selectedTaskId ? tasks.find(t => t.id === selectedTaskId) : null;

  const renderTaskDetails = () => {
    if (!selectedTask || !taskManagement) return null;

    return (
      <div className={styles.taskDetails}>
        <div className={styles.taskDetailsHeader}>
          <button onClick={() => setSelectedTaskId(null)} className={styles.backButton}>
            ← Back
          </button>
          <span className={styles.taskDetailsTitle}>Task Details</span>
        </div>

        <div className={styles.taskDetailsBody}>
          <div className={styles.taskSection}>
            <h5>Basic Information</h5>
            <div className={styles.taskField}>
              <label>Name:</label>
              <span>{selectedTask.name}</span>
            </div>
            <div className={styles.taskField}>
              <label>Type:</label>
              <span>{selectedTask.task_type}</span>
            </div>
            <div className={styles.taskField}>
              <label>Status:</label>
              <span className={`${styles.statusBadge} ${getStatusClass(selectedTask.status.state)}`}>
                {getStatusIcon(selectedTask.status.state)} {selectedTask.status.state}
              </span>
            </div>
            <div className={styles.taskField}>
              <label>Priority:</label>
              <span className={styles.priorityBadge}>{selectedTask.priority}</span>
            </div>
            <div className={styles.taskFieldFull}>
              <label>Description:</label>
              <p>{selectedTask.description}</p>
            </div>
          </div>

          <div className={styles.taskSection}>
            <h5>Execution</h5>
            <div className={styles.taskField}>
              <label>Progress:</label>
              <div className={styles.progressContainer}>
                <div className={styles.progressBar}>
                  <div
                    className={styles.progressFill}
                    style={{ width: `${selectedTask.status.state === 'completed' ? 100 : selectedTask.progress}%` }}
                  />
                </div>
                <span>{selectedTask.status.state === 'completed' ? '100' : Math.round(selectedTask.progress)}%</span>
              </div>
            </div>
            {selectedTask.action_duration && (
              <div className={styles.taskField}>
                <label>Duration:</label>
                <span>{(selectedTask.action_duration / 1000).toFixed(2)}s</span>
              </div>
            )}
            <div className={styles.taskField}>
              <label>Created:</label>
              <span>{new Date(selectedTask.created_at).toLocaleString()}</span>
            </div>
          </div>

          {selectedTask.action_result && (
            <div className={styles.taskSection}>
              <h5>Result</h5>
              <div className={styles.taskResultBox}>
                <MarkdownRenderer content={selectedTask.action_result} />
              </div>
            </div>
          )}

          <div className={styles.taskActions}>
            {selectedTask.status.state === 'submitted' && (
              <>
                <button onClick={() => { taskManagement.startTask(selectedTask.id); setSelectedTaskId(null); }} className={styles.actionButton}>
                  ▶️ Start
                </button>
                <button onClick={() => { taskManagement.processTask(selectedTask.id); setSelectedTaskId(null); }} className={styles.actionButton}>
                  ⚡ Process
                </button>
              </>
            )}
            {selectedTask.status.state === 'working' && (
              <>
                <button onClick={() => { taskManagement.pauseTask(selectedTask.id); setSelectedTaskId(null); }} className={styles.actionButton}>
                  ⏸️ Pause
                </button>
                <button onClick={() => { taskManagement.cancelTask(selectedTask.id); setSelectedTaskId(null); }} className={styles.actionButton}>
                  🚫 Cancel
                </button>
              </>
            )}
            {selectedTask.status.state === 'failed' && (
              <button onClick={() => { taskManagement.retryTask(selectedTask.id); setSelectedTaskId(null); }} className={styles.actionButton}>
                🔄 Retry
              </button>
            )}
            <button onClick={() => { taskManagement.deleteTask(selectedTask.id); setSelectedTaskId(null); }} className={`${styles.actionButton} ${styles.dangerButton}`}>
              🗑️ Delete
            </button>
          </div>
        </div>
      </div>
    );
  };

  const renderTaskList = () => {
    if (!taskManagement) {
      return (
        <div className={styles.scopePlaceholder}>
          <span className={styles.placeholderIcon}>◈</span>
          <span>Task management not available</span>
        </div>
      );
    }

    return (
      <div className={styles.taskScope}>
        <div className={styles.taskTabs}>
          <button className={`${styles.taskTab} ${activeTaskCategory === 'todo' ? styles.activeTab : ''}`} onClick={() => setActiveTaskCategory('todo')}>
            ⏳ Todo ({todoTasks.length})
          </button>
          <button className={`${styles.taskTab} ${activeTaskCategory === 'running' ? styles.activeTab : ''}`} onClick={() => setActiveTaskCategory('running')}>
            ⚡ Running ({runningTasks.length})
          </button>
          <button className={`${styles.taskTab} ${activeTaskCategory === 'completed' ? styles.activeTab : ''}`} onClick={() => setActiveTaskCategory('completed')}>
            ✅ Done ({completedTasks.length})
          </button>
        </div>

        <button onClick={() => setShowTaskForm(true)} className={styles.createTaskButton}>
          ➕ Create Task
        </button>

        <div className={styles.taskList}>
          {currentCategoryTasks.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No {activeTaskCategory} tasks</p>
              {activeTaskCategory === 'todo' && <p className={styles.emptyHint}>Create a task to get started</p>}
            </div>
          ) : (
            currentCategoryTasks.map((task) => (
              <div key={task.id} className={`${styles.taskItem} ${getStatusClass(task.status.state)}`} onClick={() => setSelectedTaskId(task.id)}>
                <div className={styles.taskItemHeader}>
                  <span className={styles.taskName}>{task.name}</span>
                  <span className={styles.taskStatus}>{getStatusIcon(task.status.state)}</span>
                </div>
                <div className={styles.taskItemMeta}>
                  <span className={styles.taskTime}>{new Date(task.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                  <span className={styles.taskCategory}>{getCategoryIcon(task.category)}</span>
                </div>
                <div className={styles.taskItemActions}>
                  {task.status.state === 'submitted' && (
                    <>
                      <button onClick={(e) => { e.stopPropagation(); taskManagement.startTask(task.id); }} className={styles.taskQuickAction} title="Start">▶️</button>
                      <button onClick={(e) => { e.stopPropagation(); taskManagement.processTask(task.id); }} className={styles.taskQuickAction} title="Process">⚡</button>
                    </>
                  )}
                  {task.status.state === 'working' && (
                    <button onClick={(e) => { e.stopPropagation(); taskManagement.cancelTask(task.id); }} className={styles.taskQuickAction} title="Cancel">❌</button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    );
  };

  const renderTaskForm = () => {
    if (!taskManagement) return null;

    return (
      <div className={styles.taskFormWrapper}>
        <div className={styles.taskFormHeader}>
          <button onClick={() => setShowTaskForm(false)} className={styles.backButton}>← Back</button>
          <span>Create Task</span>
        </div>
        <TaskCreationForm
          onCreateTask={async (request) => {
            const success = await taskManagement.createTask(request);
            if (success) setShowTaskForm(false);
            return success;
          }}
          isLoading={taskManagement.isLoading}
          onCancel={() => setShowTaskForm(false)}
          variant="embedded"
          availableModels={availableModels}
        />
      </div>
    );
  };

  // ============================================
  // AGENTS SCOPE
  // ============================================

  const selectedAgent = selectedAgentId ? agents.find(a => a.name === selectedAgentId) : null;

  const renderAgentDetails = () => {
    if (!selectedAgent) return null;

    const isOnline = agentService.isAgentOnline(selectedAgent);

    return (
      <div className={styles.agentDetails}>
        <div className={styles.agentDetailsHeader}>
          <button onClick={() => setSelectedAgentId(null)} className={styles.backButton}>
            ← Back
          </button>
          <span className={styles.agentDetailsTitle}>
            {selectedAgent.name.charAt(0).toUpperCase() + selectedAgent.name.slice(1)}
          </span>
          <div className={styles.agentHeaderActions}>
            <button
              onClick={() => {
                if (setActiveAgent && (selectedAgent.name === 'hecate' || selectedAgent.name === 'siren')) {
                  setActiveAgent(selectedAgent.name);
                }
              }}
              className={styles.agentChatButton}
              disabled={!isOnline}
            >
              💬 Chat
            </button>
          </div>
        </div>

        <div className={styles.agentDetailsBody}>
          <div className={styles.agentSection}>
            <h5>Status</h5>
            <div className={styles.agentField}>
              <label>Type:</label>
              <span>{selectedAgent.type}</span>
            </div>
            <div className={styles.agentField}>
              <label>Status:</label>
              <span className={`${styles.agentStatusBadge} ${selectedAgent.status === 'healthy' ? styles.healthy : styles.unhealthy}`}>
                {selectedAgent.status === 'healthy' ? '✅' : '❌'} {selectedAgent.status}
              </span>
            </div>
            <div className={styles.agentField}>
              <label>Endpoint:</label>
              <span className={styles.agentEndpoint}>{selectedAgent.endpoint}</span>
            </div>
          </div>

          <div className={styles.agentSection}>
            <h5>Description</h5>
            <p className={styles.agentDescription}>{selectedAgent.description}</p>
          </div>

          {/* Agent-specific sections */}
          {selectedAgent.name === 'hecate' && (
            <>
              <div className={styles.agentSection}>
                <h5>Core Mission</h5>
                <p className={styles.agentMission}>
                  Hecate serves as NullBlock's neural core and primary conversational interface.
                  As the orchestration engine, Hecate coordinates specialized agents for blockchain operations,
                  DeFi analysis, market intelligence, and complex workflow management.
                </p>
              </div>
              <div className={styles.agentSection}>
                <h5>Key Capabilities</h5>
                <div className={styles.capabilitiesGrid}>
                  <div className={styles.capabilityItem}>🤖 Multi-Agent Orchestration</div>
                  <div className={styles.capabilityItem}>💬 Conversational Interface</div>
                  <div className={styles.capabilityItem}>🔍 Intent Analysis</div>
                  <div className={styles.capabilityItem}>📋 Task Management</div>
                  <div className={styles.capabilityItem}>🧠 LLM Coordination</div>
                </div>
              </div>
            </>
          )}

          {selectedAgent.name === 'siren' && (
            <>
              <div className={styles.agentSection}>
                <h5>Core Mission</h5>
                <p className={styles.agentMission}>
                  Siren serves as NullBlock's frontline evangelist in the decentralized arena,
                  driving go-to-market strategies, tokenomics storytelling, and viral outreach
                  to amplify adoption across blockchain networks.
                </p>
              </div>
              <div className={styles.agentSection}>
                <h5>Key Capabilities</h5>
                <div className={styles.capabilitiesGrid}>
                  <div className={styles.capabilityItem}>📝 Campaign Design</div>
                  <div className={styles.capabilityItem}>💰 Tokenomics Narrative</div>
                  <div className={styles.capabilityItem}>📊 Sentiment Analysis</div>
                  <div className={styles.capabilityItem}>🤝 Partnership Brokering</div>
                  <div className={styles.capabilityItem}>📱 Social Media</div>
                </div>
              </div>
            </>
          )}

          <div className={styles.agentSection}>
            <h5>Capabilities</h5>
            <div className={styles.capabilitiesGrid}>
              {selectedAgent.capabilities.map((capability, index) => (
                <div key={index} className={styles.capabilityItem}>
                  {agentService.getCapabilityIcon(capability)} {capability.replace(/_/g, ' ')}
                </div>
              ))}
            </div>
          </div>

          {selectedAgent.metrics && (
            <div className={styles.agentSection}>
              <h5>Metrics</h5>
              <div className={styles.metricsGrid}>
                {agentService.getAgentMetrics(selectedAgent).map((metric, index) => (
                  <div key={index} className={styles.metricItem}>{metric}</div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    );
  };

  const renderAgentList = () => {
    if (isLoadingAgents) {
      return (
        <div className={styles.scopePlaceholder}>
          <span className={styles.placeholderIcon}>◉</span>
          <span>Loading agents...</span>
        </div>
      );
    }

    if (agentsError) {
      return (
        <div className={styles.scopePlaceholder}>
          <span className={styles.placeholderIcon}>❌</span>
          <span>{agentsError}</span>
          <button onClick={fetchAgents} className={styles.retryButton}>Retry</button>
        </div>
      );
    }

    return (
      <div className={styles.agentScope}>
        <div className={styles.agentHeader}>
          <span className={styles.agentCount}>🤖 Active Agents ({agents.length})</span>
          <button onClick={fetchAgents} className={styles.refreshButton} title="Refresh">
            🔄
          </button>
        </div>

        <div className={styles.agentList}>
          {agents.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No agents found</p>
              <p className={styles.emptyHint}>Check that the agents service is running</p>
            </div>
          ) : (
            agents.map((agent) => (
              <div key={agent.name} className={styles.agentItem} onClick={() => setSelectedAgentId(agent.name)}>
                <div className={styles.agentItemHeader}>
                  <span className={styles.agentName}>
                    {agent.name.charAt(0).toUpperCase() + agent.name.slice(1)}
                  </span>
                  <span className={`${styles.agentItemStatus} ${agent.status === 'healthy' ? styles.healthy : styles.unhealthy}`}>
                    {agent.status === 'healthy' ? '✅' : '❌'}
                  </span>
                </div>
                <div className={styles.agentItemType}>{agent.type}</div>
                <div className={styles.agentItemCapabilities}>
                  {agent.capabilities.slice(0, 3).map((cap, i) => (
                    <span key={i} className={styles.capabilityTag}>
                      {agentService.getCapabilityIcon(cap)}
                    </span>
                  ))}
                  {agent.capabilities.length > 3 && (
                    <span className={styles.capabilityMore}>+{agent.capabilities.length - 3}</span>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    );
  };

  // ============================================
  // MODEL INFO SCOPE
  // ============================================

  const [showModelList, setShowModelList] = useState(false);
  const [activeModelCategory, setActiveModelCategory] = useState<string | null>(null);

  const renderModelInfo = () => {
    if (!modelManagement) {
      return (
        <div className={styles.scopePlaceholder}>
          <span className={styles.placeholderIcon}>◎</span>
          <span>Model management not available</span>
        </div>
      );
    }

    const { isLoadingModelInfo, currentSelectedModel, availableModels: models } = modelManagement;

    // Find current model info from available models
    const currentModelInfo = models.find(m => m.name === currentSelectedModel);

    if (isLoadingModelInfo) {
      return (
        <div className={styles.scopePlaceholder}>
          <span className={styles.placeholderIcon}>◎</span>
          <span>Loading model info...</span>
        </div>
      );
    }

    if (showModelList) {
      return renderModelSelection();
    }

    return (
      <div className={styles.modelScope}>
        {/* API Key Warning */}
        {!hasApiKey && (
          <div className={styles.apiKeyWarning}>
            <span className={styles.warningIcon}>🔒</span>
            <span>Add an API key in Settings to unlock all models</span>
          </div>
        )}

        {/* Current Model */}
        <div className={styles.modelCurrent}>
          <div className={styles.modelCurrentHeader}>
            <span className={styles.modelIcon}>{currentModelInfo?.icon || '🤖'}</span>
            <div className={styles.modelCurrentInfo}>
              <span className={styles.modelCurrentName}>
                {currentModelInfo?.display_name || currentSelectedModel?.split('/').pop() || 'No Model'}
              </span>
              <span className={styles.modelCurrentProvider}>
                {currentModelInfo?.provider || 'Unknown'}
              </span>
            </div>
          </div>
          <button
            onClick={() => setShowModelList(true)}
            className={styles.switchModelButton}
          >
            Switch
          </button>
        </div>

        {/* Model Categories */}
        <div className={styles.modelSection}>
          <h5>Model Categories</h5>
          <div className={styles.modelCategories}>
            <button
              className={styles.modelCategoryButton}
              onClick={() => {
                setActiveModelCategory('free');
                setShowModelList(true);
              }}
            >
              <span>🆓</span>
              <span>Free</span>
              <span className={styles.categoryCount}>
                {modelManagement.getFreeModels(models, 999).length}
              </span>
            </button>
            <button
              className={`${styles.modelCategoryButton} ${!hasApiKey ? styles.categoryLocked : ''}`}
              onClick={() => {
                setActiveModelCategory('fast');
                setShowModelList(true);
              }}
            >
              <span>⚡</span>
              <span>Fast</span>
              <span className={styles.categoryCount}>
                {modelManagement.getFastModels(models, 999).length}
              </span>
              {!hasApiKey && <span className={styles.lockBadge}>🔒</span>}
            </button>
            <button
              className={`${styles.modelCategoryButton} ${!hasApiKey ? styles.categoryLocked : ''}`}
              onClick={() => {
                setActiveModelCategory('thinkers');
                setShowModelList(true);
              }}
            >
              <span>🧠</span>
              <span>Thinkers</span>
              <span className={styles.categoryCount}>
                {modelManagement.getThinkerModels(models, 999).length}
              </span>
              {!hasApiKey && <span className={styles.lockBadge}>🔒</span>}
            </button>
            <button
              className={`${styles.modelCategoryButton} ${!hasApiKey ? styles.categoryLocked : ''}`}
              onClick={() => {
                setActiveModelCategory('image');
                setShowModelList(true);
              }}
            >
              <span>🎨</span>
              <span>Image</span>
              <span className={styles.categoryCount}>
                {modelManagement.getImageModels(models, 999).length}
              </span>
              {!hasApiKey && <span className={styles.lockBadge}>🔒</span>}
            </button>
          </div>
        </div>

        {/* Model Stats */}
        {currentModelInfo && (
          <div className={styles.modelSection}>
            <h5>Model Details</h5>
            <div className={styles.modelStats}>
              {currentModelInfo.context_length && (
                <div className={styles.modelStat}>
                  <label>Context:</label>
                  <span>{(currentModelInfo.context_length / 1000).toFixed(0)}K</span>
                </div>
              )}
              {currentModelInfo.pricing && (
                <div className={styles.modelStat}>
                  <label>Pricing:</label>
                  <span>
                    {currentModelInfo.pricing.prompt === '0' ? '🆓 Free' :
                     `$${(parseFloat(currentModelInfo.pricing.prompt) * 1000000).toFixed(2)}/M`}
                  </span>
                </div>
              )}
              {currentModelInfo.tier && (
                <div className={styles.modelStat}>
                  <label>Tier:</label>
                  <span className={styles.tierBadge}>
                    {currentModelInfo.tier === 'economical' ? '🆓 Free' :
                     currentModelInfo.tier === 'fast' ? '⚡ Fast' :
                     currentModelInfo.tier === 'premium' ? '💎 Premium' : '⭐ Standard'}
                  </span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Refresh Button */}
        <button
          onClick={() => modelManagement.loadAvailableModels()}
          className={styles.refreshModelsButton}
        >
          🔄 Refresh Models
        </button>
      </div>
    );
  };

  const renderModelSelection = () => {
    if (!modelManagement) return null;

    const { availableModels: models, currentSelectedModel } = modelManagement;

    let displayModels: any[] = [];
    let categoryTitle = 'All Models';

    if (activeModelCategory === 'free') {
      displayModels = modelManagement.getFreeModels(models, 999);
      categoryTitle = '🆓 Free Models';
    } else if (activeModelCategory === 'fast') {
      displayModels = modelManagement.getFastModels(models, 999);
      categoryTitle = '⚡ Fast Models';
    } else if (activeModelCategory === 'thinkers') {
      displayModels = modelManagement.getThinkerModels(models, 999);
      categoryTitle = '🧠 Thinker Models';
    } else if (activeModelCategory === 'image') {
      displayModels = modelManagement.getImageModels(models, 999);
      categoryTitle = '🎨 Image Models';
    } else {
      displayModels = models.slice(0, 50);
      categoryTitle = 'All Models';
    }

    return (
      <div className={styles.modelSelection}>
        <div className={styles.modelSelectionHeader}>
          <button
            onClick={() => {
              setShowModelList(false);
              setActiveModelCategory(null);
            }}
            className={styles.backButton}
          >
            ← Back
          </button>
          <span className={styles.modelSelectionTitle}>{categoryTitle}</span>
        </div>

        <div className={styles.modelList}>
          {displayModels.map((model, index) => {
            const locked = isModelLocked(model);
            return (
              <button
                key={`${model.name}-${index}`}
                className={`${styles.modelItem} ${model.name === currentSelectedModel ? styles.selectedModel : ''} ${locked ? styles.modelLocked : ''}`}
                onClick={() => {
                  if (locked) return; // Don't allow selection of locked models
                  modelManagement.handleModelSelection(model.name);
                  setShowModelList(false);
                  setActiveModelCategory(null);
                }}
                disabled={locked}
                title={locked ? 'Add an API key in Settings or purchase credits to unlock' : undefined}
              >
                <div className={styles.modelItemInfo}>
                  <span className={styles.modelItemIcon}>{model.icon || '🤖'}</span>
                  <div>
                    <span className={styles.modelItemName}>{model.display_name || model.name}</span>
                    <span className={styles.modelItemProvider}>{model.provider}</span>
                  </div>
                </div>
                <div className={styles.modelItemMeta}>
                  {locked ? (
                    <span className={styles.lockedIcon} title="Add an API key in Settings or purchase credits to unlock">🔒</span>
                  ) : (
                    <span className={styles.modelTier}>
                      {model.tier === 'economical' ? '🆓' :
                       model.tier === 'fast' ? '⚡' :
                       model.tier === 'premium' ? '💎' : '⭐'}
                    </span>
                  )}
                  {model.name === currentSelectedModel && <span className={styles.selectedBadge}>✓</span>}
                </div>
              </button>
            );
          })}
        </div>
      </div>
    );
  };

  // ============================================
  // RENDER SCOPE CONTENT
  // ============================================

  const renderScopeContent = () => {
    switch (selectedScope) {
      case 'tasks':
        if (showTaskForm) return renderTaskForm();
        if (selectedTaskId) return renderTaskDetails();
        return renderTaskList();

      case 'agents':
        if (selectedAgentId) return renderAgentDetails();
        return renderAgentList();

      case 'model-info':
        return renderModelInfo();

      default:
        return null;
    }
  };

  if (!isActive) return null;

  return (
    <div className={styles.voidScopesContainer} ref={containerRef}>
      <div className={styles.scopeBar}>
        <div className={styles.scopeSelector}>
          <button
            type="button"
            className={`${styles.scopeToggle} ${isDropdownOpen || selectedScope ? styles.active : ''}`}
            onClick={handleToggleDropdown}
            aria-label="Toggle scope selector"
          >
            <span className={styles.scopeLabel}>{getSelectedLabel()}</span>
            <span className={styles.carrot} style={{ transform: isDropdownOpen || selectedScope ? 'rotate(180deg)' : 'rotate(0deg)' }}>
              ▲
            </span>
          </button>

          {selectedScope && (
            <button type="button" className={styles.closeButton} onClick={handleClose} aria-label="Close scope">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>

        {isDropdownOpen && (
          <div className={styles.dropdownMenu} ref={dropdownRef}>
            {SCOPE_OPTIONS.map((option) => (
              <button
                key={option.id}
                type="button"
                className={`${styles.dropdownItem} ${selectedScope === option.id ? styles.dropdownItemActive : ''}`}
                onClick={() => handleScopeSelect(option.id)}
              >
                <span className={styles.dropdownIcon}>{option.icon}</span>
                <span className={styles.dropdownLabel}>{option.label}</span>
              </button>
            ))}
          </div>
        )}

        {selectedScope && (
          <div className={styles.scopePanel}>
            <div className={styles.scopeContent}>
              {renderScopeContent()}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default VoidScopes;
