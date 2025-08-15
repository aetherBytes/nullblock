"""
Goal-driven workflow orchestration engine
"""

import logging
import asyncio
import json
from typing import Dict, List, Optional, Any, Callable, Union
from datetime import datetime, timedelta
from enum import Enum
import uuid
from dataclasses import dataclass, field
from pydantic import BaseModel, Field
import networkx as nx
from croniter import croniter

logger = logging.getLogger(__name__)


class TaskStatus(Enum):
    """Task execution status"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"
    RETRYING = "retrying"


class WorkflowStatus(Enum):
    """Workflow execution status"""
    CREATED = "created"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    PAUSED = "paused"
    CANCELLED = "cancelled"


class TaskPriority(Enum):
    """Task priority levels"""
    LOW = 1
    NORMAL = 2
    HIGH = 3
    CRITICAL = 4


class Goal(BaseModel):
    """Goal definition for workflow orchestration"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    description: str = Field(..., description="Human-readable goal description")
    target_metric: str = Field(..., description="Metric to optimize (profit, yield, efficiency)")
    target_value: float = Field(..., description="Target value for the metric")
    constraints: Dict[str, Any] = Field(default_factory=dict, description="Goal constraints")
    deadline: Optional[datetime] = Field(None, description="Goal deadline")
    priority: TaskPriority = Field(default=TaskPriority.NORMAL)
    created_at: datetime = Field(default_factory=datetime.now)
    
    class Config:
        use_enum_values = True


class Task(BaseModel):
    """Individual task within a workflow"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str = Field(..., description="Task name")
    description: str = Field(..., description="Task description")
    agent_type: str = Field(..., description="Type of agent to execute task")
    parameters: Dict[str, Any] = Field(default_factory=dict)
    dependencies: List[str] = Field(default_factory=list, description="Task IDs this task depends on")
    status: TaskStatus = Field(default=TaskStatus.PENDING)
    priority: TaskPriority = Field(default=TaskPriority.NORMAL)
    retry_count: int = Field(default=0)
    max_retries: int = Field(default=3)
    timeout_seconds: int = Field(default=300)
    result: Optional[Any] = Field(None)
    error: Optional[str] = Field(None)
    started_at: Optional[datetime] = Field(None)
    completed_at: Optional[datetime] = Field(None)
    
    class Config:
        use_enum_values = True


class Workflow(BaseModel):
    """Workflow containing multiple coordinated tasks"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str = Field(..., description="Workflow name")
    description: str = Field(..., description="Workflow description")
    goal: Goal = Field(..., description="Primary goal this workflow achieves")
    tasks: List[Task] = Field(default_factory=list)
    status: WorkflowStatus = Field(default=WorkflowStatus.CREATED)
    user_id: str = Field(..., description="User who created the workflow")
    schedule: Optional[str] = Field(None, description="Cron expression for scheduled execution")
    context: Dict[str, Any] = Field(default_factory=dict, description="Workflow context data")
    metadata: Dict[str, Any] = Field(default_factory=dict)
    created_at: datetime = Field(default_factory=datetime.now)
    started_at: Optional[datetime] = Field(None)
    completed_at: Optional[datetime] = Field(None)
    
    class Config:
        use_enum_values = True
    
    def add_task(self, task: Task) -> None:
        """Add task to workflow"""
        self.tasks.append(task)
    
    def get_task(self, task_id: str) -> Optional[Task]:
        """Get task by ID"""
        for task in self.tasks:
            if task.id == task_id:
                return task
        return None
    
    def get_ready_tasks(self) -> List[Task]:
        """Get tasks that are ready to run (dependencies completed)"""
        ready_tasks = []
        
        for task in self.tasks:
            if task.status != TaskStatus.PENDING:
                continue
            
            # Check if all dependencies are completed
            dependencies_met = True
            for dep_id in task.dependencies:
                dep_task = self.get_task(dep_id)
                if not dep_task or dep_task.status != TaskStatus.COMPLETED:
                    dependencies_met = False
                    break
            
            if dependencies_met:
                ready_tasks.append(task)
        
        # Sort by priority
        ready_tasks.sort(key=lambda t: t.priority.value, reverse=True)
        return ready_tasks
    
    def is_completed(self) -> bool:
        """Check if all tasks are completed"""
        return all(task.status in [TaskStatus.COMPLETED, TaskStatus.CANCELLED] for task in self.tasks)
    
    def has_failed_tasks(self) -> bool:
        """Check if any tasks have failed"""
        return any(task.status == TaskStatus.FAILED for task in self.tasks)


class AgentInterface:
    """Interface for agent execution"""
    
    async def execute_task(self, task: Task, context: Dict[str, Any]) -> Any:
        """Execute a task and return the result"""
        raise NotImplementedError


class WorkflowOrchestrator:
    """Main workflow orchestration engine"""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        self.active_workflows: Dict[str, Workflow] = {}
        self.agents: Dict[str, AgentInterface] = {}
        self.task_queue: asyncio.Queue = asyncio.Queue()
        self.worker_count = 4
        self.workers: List[asyncio.Task] = []
        self.running = False
        
        # Workflow templates for common patterns
        self.workflow_templates = {
            "arbitrage": self._create_arbitrage_template,
            "defi_yield": self._create_defi_template,
            "nft_trading": self._create_nft_template,
            "dao_governance": self._create_dao_template
        }
    
    def register_agent(self, agent_type: str, agent: AgentInterface):
        """Register an agent for task execution"""
        self.agents[agent_type] = agent
        self.logger.info(f"Registered agent: {agent_type}")
    
    async def start(self):
        """Start the orchestration engine"""
        if self.running:
            return
        
        self.running = True
        
        # Start worker tasks
        for i in range(self.worker_count):
            worker = asyncio.create_task(self._worker(f"worker-{i}"))
            self.workers.append(worker)
        
        self.logger.info(f"Started orchestration engine with {self.worker_count} workers")
    
    async def stop(self):
        """Stop the orchestration engine"""
        if not self.running:
            return
        
        self.running = False
        
        # Cancel all workers
        for worker in self.workers:
            worker.cancel()
        
        # Wait for workers to finish
        await asyncio.gather(*self.workers, return_exceptions=True)
        self.workers.clear()
        
        self.logger.info("Stopped orchestration engine")
    
    async def submit_workflow(self, workflow: Workflow) -> str:
        """Submit workflow for execution"""
        workflow.status = WorkflowStatus.RUNNING
        workflow.started_at = datetime.now()
        
        self.active_workflows[workflow.id] = workflow
        
        # Queue ready tasks
        ready_tasks = workflow.get_ready_tasks()
        for task in ready_tasks:
            await self.task_queue.put((workflow.id, task.id))
        
        self.logger.info(f"Submitted workflow {workflow.id} with {len(ready_tasks)} ready tasks")
        return workflow.id
    
    async def create_workflow_from_goal(
        self, 
        goal: Goal, 
        user_id: str,
        template_type: Optional[str] = None
    ) -> Workflow:
        """Create workflow from goal using templates"""
        
        if template_type and template_type in self.workflow_templates:
            workflow = self.workflow_templates[template_type](goal, user_id)
        else:
            # Create basic workflow
            workflow = Workflow(
                name=f"Goal: {goal.description}",
                description=f"Workflow to achieve: {goal.description}",
                goal=goal,
                user_id=user_id
            )
        
        self.logger.info(f"Created workflow {workflow.id} for goal: {goal.description}")
        return workflow
    
    def _create_arbitrage_template(self, goal: Goal, user_id: str) -> Workflow:
        """Create arbitrage workflow template"""
        workflow = Workflow(
            name="Arbitrage Trading Workflow",
            description="Automated arbitrage trading with MEV protection",
            goal=goal,
            user_id=user_id
        )
        
        # Price monitoring task
        price_task = Task(
            name="Monitor Prices",
            description="Monitor DEX prices for arbitrage opportunities",
            agent_type="price_agent",
            parameters={
                "dexes": ["uniswap", "sushiswap"],
                "tokens": ["ETH", "USDC"],
                "min_profit_threshold": goal.constraints.get("min_profit", 0.01)
            },
            priority=TaskPriority.HIGH
        )
        
        # Strategy task
        strategy_task = Task(
            name="Analyze Strategy",
            description="Analyze and validate arbitrage strategy",
            agent_type="strategy_agent",
            parameters={
                "risk_tolerance": goal.constraints.get("risk_tolerance", "medium")
            },
            dependencies=[price_task.id],
            priority=TaskPriority.HIGH
        )
        
        # Execution task
        execution_task = Task(
            name="Execute Trade",
            description="Execute arbitrage trade with MEV protection",
            agent_type="execution_agent",
            parameters={
                "use_flashbots": True,
                "max_slippage": goal.constraints.get("max_slippage", 0.005)
            },
            dependencies=[strategy_task.id],
            priority=TaskPriority.CRITICAL
        )
        
        # Reporting task
        reporting_task = Task(
            name="Generate Report",
            description="Generate trade execution report",
            agent_type="reporting_agent",
            parameters={},
            dependencies=[execution_task.id],
            priority=TaskPriority.LOW
        )
        
        workflow.add_task(price_task)
        workflow.add_task(strategy_task)
        workflow.add_task(execution_task)
        workflow.add_task(reporting_task)
        
        return workflow
    
    def _create_defi_template(self, goal: Goal, user_id: str) -> Workflow:
        """Create DeFi yield farming workflow template"""
        workflow = Workflow(
            name="DeFi Yield Optimization",
            description="Automated DeFi yield farming and rebalancing",
            goal=goal,
            user_id=user_id
        )
        
        # Data collection task
        data_task = Task(
            name="Collect Yield Data",
            description="Collect yield rates from DeFi protocols",
            agent_type="data_agent",
            parameters={
                "protocols": ["aave", "compound", "yearn"],
                "min_apy": goal.constraints.get("min_apy", 0.05)
            },
            priority=TaskPriority.NORMAL
        )
        
        # Analysis task
        analysis_task = Task(
            name="Analyze Opportunities",
            description="Analyze yield opportunities and risks",
            agent_type="analysis_agent",
            parameters={
                "risk_profile": goal.constraints.get("risk_profile", "medium")
            },
            dependencies=[data_task.id],
            priority=TaskPriority.HIGH
        )
        
        # Execution task
        execution_task = Task(
            name="Rebalance Portfolio",
            description="Execute portfolio rebalancing",
            agent_type="execution_agent",
            parameters={
                "max_allocation": goal.constraints.get("max_allocation", 0.25)
            },
            dependencies=[analysis_task.id],
            priority=TaskPriority.HIGH
        )
        
        workflow.add_task(data_task)
        workflow.add_task(analysis_task)
        workflow.add_task(execution_task)
        
        return workflow
    
    def _create_nft_template(self, goal: Goal, user_id: str) -> Workflow:
        """Create NFT trading workflow template"""
        workflow = Workflow(
            name="NFT Trading Automation",
            description="Automated NFT trading and bidding",
            goal=goal,
            user_id=user_id
        )
        
        # Market monitoring
        market_task = Task(
            name="Monitor NFT Markets",
            description="Monitor NFT collections and floor prices",
            agent_type="market_agent",
            parameters={
                "collections": goal.constraints.get("collections", []),
                "max_floor_price": goal.constraints.get("max_floor_price", 1.0)
            },
            priority=TaskPriority.NORMAL
        )
        
        # Bidding strategy
        bidding_task = Task(
            name="Execute Bidding Strategy",
            description="Place strategic bids on NFTs",
            agent_type="bidding_agent",
            parameters={
                "bid_percentage": goal.constraints.get("bid_percentage", 0.8)
            },
            dependencies=[market_task.id],
            priority=TaskPriority.HIGH
        )
        
        workflow.add_task(market_task)
        workflow.add_task(bidding_task)
        
        return workflow
    
    def _create_dao_template(self, goal: Goal, user_id: str) -> Workflow:
        """Create DAO governance workflow template"""
        workflow = Workflow(
            name="DAO Governance Automation",
            description="Automated DAO proposal analysis and voting",
            goal=goal,
            user_id=user_id
        )
        
        # Proposal monitoring
        proposal_task = Task(
            name="Monitor Proposals",
            description="Monitor DAO proposals and deadlines",
            agent_type="proposal_agent",
            parameters={
                "daos": goal.constraints.get("daos", []),
                "voting_rules": goal.constraints.get("voting_rules", {})
            },
            priority=TaskPriority.NORMAL
        )
        
        # Voting execution
        voting_task = Task(
            name="Execute Votes",
            description="Execute votes based on user preferences",
            agent_type="voting_agent",
            parameters={},
            dependencies=[proposal_task.id],
            priority=TaskPriority.HIGH
        )
        
        workflow.add_task(proposal_task)
        workflow.add_task(voting_task)
        
        return workflow
    
    async def _worker(self, worker_name: str):
        """Worker task to process workflow tasks"""
        self.logger.info(f"Started worker: {worker_name}")
        
        while self.running:
            try:
                # Get task from queue with timeout
                workflow_id, task_id = await asyncio.wait_for(
                    self.task_queue.get(), timeout=1.0
                )
                
                await self._execute_task(workflow_id, task_id)
                
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                self.logger.error(f"Worker {worker_name} error: {e}")
        
        self.logger.info(f"Worker {worker_name} stopped")
    
    async def _execute_task(self, workflow_id: str, task_id: str):
        """Execute a specific task"""
        workflow = self.active_workflows.get(workflow_id)
        if not workflow:
            self.logger.error(f"Workflow {workflow_id} not found")
            return
        
        task = workflow.get_task(task_id)
        if not task:
            self.logger.error(f"Task {task_id} not found in workflow {workflow_id}")
            return
        
        # Check if agent is available
        agent = self.agents.get(task.agent_type)
        if not agent:
            self.logger.error(f"Agent {task.agent_type} not registered")
            task.status = TaskStatus.FAILED
            task.error = f"Agent {task.agent_type} not available"
            return
        
        # Update task status
        task.status = TaskStatus.RUNNING
        task.started_at = datetime.now()
        
        self.logger.info(f"Executing task {task.name} in workflow {workflow.name}")
        
        try:
            # Execute task with timeout
            result = await asyncio.wait_for(
                agent.execute_task(task, workflow.context),
                timeout=task.timeout_seconds
            )
            
            # Task completed successfully
            task.status = TaskStatus.COMPLETED
            task.result = result
            task.completed_at = datetime.now()
            
            self.logger.info(f"Task {task.name} completed successfully")
            
            # Queue dependent tasks
            await self._queue_dependent_tasks(workflow)
            
        except asyncio.TimeoutError:
            task.status = TaskStatus.FAILED
            task.error = "Task timeout"
            self.logger.error(f"Task {task.name} timed out")
            
        except Exception as e:
            task.status = TaskStatus.FAILED
            task.error = str(e)
            self.logger.error(f"Task {task.name} failed: {e}")
            
            # Retry logic
            if task.retry_count < task.max_retries:
                task.retry_count += 1
                task.status = TaskStatus.RETRYING
                await asyncio.sleep(2 ** task.retry_count)  # Exponential backoff
                await self.task_queue.put((workflow_id, task_id))
                self.logger.info(f"Retrying task {task.name} (attempt {task.retry_count})")
        
        # Check if workflow is complete
        await self._check_workflow_completion(workflow)
    
    async def _queue_dependent_tasks(self, workflow: Workflow):
        """Queue tasks that are now ready to run"""
        ready_tasks = workflow.get_ready_tasks()
        for task in ready_tasks:
            await self.task_queue.put((workflow.id, task.id))
    
    async def _check_workflow_completion(self, workflow: Workflow):
        """Check if workflow is complete and update status"""
        if workflow.is_completed():
            if workflow.has_failed_tasks():
                workflow.status = WorkflowStatus.FAILED
            else:
                workflow.status = WorkflowStatus.COMPLETED
            
            workflow.completed_at = datetime.now()
            self.logger.info(f"Workflow {workflow.name} completed with status: {workflow.status}")
    
    def get_workflow_status(self, workflow_id: str) -> Optional[Dict[str, Any]]:
        """Get workflow status and progress"""
        workflow = self.active_workflows.get(workflow_id)
        if not workflow:
            return None
        
        total_tasks = len(workflow.tasks)
        completed_tasks = sum(1 for task in workflow.tasks if task.status == TaskStatus.COMPLETED)
        failed_tasks = sum(1 for task in workflow.tasks if task.status == TaskStatus.FAILED)
        
        return {
            "workflow_id": workflow.id,
            "name": workflow.name,
            "status": workflow.status,
            "progress": {
                "total_tasks": total_tasks,
                "completed_tasks": completed_tasks,
                "failed_tasks": failed_tasks,
                "progress_percentage": (completed_tasks / total_tasks * 100) if total_tasks > 0 else 0
            },
            "goal": workflow.goal.model_dump(),
            "started_at": workflow.started_at,
            "completed_at": workflow.completed_at
        }
    
    def create_workflow(self, name: str, description: str, goal_description: str, 
                       target_metric: str, target_value: float, user_id: str) -> str:
        """Create a new workflow from parameters"""
        goal = Goal(
            description=goal_description,
            target_metric=target_metric,
            target_value=target_value
        )
        
        workflow = Workflow(
            name=name,
            description=description,
            goal=goal,
            user_id=user_id
        )
        
        self.active_workflows[workflow.id] = workflow
        return workflow.id
    
    def start_workflow(self, workflow_id: str) -> bool:
        """Start a workflow by ID"""
        workflow = self.active_workflows.get(workflow_id)
        if not workflow:
            return False
        
        workflow.status = WorkflowStatus.RUNNING
        workflow.started_at = datetime.now()
        
        # In a full implementation, this would start the async task processing
        # For now, we just update the status
        return True
    
    def run(self):
        """Start the workflow engine (sync version)"""
        self.logger.info("Starting Nullblock Orchestration Engine...")
        
        # Run the async startup
        try:
            asyncio.run(self._run_engine())
        except KeyboardInterrupt:
            self.logger.info("Shutting down Nullblock Orchestration Engine...")
    
    async def _run_engine(self):
        """Main engine loop"""
        # Start the async orchestrator
        await self.start()
        
        self.logger.info("Nullblock Orchestration Engine started successfully")
        
        # Keep running until interrupted
        try:
            while True:
                await asyncio.sleep(1)
        except KeyboardInterrupt:
            await self.stop()
    
    def stop(self):
        """Stop the workflow engine"""
        self.logger.info("Stopping Nullblock Orchestration Engine...")
        self.running = False