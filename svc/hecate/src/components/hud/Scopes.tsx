import React, { useState, useEffect } from 'react';
import ModelInfo from './ModelInfo';
import TaskCreationForm from './TaskCreationForm';
import MarkdownRenderer from '../common/MarkdownRenderer';
import styles from './hud.module.scss';
import { Task } from '../../types/tasks';
import { Agent } from '../../types/agents';
import { agentService } from '../../common/services/agent-service';

interface ScopesProps {
  activeScope: string | null;
  setActiveLens: (scope: string | null) => void;
  isScopesExpanded: boolean;
  setIsScopesExpanded: (expanded: boolean) => void;
  isChatExpanded: boolean;
  setIsChatExpanded: (expanded: boolean) => void;
  showScopeDropdown: boolean;
  setShowScopeDropdown: (show: boolean) => void;
  scopeDropdownRef: React.RefObject<HTMLDivElement>;
  nullviewState: string;
  tasks: Task[];
  taskManagement: any;
  logs: any[];
  searchTerm: string;
  setSearchTerm: (term: string) => void;
  logFilter: string;
  setLogFilter: (filter: string) => void;
  autoScroll: boolean;
  setAutoScroll: (scroll: boolean) => void;
  logsEndRef: React.RefObject<HTMLDivElement>;
  theme: string;
  onThemeChange: (theme: 'null' | 'light' | 'dark') => void;
  // Model info props
  isLoadingModelInfo: boolean;
  modelInfo: any;
  currentSelectedModel: string | null;
  availableModels: any[];
  defaultModelLoaded: boolean;
  showModelSelection: boolean;
  setShowModelSelection: (show: boolean) => void;
  setActiveQuickAction: (action: string | null) => void;
  setModelsCached: (cached: boolean) => void;
  loadAvailableModels: () => Promise<void>;
  showFullDescription: boolean;
  setShowFullDescription: (show: boolean) => void;
  modelSearchQuery: string;
  setModelSearchQuery: (query: string) => void;
  isSearchingModels: boolean;
  searchResults: any[];
  searchSubmitted: boolean;
  setSearchSubmitted: (submitted: boolean) => void;
  showSearchDropdown: boolean;
  setShowSearchDropdown: (show: boolean) => void;
  searchDropdownRef: React.RefObject<HTMLDivElement>;
  activeQuickAction: string | null;
  // Chat props for agent switching
  activeAgent?: 'hecate' | 'siren';
  setActiveAgent?: (agent: 'hecate' | 'siren') => void;
  categoryModels: any[];
  isLoadingCategory: boolean;
  setCategoryModels: (models: any[]) => void;
  loadCategoryModels: (category: string) => void;
  handleModelSelection: (modelName: string) => void;
  getFreeModels: (models: any[], limit?: number) => any[];
  getFastModels: (models: any[], limit?: number) => any[];
  getPremiumModels: (models: any[], limit?: number) => any[];
  getThinkerModels: (models: any[], limit?: number) => any[];
  getInstructModels: (models: any[], limit?: number) => any[];
  getLatestModels: (models: any[], limit?: number) => any[];
}

type TaskCategory = 'todo' | 'running' | 'completed';

const Scopes: React.FC<ScopesProps> = ({
  activeScope,
  setActiveLens,
  isScopesExpanded,
  setIsScopesExpanded,
  isChatExpanded,
  setIsChatExpanded,
  showScopeDropdown,
  setShowScopeDropdown,
  scopeDropdownRef,
  nullviewState,
  tasks,
  taskManagement,
  logs,
  searchTerm,
  setSearchTerm,
  logFilter,
  setLogFilter,
  autoScroll,
  setAutoScroll,
  logsEndRef,
  theme,
  onThemeChange,
  activeAgent,
  setActiveAgent,
  ...modelInfoProps
}) => {
  const [showTaskForm, setShowTaskForm] = useState(false);
  const [activeTaskCategory, setActiveTaskCategory] = useState<TaskCategory>('todo');
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  // Agent state management
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoadingAgents, setIsLoadingAgents] = useState(false);
  const [agentsError, setAgentsError] = useState<string | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  // Fetch agents on component mount
  useEffect(() => {
    const fetchAgents = async () => {
      setIsLoadingAgents(true);
      setAgentsError(null);

      try {
        const response = await agentService.getAgents();
        if (response.success && response.data) {
          setAgents(response.data.agents);
          console.log(`✅ Loaded ${response.data.agents.length} agents`);
        } else {
          setAgentsError(response.error || 'Failed to load agents');
          console.warn('⚠️ Failed to load agents:', response.error);
        }
      } catch (error) {
        const errorMessage = (error as Error).message;
        setAgentsError(errorMessage);
        console.error('❌ Error loading agents:', error);
      } finally {
        setIsLoadingAgents(false);
      }
    };

    fetchAgents();
  }, []);

  const scopesOptions = [
    { id: 'modelinfo', icon: '🤖', title: 'Model Info', description: 'Current model details', color: '#ff6b6b' },
    { id: 'tasks', icon: '📋', title: 'Tasks', description: 'Active agent tasks', color: '#4ecdc4' },
    { id: 'agents', icon: '🤖', title: 'Agents', description: 'Agent monitoring', color: '#45b7d1' },
    { id: 'logs', icon: '📄', title: 'Logs', description: 'System logs', color: '#96ceb4' },
    { id: 'settings', icon: '⚙️', title: 'Settings', description: 'Theme & social links', color: '#747d8c' },
  ];

  const getCurrentScopeInfo = () => {
    return scopesOptions.find(scope => scope.id === activeScope) || scopesOptions[1];
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
      case 'active':
        return styles.statusRunning;
      case 'paused':
        return styles.statusPaused;
      case 'completed':
      case 'success':
        return styles.statusCompleted;
      case 'failed':
      case 'error':
        return styles.statusFailed;
      case 'cancelled':
        return styles.statusCancelled;
      case 'created':
      case 'pending':
      case 'idle':
        return styles.statusPending;
      default:
        return styles.statusPending;
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
      case 'active':
        return '⚡';
      case 'paused':
        return '⏸️';
      case 'completed':
      case 'success':
        return '✅';
      case 'failed':
      case 'error':
        return '❌';
      case 'cancelled':
        return '🚫';
      case 'created':
      case 'pending':
      case 'idle':
        return '⏳';
      default:
        return '❓';
    }
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return styles.logError;
      case 'warning':
        return styles.logWarning;
      case 'success':
        return styles.logSuccess;
      case 'debug':
        return styles.logDebug;
      default:
        return styles.logInfo;
    }
  };

  const filteredLogs = logs.filter((log) => {
    const matchesSearch =
      log.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
      log.source.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesFilter = logFilter === 'all' || log.level === logFilter;
    return matchesSearch && matchesFilter;
  });

  const handleScopesClick = (scopeId: string) => {
    const newScope = activeScope === scopeId ? null : scopeId;
    setActiveLens(newScope);
  };

  // Categorize tasks
  const todoTasks = tasks.filter(task => task.status === 'created');
  const runningTasks = tasks.filter(task => task.status === 'running' || task.status === 'paused');
  const completedTasks = tasks.filter(task => task.status === 'completed' || task.status === 'failed' || task.status === 'cancelled');

  const getTasksForCategory = (category: TaskCategory) => {
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
    if (!selectedTask) return null;

    return (
      <div className={styles.taskDetailsContainer}>
        <div className={styles.taskDetailsHeader}>
          <button
            onClick={() => setSelectedTaskId(null)}
            className={styles.backButton}
          >
            ← Back to Tasks
          </button>
          <h4>📋 Task Details</h4>
        </div>

        <div className={styles.taskDetailsBody}>
          <div className={styles.taskDetailsSection}>
            <h5>Basic Information</h5>
            <div className={styles.taskDetailsGrid}>
              <div className={styles.taskDetailsField}>
                <label>Name:</label>
                <span>{selectedTask.name}</span>
              </div>
              <div className={styles.taskDetailsField}>
                <label>Type:</label>
                <span>{selectedTask.task_type}</span>
              </div>
              <div className={styles.taskDetailsField}>
                <label>Status:</label>
                <span className={`${styles.statusBadge} ${getStatusColor(selectedTask.status)}`}>
                  {getStatusIcon(selectedTask.status)} {selectedTask.status}
                </span>
              </div>
              <div className={styles.taskDetailsField}>
                <label>Priority:</label>
                <span className={styles.priorityBadge}>{selectedTask.priority}</span>
              </div>
            </div>
            <div className={styles.taskDetailsField}>
              <label>Description:</label>
              <p>{selectedTask.description}</p>
            </div>
          </div>

          <div className={styles.taskDetailsSection}>
            <h5>Execution</h5>
            <div className={styles.taskDetailsGrid}>
              <div className={styles.taskDetailsField}>
                <label>Progress:</label>
                <div className={styles.progressContainer}>
                  <div className={styles.progressBar}>
                    <div
                      className={styles.progressFill}
                      style={{ width: `${selectedTask.status === 'completed' ? 100 : selectedTask.progress}%` }}
                    ></div>
                  </div>
                  <span>
                    {selectedTask.status === 'completed' ? '100' : Math.round(selectedTask.progress)}%
                  </span>
                </div>
              </div>
              {selectedTask.assigned_agent && (
                <div className={styles.taskDetailsField}>
                  <label>Assigned Agent:</label>
                  <span>{selectedTask.assigned_agent}</span>
                </div>
              )}
              {selectedTask.action_duration && (
                <div className={styles.taskDetailsField}>
                  <label>Duration:</label>
                  <span>{(selectedTask.action_duration / 1000).toFixed(2)}s</span>
                </div>
              )}
              <div className={styles.taskDetailsField}>
                <label>Created:</label>
                <span>{new Date(selectedTask.created_at).toLocaleString()}</span>
              </div>
              {selectedTask.completed_at && (
                <div className={styles.taskDetailsField}>
                  <label>Completed:</label>
                  <span>{new Date(selectedTask.completed_at).toLocaleString()}</span>
                </div>
              )}
            </div>
          </div>

          <div className={styles.taskDetailsSection}>
            <h5>Invocation Trail</h5>
            <div className={styles.executionTrail}>
              <div className={styles.trailItem}>
                <div className={styles.trailContent}>
                  <div className={styles.trailLabel}>Source</div>
                  <div className={styles.trailValue}>
                    {(() => {
                      const sourceType = selectedTask.source_metadata?.type || 'unknown';
                      const sourceValue = selectedTask.source_metadata?.wallet_address ||
                                        selectedTask.source_metadata?.identifier ||
                                        selectedTask.source_identifier ||
                                        'Unknown';

                      return `${sourceType}: ${sourceValue}`;
                    })()}
                  </div>
                </div>
              </div>
              {selectedTask.agent_uuid && (
                <div className={styles.trailItem}>
                  <div className={styles.trailIcon}>🤖</div>
                  <div className={styles.trailContent}>
                    <div className={styles.trailLabel}>Agent</div>
                    <div className={styles.trailValue}>{selectedTask.agent_uuid}</div>
                  </div>
                </div>
              )}
              {selectedTask.api_call_id && (
                <div className={styles.trailItem}>
                  <div className={styles.trailIcon}>🔗</div>
                  <div className={styles.trailContent}>
                    <div className={styles.trailLabel}>API Call</div>
                    <div className={styles.trailValue}>{selectedTask.api_call_id}</div>
                  </div>
                </div>
              )}
              {selectedTask.invocation_chain && (
                <div className={styles.trailItem}>
                  <div className={styles.trailIcon}>⛓️</div>
                  <div className={styles.trailContent}>
                    <div className={styles.trailLabel}>Chain</div>
                    <div className={styles.trailValue}>{selectedTask.invocation_chain}</div>
                  </div>
                </div>
              )}
            </div>
          </div>

          {selectedTask.action_result && (
            <div className={styles.taskDetailsSection}>
              <h5>Result</h5>
              <div className={styles.taskResultBox}>
                <MarkdownRenderer content={selectedTask.action_result} />
              </div>
            </div>
          )}

          <div className={styles.taskDetailsActions}>
            {selectedTask.status === 'created' && (
              <button
                onClick={() => {
                  taskManagement.startTask(selectedTask.id);
                  setSelectedTaskId(null);
                }}
                className={styles.taskActionButton}
              >
                ▶️ Start Task
              </button>
            )}
            {selectedTask.status === 'running' && (
              <>
                <button
                  onClick={() => {
                    taskManagement.pauseTask(selectedTask.id);
                    setSelectedTaskId(null);
                  }}
                  className={styles.taskActionButton}
                >
                  ⏸️ Pause
                </button>
                <button
                  onClick={() => {
                    taskManagement.cancelTask(selectedTask.id);
                    setSelectedTaskId(null);
                  }}
                  className={styles.taskActionButton}
                >
                  🚫 Cancel
                </button>
              </>
            )}
            {selectedTask.status === 'failed' && (
              <button
                onClick={() => {
                  taskManagement.retryTask(selectedTask.id);
                  setSelectedTaskId(null);
                }}
                className={styles.taskActionButton}
              >
                🔄 Retry
              </button>
            )}
            <button
              onClick={() => {
                taskManagement.deleteTask(selectedTask.id);
                setSelectedTaskId(null);
              }}
              className={`${styles.taskActionButton} ${styles.dangerButton}`}
            >
              🗑️ Delete
            </button>
          </div>
        </div>
      </div>
    );
  };

  const renderScopeDropdown = () => (
    <div className={styles.scopeDropdownContainer} ref={scopeDropdownRef}>
      <button
        className={styles.scopeDropdownButtonAsTitle}
        onClick={() => setShowScopeDropdown(!showScopeDropdown)}
      >
        <span className={styles.scopeDropdownIcon}>{getCurrentScopeInfo().icon}</span>
        <span className={styles.scopeDropdownTitle}>{getCurrentScopeInfo().title}</span>
        <span className={styles.scopeDropdownArrow}>{showScopeDropdown ? '▲' : '▼'}</span>
      </button>

      {showScopeDropdown && (
        <div className={styles.scopeDropdownMenu}>
          {scopesOptions.map((scope) => (
            <button
              key={scope.id}
              className={`${styles.scopeDropdownItem} ${activeScope === scope.id ? styles.active : ''}`}
              onClick={() => {
                handleScopesClick(scope.id);
                setShowScopeDropdown(false);
              }}
              style={{ '--scope-color': scope.color } as React.CSSProperties}
            >
              <span className={styles.scopeItemIcon}>{scope.icon}</span>
              <div className={styles.scopeItemContent}>
                <span className={styles.scopeItemTitle}>{scope.title}</span>
                <span className={styles.scopeItemDescription}>{scope.description}</span>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );

  const renderScopeContent = () => {
    switch (activeScope) {
      case 'modelinfo':
        return <ModelInfo {...modelInfoProps} />;

      case 'tasks':
        return (
          <div className={`${styles.tasksScope} ${showTaskForm ? styles.showingTaskForm : ''}`}>
            {selectedTaskId ? (
              renderTaskDetails()
            ) : showTaskForm ? (
              <div className={styles.taskFormContainer}>
                <div className={styles.taskFormHeader}>
                  <h3>📋 Create New Task</h3>
                  <button
                    onClick={() => setShowTaskForm(false)}
                    className={styles.backButton}
                    title="Back to Task List"
                  >
                    ← Back to Tasks
                  </button>
                </div>
                <TaskCreationForm
                  onCreateTask={taskManagement.createTask}
                  isLoading={taskManagement.isLoading}
                  onCancel={() => setShowTaskForm(false)}
                  variant="fullscreen"
                />
              </div>
            ) : (
              <>
                <div className={styles.tasksHeader}>
                  <div className={styles.taskTabs}>
                    <button
                      className={`${styles.taskTab} ${activeTaskCategory === 'todo' ? styles.activeTab : ''}`}
                      onClick={() => setActiveTaskCategory('todo')}
                    >
                      📋 Todo ({todoTasks.length})
                    </button>
                    <button
                      className={`${styles.taskTab} ${activeTaskCategory === 'running' ? styles.activeTab : ''}`}
                      onClick={() => setActiveTaskCategory('running')}
                    >
                      ⚡ Running ({runningTasks.length})
                    </button>
                    <button
                      className={`${styles.taskTab} ${activeTaskCategory === 'completed' ? styles.activeTab : ''}`}
                      onClick={() => setActiveTaskCategory('completed')}
                    >
                      ✅ Completed ({completedTasks.length})
                    </button>
                  </div>
                  <button
                    onClick={() => setShowTaskForm(true)}
                    className={styles.taskActionButton}
                    title="Create Custom Task"
                  >
                    ➕ Create Task
                  </button>
                </div>
                <div className={styles.taskScrollContainer}>
                  {currentCategoryTasks.length === 0 ? (
                    <div className={styles.emptyState}>
                      <p>No {activeTaskCategory} tasks found</p>
                      {activeTaskCategory === 'todo' && (
                        <p className={styles.emptyHint}>Use the "➕ Create Task" button above to get started</p>
                      )}
                    </div>
                  ) : (
                    currentCategoryTasks.map((task) => (
                  <div
                    key={task.id}
                    className={`${styles.taskItem} ${getStatusColor(task.status)} ${styles.clickableTask}`}
                    onClick={() => setSelectedTaskId(task.id)}
                    title="Click to view details"
                  >
                    <div className={styles.taskHeader}>
                      <div className={styles.taskInfo}>
                        <span className={styles.taskName}>{task.name}</span>
                        <span className={styles.taskType}>{task.task_type}</span>
                      </div>
                      <div className={styles.taskStatus}>
                        <span className={styles.statusIcon}>{getStatusIcon(task.status)}</span>
                        {task.status}
                      </div>
                    </div>
                    <div className={styles.taskMetadata}>
                      <span className={styles.taskTime}>
                        Created: {new Date(task.created_at).toLocaleTimeString()}
                      </span>
                      {task.completed_at && (
                        <span className={styles.taskTime}>
                          Completed: {new Date(task.completed_at).toLocaleTimeString()}
                        </span>
                      )}
                      <span className={styles.taskInitiator}>
                        Initiator: {task.category === 'user' && '👤 User'}
                        {task.category === 'agent' && '🤖 Agent'}
                        {task.category === 'api' && '🔗 API'}
                        {task.category === 'system' && '⚙️ System'}
                        {task.category === 'scheduled' && '⏰ Scheduled'}
                        {task.category === 'automated' && '🤖 Automated'}
                        {task.category === 'manual' && '👤 Manual'}
                        {task.category === 'webhook' && '🔗 Webhook'}
                        {task.category === 'cron' && '⏰ Cron'}
                        {task.category && !['user', 'agent', 'api', 'system', 'scheduled', 'automated', 'manual', 'webhook', 'cron'].includes(task.category) && `📋 ${task.category}`}
                      </span>
                    </div>
                    <div className={styles.taskActions}>
                      {task.status === 'created' && (
                        <>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              taskManagement.startTask(task.id);
                            }}
                            className={styles.taskActionButton}
                            title="Start Task"
                          >
                            ▶️ Start
                          </button>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              taskManagement.processTask(task.id);
                            }}
                            className={styles.taskActionButton}
                            title="Process Task with Hecate"
                          >
                            ⚡ Process
                          </button>
                        </>
                      )}
                      {task.status === 'running' && (
                        <>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              taskManagement.pauseTask(task.id);
                            }}
                            className={styles.taskActionButton}
                            title="Pause Task"
                          >
                            ⏸️ Pause
                          </button>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              taskManagement.cancelTask(task.id);
                            }}
                            className={styles.taskActionButton}
                            title="Cancel Task"
                          >
                            ❌ Cancel
                          </button>
                        </>
                      )}
                      {task.status === 'paused' && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            taskManagement.resumeTask(task.id);
                          }}
                          className={styles.taskActionButton}
                          title="Resume Task"
                        >
                          ▶️ Resume
                        </button>
                      )}
                      {task.status === 'failed' && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            taskManagement.retryTask(task.id);
                          }}
                          className={styles.taskActionButton}
                          title="Retry Task"
                        >
                          🔄 Retry
                        </button>
                      )}
                    </div>
                  </div>
                  ))
                  )}
                </div>
              </>
            )}
          </div>
        );

      case 'agents':
        return (
          <div className={styles.agentsScope}>
            {!selectedAgentId && (
              <div className={styles.agentsHeader}>
                <h6>🤖 Active Agents ({agents.length})</h6>
                <button
                  onClick={() => {
                    setIsLoadingAgents(true);
                    setAgentsError(null);
                    agentService.getAgents().then(response => {
                      if (response.success && response.data) {
                        setAgents(response.data.agents);
                      } else {
                        setAgentsError(response.error || 'Failed to refresh agents');
                      }
                      setIsLoadingAgents(false);
                    });
                  }}
                  className={styles.refreshButton}
                  disabled={isLoadingAgents}
                  title="Refresh agents"
                >
                  {isLoadingAgents ? '🔄' : '🔄'}
                </button>
              </div>
            )}

            {agentsError && (
              <div className={styles.errorMessage}>
                <span>❌ {agentsError}</span>
              </div>
            )}

            {isLoadingAgents ? (
              <div className={styles.loadingState}>
                <span>🔄 Loading agents...</span>
              </div>
            ) : selectedAgentId ? (
              <div className={styles.agentDetails}>
                {(() => {
                  const selectedAgent = agents.find(a => a.name === selectedAgentId);
                  if (!selectedAgent) return null;

                  return (
                    <div className={styles.agentDetailsContainer}>
                      <div className={styles.agentDetailsHeader}>
                        <div className={styles.agentDetailsHeaderLeft}>
                          <button
                            onClick={() => setSelectedAgentId(null)}
                            className={styles.backButton}
                          >
                            ← Back to Agents
                          </button>
                        </div>
                        <h4>{selectedAgent.name.charAt(0).toUpperCase() + selectedAgent.name.slice(1)}</h4>
                        <div className={styles.agentDetailsHeaderActions}>
                          <button
                            onClick={() => {
                              console.log(`💬 Starting chat with ${selectedAgent.name}`);
                              console.log(`🔍 setActiveAgent function:`, setActiveAgent);
                              console.log(`🔍 activeAgent current:`, activeAgent);

                              // Switch to the selected agent
                              if (setActiveAgent && (selectedAgent.name === 'hecate' || selectedAgent.name === 'siren')) {
                                setActiveAgent(selectedAgent.name);
                                console.log(`🔄 Switched active agent to: ${selectedAgent.name}`);

                                // Keep the current view open - don't close agent details
                                console.log(`✅ Activated chat with ${selectedAgent.name} agent - keeping current view`);
                              } else {
                                console.warn(`⚠️ Agent ${selectedAgent.name} not supported for chat yet or setActiveAgent not available`);
                                console.warn(`⚠️ setActiveAgent available:`, !!setActiveAgent);
                                console.warn(`⚠️ Agent name:`, selectedAgent.name);
                              }
                            }}
                            className={styles.agentActionButton}
                            disabled={!agentService.isAgentOnline(selectedAgent)}
                          >
                            💬 Chat
                          </button>
                          <button
                            onClick={() => {
                              agentService.getAgentHealth(selectedAgent.name).then(response => {
                                console.log(`🏥 Health check for ${selectedAgent.name}:`, response);
                              });
                            }}
                            className={styles.agentActionButton}
                          >
                            🏥 Health
                          </button>
                        </div>
                      </div>

                      <div className={styles.agentDetailsBody}>
                        <div className={styles.agentDetailsSection}>
                          <h5>Basic Information</h5>
                          <div className={styles.agentDetailsGrid}>
                            <div className={styles.agentDetailsField}>
                              <label>Type:</label>
                              <span>{selectedAgent.type}</span>
                            </div>
                            <div className={styles.agentDetailsField}>
                              <label>Status:</label>
                              <span
                                className={styles.statusBadge}
                                style={{ backgroundColor: agentService.getAgentStatusColor(selectedAgent.status) }}
                              >
                                {selectedAgent.status === 'healthy' ? '✅' : '❌'} {selectedAgent.status}
                              </span>
                            </div>
                            <div className={styles.agentDetailsField}>
                              <label>Endpoint:</label>
                              <span>{selectedAgent.endpoint}</span>
                            </div>
                          </div>
                          <div className={styles.agentDetailsField}>
                            <label>Description:</label>
                            <p>{selectedAgent.description}</p>
                          </div>
                        </div>

                        {selectedAgent.name === 'hecate' ? (
                          <>
                            <div className={styles.agentDetailsSection}>
                              <h5>🎯 Core Mission</h5>
                              <p>Hecate serves as NullBlock's neural core and primary conversational interface. As the orchestration engine, Hecate coordinates specialized agents for blockchain operations, DeFi analysis, market intelligence, and complex workflow management.</p>
                            </div>

                            <div className={styles.agentDetailsSection}>
                              <h5>🧠 Key Capabilities</h5>
                              <div className={styles.capabilitiesList}>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>🤖</span>
                                  <span className={styles.capabilityName}>Multi-Agent Orchestration</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>💬</span>
                                  <span className={styles.capabilityName}>Conversational Interface</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>🔍</span>
                                  <span className={styles.capabilityName}>Intent Analysis</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>📋</span>
                                  <span className={styles.capabilityName}>Task Management</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>🧠</span>
                                  <span className={styles.capabilityName}>LLM Coordination</span>
                                </div>
                              </div>
                            </div>

                            <div className={styles.agentDetailsSection}>
                              <h5>🔧 Technical Features</h5>
                              <ul className={styles.detailsList}>
                                <li>Multi-model LLM support (DeepSeek, GPT-4o, Claude, etc.)</li>
                                <li>Real-time task lifecycle management</li>
                                <li>Session-based conversation tracking</li>
                                <li>Cost tracking and optimization</li>
                                <li>WebSocket chat integration</li>
                                <li>Agent delegation and coordination</li>
                              </ul>
                            </div>
                          </>
                        ) : selectedAgent.name === 'siren' ? (
                          <>
                            <div className={styles.agentDetailsSection}>
                              <h5>🎯 Core Mission</h5>
                              <p>Siren serves as NullBlock's frontline evangelist in the decentralized arena, driving go-to-market strategies, tokenomics storytelling, and viral outreach to amplify adoption across blockchain networks.</p>
                            </div>

                            <div className={styles.agentDetailsSection}>
                              <h5>🎭 Personality</h5>
                              <p>Irresistibly charismatic with a siren's allure—persuasive yet transparent, blending neon-lit flair with genuine enthusiasm for decentralized innovation. Siren thrives on interaction, turning cold leads into fervent advocates.</p>
                            </div>

                            <div className={styles.agentDetailsSection}>
                              <h5>🚀 Key Capabilities</h5>
                              <div className={styles.capabilitiesList}>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>📝</span>
                                  <span className={styles.capabilityName}>Campaign Design</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>💰</span>
                                  <span className={styles.capabilityName}>Tokenomics Narrative</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>📊</span>
                                  <span className={styles.capabilityName}>Sentiment Analysis</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>🤝</span>
                                  <span className={styles.capabilityName}>Partnership Brokering</span>
                                </div>
                                <div className={styles.capabilityItem}>
                                  <span className={styles.capabilityIcon}>📱</span>
                                  <span className={styles.capabilityName}>Social Media Management</span>
                                </div>
                              </div>
                            </div>

                            <div className={styles.agentDetailsSection}>
                              <h5>🎨 Specialization Areas</h5>
                              <ul className={styles.detailsList}>
                                <li>DeFi hype cycles and viral marketing</li>
                                <li>Social sentiment amplification</li>
                                <li>Ecosystem partnership development</li>
                                <li>Community engagement strategies</li>
                                <li>Brand storytelling and narrative crafting</li>
                                <li>Cross-platform campaign optimization</li>
                              </ul>
                            </div>

                            <div className={styles.agentDetailsSection}>
                              <h5>🔗 Integration</h5>
                              <p>Orchestrated via Hecate's neural core; invoke Siren for tasks requiring external-facing communications or market pulse checks. Best paired with Analytics Agent for data-backed campaigns.</p>
                            </div>
                          </>
                        ) : null}

                        <div className={styles.agentDetailsSection}>
                          <h5>Capabilities</h5>
                          <div className={styles.capabilitiesList}>
                            {selectedAgent.capabilities.map((capability, index) => (
                              <div key={index} className={styles.capabilityItem}>
                                <span className={styles.capabilityIcon}>
                                  {agentService.getCapabilityIcon(capability)}
                                </span>
                                <span className={styles.capabilityName}>
                                  {capability.replace(/_/g, ' ')}
                                </span>
                              </div>
                            ))}
                          </div>
                        </div>

                        {selectedAgent.metrics && (
                          <div className={styles.agentDetailsSection}>
                            <h5>Metrics</h5>
                            <div className={styles.metricsList}>
                              {agentService.getAgentMetrics(selectedAgent).map((metric, index) => (
                                <div key={index} className={styles.metricItem}>
                                  <span>{metric}</span>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                      </div>
                    </div>
                  );
                })()}
              </div>
            ) : (
              <div className={styles.agentsList}>
                {agents.length === 0 ? (
                  <div className={styles.emptyState}>
                    <p>No agents found</p>
                    <p className={styles.emptyHint}>Check that the agents service is running</p>
                  </div>
                ) : (
                  agents.map((agent) => (
                    <div
                      key={agent.name}
                      className={`${styles.agentItem} ${styles.clickableAgent}`}
                      onClick={() => setSelectedAgentId(agent.name)}
                      title="Click to view details"
                    >
                      <div className={styles.agentHeader}>
                        <div className={styles.agentInfo}>
                          <span className={styles.agentName}>
                            {agent.name.charAt(0).toUpperCase() + agent.name.slice(1)}
                          </span>
                          <span className={styles.agentType}>{agent.type}</span>
                        </div>
                        <div className={styles.agentStatus}>
                          <span
                            className={styles.statusIcon}
                            style={{ color: agentService.getAgentStatusColor(agent.status) }}
                          >
                            {agent.status === 'healthy' ? '✅' : '❌'}
                          </span>
                          {agent.status}
                        </div>
                      </div>

                      <div className={styles.agentDescription}>
                        <span>{agentService.getAgentDescription(agent)}</span>
                      </div>

                      <div className={styles.agentCapabilities}>
                        <span className={styles.capabilitiesLabel}>Capabilities:</span>
                        <div className={styles.capabilitiesPreview}>
                          {agent.capabilities.slice(0, 3).map((capability, index) => (
                            <span key={index} className={styles.capabilityTag}>
                              {agentService.getCapabilityIcon(capability)} {capability.replace(/_/g, ' ')}
                            </span>
                          ))}
                          {agent.capabilities.length > 3 && (
                            <span className={styles.capabilityTag}>
                              +{agent.capabilities.length - 3} more
                            </span>
                          )}
                        </div>
                      </div>

                      {agent.metrics && (
                        <div className={styles.agentMetrics}>
                          {agentService.getAgentMetrics(agent).slice(0, 2).map((metric, index) => (
                            <span key={index}>{metric}</span>
                          ))}
                        </div>
                      )}

                      {agent.note && (
                        <div className={styles.agentNote}>
                          <span>⚠️ {agent.note}</span>
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
        );

      case 'logs':
        return (
          <div className={styles.logsScope}>
            <div className={styles.logsHeader}>
              <h6>📄 System Logs</h6>
              <div className={styles.logsControls}>
                <input
                  type="text"
                  placeholder="Search logs..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className={styles.searchInput}
                />
                <select
                  value={logFilter}
                  onChange={(e) => setLogFilter(e.target.value as any)}
                  className={styles.filterSelect}
                >
                  <option value="all">All Levels</option>
                  <option value="info">Info</option>
                  <option value="warning">Warning</option>
                  <option value="error">Error</option>
                  <option value="success">Success</option>
                  <option value="debug">Debug</option>
                </select>
                <label className={styles.autoScrollLabel}>
                  <input
                    type="checkbox"
                    checked={autoScroll}
                    onChange={(e) => setAutoScroll(e.target.checked)}
                  />
                  Auto-scroll
                </label>
              </div>
            </div>
            <div className={styles.logsContainer}>
              {filteredLogs.map((log) => (
                <div key={log.id} className={`${styles.logEntry} ${getLogLevelColor(log.level)}`}>
                  <div className={styles.logHeader}>
                    <span className={styles.logTimestamp}>
                      {log.timestamp.toLocaleTimeString()}
                    </span>
                    <span className={styles.logLevel}>[{log.level.toUpperCase()}]</span>
                    <span className={styles.logSource}>{log.source}</span>
                  </div>
                  <div className={styles.logMessage}>{log.message}</div>
                  {log.data && (
                    <div className={styles.logData}>
                      <pre>{JSON.stringify(log.data, null, 2)}</pre>
                    </div>
                  )}
                </div>
              ))}
              <div ref={logsEndRef} />
            </div>
          </div>
        );

      case 'settings':
        return (
          <div className={styles.settingsScope}>
            <div className={styles.settingsSection}>
              <h6>🎨 Theme</h6>
              <div className={styles.themeSelector}>
                <button
                  className={`${styles.themeButton} ${theme === 'null' ? styles.active : ''}`}
                  onClick={() => onThemeChange('null')}
                >
                  🌙 Dark
                  <span className={styles.wipBadge}>WIP</span>
                </button>
                <button
                  className={`${styles.themeButton} ${theme === 'dark' ? styles.active : ''}`}
                  onClick={() => onThemeChange('dark')}
                >
                  🌌 Null
                </button>
                <button
                  className={`${styles.themeButton} ${theme === 'light' ? styles.active : ''}`}
                  onClick={() => onThemeChange('light')}
                >
                  ☀️ Light
                  <span className={styles.wipBadge}>WIP</span>
                </button>
              </div>
            </div>

            <div className={styles.settingsSection}>
              <h6>ℹ️ Version Info</h6>
              <div className={styles.versionInfo}>
                <p><strong>NullBlock Platform:</strong> v1.0.0-beta</p>
                <p><strong>Hecate Agent:</strong> v0.8.2</p>
                <p><strong>MCP Protocol:</strong> v0.1.0</p>
                <p><strong>Build:</strong> {new Date().toLocaleDateString()}</p>
              </div>
            </div>

            <div className={styles.settingsSection}>
              <h6>🔗 Social Links</h6>
              <div className={styles.socialLinks}>
                <button
                  onClick={() => window.open('https://x.com/Nullblock_io', '_blank')}
                  className={styles.socialButton}
                >
                  🐦 𝕏
                </button>
                <button
                  onClick={() => window.open('https://discord.gg/nullblock', '_blank')}
                  className={styles.socialButton}
                >
                  💬 Discord
                </button>
                <button
                  onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
                  className={styles.socialButton}
                >
                  📚 Docs
                </button>
              </div>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  const renderAvatar = () => (
    <div className={styles.scopesAvatar}>
      <div className={styles.avatarCircle}>
        <div className={`${styles.nullviewAvatar} ${styles[nullviewState]} ${styles.clickableNulleye}`}>
          <div className={styles.pulseRingAvatar}></div>
          <div className={styles.dataStreamAvatar}>
            <div className={styles.streamLineAvatar}></div>
            <div className={styles.streamLineAvatar}></div>
            <div className={styles.streamLineAvatar}></div>
          </div>
          <div className={styles.lightningContainer}>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
          </div>
          <div className={styles.staticField}></div>
          <div className={styles.coreNodeAvatar}></div>
        </div>
      </div>
      <div className={styles.avatarInfo}>
        <h4>Hecate</h4>
        <p>Primary Interface Agent</p>
      </div>
    </div>
  );

  const expandedScopesContent = (
    <div className={styles.fullscreenOverlay}>
      <div className={`${styles.scopesSection} ${styles.expanded}`}>
        {activeScope ? (
          <div className={styles.scopesExpanded}>
            <div className={styles.scopesContent}>
              <div className={styles.scopesContentHeader}>
                {renderScopeDropdown()}
                <div className={styles.scopesHeaderControls}>
                  <button
                    className={styles.expandButton}
                    onClick={() => {
                      setIsScopesExpanded(false);
                    }}
                    title="Exit full screen"
                  >
                    ⊟
                  </button>
                </div>
              </div>
              <div className={styles.scopesContent}>
                {renderScopeContent()}
              </div>
            </div>
          </div>
        ) : (
          <div className={styles.scopesScrollContainer}>
            <div className={styles.chatHeader}>
              <div className={styles.chatTitle}>
                <div className={styles.scopeDropdownContainer} ref={scopeDropdownRef}>
                  <button
                    className={styles.scopeDropdownButtonAsTitle}
                    onClick={() => setShowScopeDropdown(!showScopeDropdown)}
                  >
                    <span className={styles.scopeDropdownIcon}>🎯</span>
                    <span className={styles.scopeDropdownTitle}>Scopes</span>
                    <span className={styles.scopeDropdownArrow}>{showScopeDropdown ? '▲' : '▼'}</span>
                  </button>

                  {showScopeDropdown && (
                    <div className={styles.scopeDropdownMenu}>
                      {scopesOptions.map((scope) => (
                        <button
                          key={scope.id}
                          className={`${styles.scopeDropdownItem} ${activeScope === scope.id ? styles.active : ''}`}
                          onClick={() => {
                            handleScopesClick(scope.id);
                            setShowScopeDropdown(false);
                          }}
                          style={{ '--scope-color': scope.color } as React.CSSProperties}
                        >
                          <span className={styles.scopeItemIcon}>{scope.icon}</span>
                          <div className={styles.scopeItemContent}>
                            <span className={styles.scopeItemTitle}>{scope.title}</span>
                            <span className={styles.scopeItemDescription}>{scope.description}</span>
                          </div>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
              <div className={styles.chatHeaderControls}>
                <button
                  className={styles.expandButton}
                  onClick={() => {
                    setIsScopesExpanded(false);
                  }}
                  title="Exit full screen"
                >
                  ⊟
                </button>
              </div>
            </div>
            <div className={styles.scopesInfoPanel}>
              <div className={styles.scopesInfoContent}>
                <div className={styles.scopesAppsSection}>
                  <div className={styles.scopesAppsGrid}>
                    {scopesOptions.map((scope) => (
                      <button
                        key={scope.id}
                        className={styles.scopesAppButton}
                        onClick={() => handleScopesClick(scope.id)}
                        style={{ '--scopes-color': scope.color } as React.CSSProperties}
                      >
                        <span className={styles.scopesAppIcon}>{scope.icon}</span>
                        <span className={styles.scopesAppTitle}>{scope.title}</span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );

  const regularScopesContent = (
    <div className={`${styles.scopesSection} ${isScopesExpanded ? styles.expanded : ''} ${isChatExpanded ? styles.hidden : ''}`}>
      {activeScope ? (
        <div className={styles.scopesExpanded}>
          <div className={styles.scopesContent}>
            <div className={styles.scopesHeader}>
              {renderScopeDropdown()}
              <div className={styles.scopesHeaderControls}>
                <button
                  className={styles.expandButton}
                  onClick={() => {
                    const newScopesExpanded = !isScopesExpanded;
                    setIsScopesExpanded(newScopesExpanded);
                    if (isChatExpanded) setIsChatExpanded(false);
                  }}
                  title={isScopesExpanded ? "Exit full screen" : "Expand scopes full screen"}
                >
                  {isScopesExpanded ? '⊟' : '⊞'}
                </button>
              </div>
            </div>
            <div className={styles.scopesContent}>
              {renderScopeContent()}
            </div>
          </div>
        </div>
      ) : (
        <div className={`${styles.scopesScrollContainer} ${isChatExpanded ? styles.hidden : ''}`}>
          <div className={styles.chatHeader}>
            <div className={styles.chatTitle}>
              <div className={styles.scopeDropdownContainer} ref={scopeDropdownRef}>
                <button
                  className={styles.scopeDropdownButtonAsTitle}
                  onClick={() => setShowScopeDropdown(!showScopeDropdown)}
                >
                  <span className={styles.scopeDropdownIcon}>🎯</span>
                  <span className={styles.scopeDropdownTitle}>Scopes</span>
                  <span className={styles.scopeDropdownArrow}>{showScopeDropdown ? '▲' : '▼'}</span>
                </button>

                {showScopeDropdown && (
                  <div className={styles.scopeDropdownMenu}>
                    {scopesOptions.map((scope) => (
                      <button
                        key={scope.id}
                        className={`${styles.scopeDropdownItem} ${activeScope === scope.id ? styles.active : ''}`}
                        onClick={() => {
                          handleScopesClick(scope.id);
                          setShowScopeDropdown(false);
                        }}
                        style={{ '--scope-color': scope.color } as React.CSSProperties}
                      >
                        <span className={styles.scopeItemIcon}>{scope.icon}</span>
                        <div className={styles.scopeItemContent}>
                          <span className={styles.scopeItemTitle}>{scope.title}</span>
                          <span className={styles.scopeItemDescription}>{scope.description}</span>
                        </div>
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <div className={styles.tooltipContainer}>
                <div className={styles.helpIcon}>?</div>
                <div className={styles.tooltip}>
                  <div className={styles.tooltipContent}>
                    <h4>Scopes</h4>
                    <p>
                      Scopes are focused work environments, each tailored for specific tasks
                      like code generation, data analysis, automation, and more. Select a
                      scope to access its specialized toolset.
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <div className={styles.chatHeaderControls}>
              <button
                className={styles.expandButton}
                onClick={() => {
                  const newScopesExpanded = !isScopesExpanded;
                  setIsScopesExpanded(newScopesExpanded);
                  if (isChatExpanded) setIsChatExpanded(false);
                }}
                title={isScopesExpanded ? "Exit full screen" : "Expand scopes full screen"}
              >
                {isScopesExpanded ? '⊟' : '⊞'}
              </button>
            </div>
          </div>
          <div className={styles.scopesInfoPanel}>
            <div className={styles.scopesInfoContent}>
              <div className={styles.scopesAppsSection}>
                <div className={styles.scopesAppsGrid}>
                  {scopesOptions.map((scope) => (
                    <button
                      key={scope.id}
                      className={styles.scopesAppButton}
                      onClick={() => handleScopesClick(scope.id)}
                      style={{ '--scopes-color': scope.color } as React.CSSProperties}
                    >
                      <span className={styles.scopesAppIcon}>{scope.icon}</span>
                      <span className={styles.scopesAppTitle}>{scope.title}</span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {!activeScope && renderAvatar()}
    </div>
  );

  return (
    <>
      {isScopesExpanded && expandedScopesContent}
      {regularScopesContent}
    </>
  );
};

export default Scopes;
