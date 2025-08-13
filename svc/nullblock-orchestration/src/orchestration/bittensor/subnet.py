"""
Bittensor subnet integration for crowdsourcing agentic tasks
"""

import logging
import json
import asyncio
from typing import Dict, List, Optional, Any, Callable
from datetime import datetime, timedelta
from enum import Enum
import hashlib
from dataclasses import dataclass
from pydantic import BaseModel, Field
import uuid

logger = logging.getLogger(__name__)


class TaskType(Enum):
    """Types of tasks that can be submitted to the subnet"""
    ARBITRAGE_STRATEGY = "arbitrage_strategy"
    DEFI_OPTIMIZATION = "defi_optimization"
    NFT_ANALYSIS = "nft_analysis"
    DAO_PROPOSAL_ANALYSIS = "dao_proposal_analysis"
    RISK_ASSESSMENT = "risk_assessment"
    MARKET_PREDICTION = "market_prediction"


class TaskStatus(Enum):
    """Status of subnet task"""
    SUBMITTED = "submitted"
    ASSIGNED = "assigned"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class ContributorRole(Enum):
    """Role of subnet contributor"""
    VALIDATOR = "validator"
    MINER = "miner"
    REQUESTER = "requester"


@dataclass
class TaskReward:
    """Reward structure for task completion"""
    base_reward: float
    performance_multiplier: float
    total_reward: float
    currency: str = "NULL"  # $NULL tokens


class SubnetTask(BaseModel):
    """Task submitted to Bittensor subnet"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    task_type: TaskType = Field(..., description="Type of task")
    title: str = Field(..., description="Task title")
    description: str = Field(..., description="Detailed task description")
    requirements: Dict[str, Any] = Field(default_factory=dict, description="Task requirements")
    constraints: Dict[str, Any] = Field(default_factory=dict, description="Task constraints")
    reward: TaskReward = Field(..., description="Reward for completion")
    deadline: Optional[datetime] = Field(None, description="Task deadline")
    priority: int = Field(default=1, description="Task priority (1-10)")
    requester_id: str = Field(..., description="ID of task requester")
    status: TaskStatus = Field(default=TaskStatus.SUBMITTED)
    assigned_to: Optional[str] = Field(None, description="Assigned miner ID")
    submissions: List[Dict[str, Any]] = Field(default_factory=list)
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)
    
    class Config:
        use_enum_values = True


class TaskSubmission(BaseModel):
    """Submission for a subnet task"""
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    task_id: str = Field(..., description="ID of the task")
    contributor_id: str = Field(..., description="ID of contributing miner")
    solution: Dict[str, Any] = Field(..., description="Proposed solution")
    confidence_score: float = Field(..., description="Confidence in solution (0-1)")
    computational_proof: Optional[str] = Field(None, description="Proof of computation")
    submitted_at: datetime = Field(default_factory=datetime.now)
    validation_score: Optional[float] = Field(None, description="Validation score from validators")
    reward_earned: Optional[float] = Field(None, description="Reward earned for this submission")


class SubnetValidator:
    """Validator for subnet task quality and scoring"""
    
    def __init__(self, validator_id: str):
        self.validator_id = validator_id
        self.logger = logging.getLogger(f"{__name__}.SubnetValidator")
        
        # Validation criteria for different task types
        self.validation_criteria = {
            TaskType.ARBITRAGE_STRATEGY: {
                "min_profit_threshold": 0.005,  # 0.5% minimum profit
                "max_risk_score": 0.3,
                "required_fields": ["strategy", "execution_plan", "risk_analysis"]
            },
            TaskType.DEFI_OPTIMIZATION: {
                "min_apy_improvement": 0.01,  # 1% APY improvement
                "max_risk_increase": 0.1,
                "required_fields": ["optimization_plan", "yield_projection", "risk_assessment"]
            },
            TaskType.NFT_ANALYSIS: {
                "min_confidence": 0.7,
                "required_fields": ["analysis", "price_prediction", "market_trends"]
            }
        }
    
    async def validate_submission(
        self, 
        task: SubnetTask, 
        submission: TaskSubmission
    ) -> float:
        """Validate a task submission and return quality score (0-1)"""
        try:
            criteria = self.validation_criteria.get(task.task_type, {})
            solution = submission.solution
            
            # Basic validation
            score = 0.0
            max_score = 100.0
            
            # Check required fields
            required_fields = criteria.get("required_fields", [])
            field_score = sum(1 for field in required_fields if field in solution)
            score += (field_score / len(required_fields)) * 30 if required_fields else 30
            
            # Task-specific validation
            if task.task_type == TaskType.ARBITRAGE_STRATEGY:
                score += self._validate_arbitrage_strategy(solution, criteria)
            elif task.task_type == TaskType.DEFI_OPTIMIZATION:
                score += self._validate_defi_optimization(solution, criteria)
            elif task.task_type == TaskType.NFT_ANALYSIS:
                score += self._validate_nft_analysis(solution, criteria)
            else:
                score += 40  # Default score for unknown task types
            
            # Confidence score weight
            confidence_weight = min(submission.confidence_score * 30, 30)
            score += confidence_weight
            
            normalized_score = min(score / max_score, 1.0)
            
            self.logger.info(
                f"Validated submission {submission.id} for task {task.id}: "
                f"score={normalized_score:.3f}"
            )
            
            return normalized_score
            
        except Exception as e:
            self.logger.error(f"Validation failed for submission {submission.id}: {e}")
            return 0.0
    
    def _validate_arbitrage_strategy(
        self, 
        solution: Dict[str, Any], 
        criteria: Dict[str, Any]
    ) -> float:
        """Validate arbitrage strategy solution"""
        score = 0.0
        
        # Check profit threshold
        projected_profit = solution.get("projected_profit", 0)
        min_profit = criteria.get("min_profit_threshold", 0.005)
        if projected_profit >= min_profit:
            score += 20
        
        # Check risk analysis
        risk_score = solution.get("risk_score", 1.0)
        max_risk = criteria.get("max_risk_score", 0.3)
        if risk_score <= max_risk:
            score += 15
        
        # Check execution feasibility
        if "execution_plan" in solution and len(solution["execution_plan"]) > 0:
            score += 5
        
        return score
    
    def _validate_defi_optimization(
        self, 
        solution: Dict[str, Any], 
        criteria: Dict[str, Any]
    ) -> float:
        """Validate DeFi optimization solution"""
        score = 0.0
        
        # Check APY improvement
        apy_improvement = solution.get("apy_improvement", 0)
        min_improvement = criteria.get("min_apy_improvement", 0.01)
        if apy_improvement >= min_improvement:
            score += 25
        
        # Check risk assessment
        risk_increase = solution.get("risk_increase", 1.0)
        max_risk_increase = criteria.get("max_risk_increase", 0.1)
        if risk_increase <= max_risk_increase:
            score += 15
        
        return score
    
    def _validate_nft_analysis(
        self, 
        solution: Dict[str, Any], 
        criteria: Dict[str, Any]
    ) -> float:
        """Validate NFT analysis solution"""
        score = 0.0
        
        # Check analysis quality
        if "analysis" in solution and len(str(solution["analysis"])) > 100:
            score += 20
        
        # Check prediction confidence
        prediction_confidence = solution.get("prediction_confidence", 0)
        min_confidence = criteria.get("min_confidence", 0.7)
        if prediction_confidence >= min_confidence:
            score += 20
        
        return score


class BittensorSubnetClient:
    """Client for interacting with Nullblock Bittensor subnet"""
    
    def __init__(
        self, 
        subnet_id: str = "nullblock_subnet",
        validator_id: Optional[str] = None
    ):
        self.subnet_id = subnet_id
        self.validator_id = validator_id
        self.logger = logging.getLogger(__name__)
        
        # In-memory storage for MVP (replace with actual Bittensor integration)
        self.tasks: Dict[str, SubnetTask] = {}
        self.submissions: Dict[str, List[TaskSubmission]] = {}
        
        # Initialize validator if ID provided
        self.validator = SubnetValidator(validator_id) if validator_id else None
        
        # Task assignment queue
        self.task_queue: asyncio.Queue = asyncio.Queue()
        
        # Contributors (miners and validators)
        self.contributors: Dict[str, Dict[str, Any]] = {}
        
        self.logger.info(f"Initialized Bittensor subnet client: {subnet_id}")
    
    async def submit_task(self, task: SubnetTask) -> str:
        """Submit a task to the subnet"""
        try:
            # Calculate reward based on task complexity and priority
            base_reward = self._calculate_base_reward(task)
            task.reward = TaskReward(
                base_reward=base_reward,
                performance_multiplier=1.0,
                total_reward=base_reward
            )
            
            # Store task
            self.tasks[task.id] = task
            self.submissions[task.id] = []
            
            # Add to assignment queue
            await self.task_queue.put(task.id)
            
            self.logger.info(
                f"Submitted task {task.id} ({task.task_type.value}) "
                f"with reward {task.reward.total_reward} $NULL"
            )
            
            return task.id
            
        except Exception as e:
            self.logger.error(f"Failed to submit task: {e}")
            raise
    
    def _calculate_base_reward(self, task: SubnetTask) -> float:
        """Calculate base reward for task"""
        # Base reward structure
        base_rewards = {
            TaskType.ARBITRAGE_STRATEGY: 100.0,
            TaskType.DEFI_OPTIMIZATION: 150.0,
            TaskType.NFT_ANALYSIS: 75.0,
            TaskType.DAO_PROPOSAL_ANALYSIS: 50.0,
            TaskType.RISK_ASSESSMENT: 80.0,
            TaskType.MARKET_PREDICTION: 120.0
        }
        
        base = base_rewards.get(task.task_type, 100.0)
        
        # Priority multiplier
        priority_multiplier = 1.0 + (task.priority - 1) * 0.2
        
        # Deadline multiplier (urgent tasks get higher rewards)
        deadline_multiplier = 1.0
        if task.deadline:
            hours_until_deadline = (task.deadline - datetime.now()).total_seconds() / 3600
            if hours_until_deadline < 24:
                deadline_multiplier = 1.5
            elif hours_until_deadline < 72:
                deadline_multiplier = 1.2
        
        return base * priority_multiplier * deadline_multiplier
    
    async def assign_task(self, task_id: str, miner_id: str) -> bool:
        """Assign task to a miner"""
        try:
            task = self.tasks.get(task_id)
            if not task:
                return False
            
            if task.status != TaskStatus.SUBMITTED:
                return False
            
            task.assigned_to = miner_id
            task.status = TaskStatus.ASSIGNED
            task.updated_at = datetime.now()
            
            self.logger.info(f"Assigned task {task_id} to miner {miner_id}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to assign task {task_id}: {e}")
            return False
    
    async def submit_solution(
        self, 
        task_id: str, 
        miner_id: str, 
        solution: Dict[str, Any],
        confidence_score: float
    ) -> str:
        """Submit solution for a task"""
        try:
            task = self.tasks.get(task_id)
            if not task:
                raise ValueError(f"Task {task_id} not found")
            
            submission = TaskSubmission(
                task_id=task_id,
                contributor_id=miner_id,
                solution=solution,
                confidence_score=confidence_score
            )
            
            # Add to submissions
            if task_id not in self.submissions:
                self.submissions[task_id] = []
            self.submissions[task_id].append(submission)
            
            # Update task status
            task.status = TaskStatus.IN_PROGRESS
            task.updated_at = datetime.now()
            
            self.logger.info(f"Received solution {submission.id} for task {task_id}")
            
            # Trigger validation if validator available
            if self.validator:
                await self._validate_and_reward(task, submission)
            
            return submission.id
            
        except Exception as e:
            self.logger.error(f"Failed to submit solution: {e}")
            raise
    
    async def _validate_and_reward(self, task: SubnetTask, submission: TaskSubmission):
        """Validate submission and calculate reward"""
        try:
            # Validate submission
            validation_score = await self.validator.validate_submission(task, submission)
            submission.validation_score = validation_score
            
            # Calculate reward based on validation score
            base_reward = task.reward.base_reward
            performance_multiplier = validation_score
            earned_reward = base_reward * performance_multiplier
            
            submission.reward_earned = earned_reward
            
            # Update task reward
            task.reward.performance_multiplier = performance_multiplier
            task.reward.total_reward = earned_reward
            
            # Mark task as completed if validation passes threshold
            if validation_score >= 0.7:  # 70% threshold
                task.status = TaskStatus.COMPLETED
                
                self.logger.info(
                    f"Task {task.id} completed successfully. "
                    f"Reward: {earned_reward:.2f} $NULL"
                )
            else:
                self.logger.warning(
                    f"Submission {submission.id} failed validation "
                    f"(score: {validation_score:.3f})"
                )
            
            task.updated_at = datetime.now()
            
        except Exception as e:
            self.logger.error(f"Validation failed for submission {submission.id}: {e}")
    
    def get_task_status(self, task_id: str) -> Optional[Dict[str, Any]]:
        """Get task status and submissions"""
        task = self.tasks.get(task_id)
        if not task:
            return None
        
        submissions = self.submissions.get(task_id, [])
        
        return {
            "task": task.model_dump(),
            "submissions": [sub.model_dump() for sub in submissions],
            "submission_count": len(submissions)
        }
    
    def get_available_tasks(self, miner_id: str) -> List[Dict[str, Any]]:
        """Get available tasks for a miner"""
        available_tasks = []
        
        for task in self.tasks.values():
            if (task.status == TaskStatus.SUBMITTED and 
                (not task.deadline or task.deadline > datetime.now())):
                available_tasks.append({
                    "id": task.id,
                    "task_type": task.task_type,
                    "title": task.title,
                    "description": task.description,
                    "reward": task.reward.total_reward,
                    "deadline": task.deadline,
                    "priority": task.priority
                })
        
        # Sort by priority and reward
        available_tasks.sort(
            key=lambda t: (t["priority"], t["reward"]), 
            reverse=True
        )
        
        return available_tasks
    
    def get_contributor_stats(self, contributor_id: str) -> Dict[str, Any]:
        """Get statistics for a contributor (miner or validator)"""
        completed_tasks = 0
        total_reward = 0.0
        avg_validation_score = 0.0
        validation_scores = []
        
        for task_submissions in self.submissions.values():
            for submission in task_submissions:
                if submission.contributor_id == contributor_id:
                    if submission.reward_earned:
                        completed_tasks += 1
                        total_reward += submission.reward_earned
                    
                    if submission.validation_score:
                        validation_scores.append(submission.validation_score)
        
        if validation_scores:
            avg_validation_score = sum(validation_scores) / len(validation_scores)
        
        return {
            "contributor_id": contributor_id,
            "completed_tasks": completed_tasks,
            "total_reward": total_reward,
            "average_validation_score": avg_validation_score,
            "success_rate": avg_validation_score if validation_scores else 0.0
        }
    
    async def create_arbitrage_task(
        self, 
        requester_id: str,
        min_profit: float = 0.01,
        max_risk: float = 0.3,
        preferred_dexes: List[str] = None,
        priority: int = 5
    ) -> str:
        """Create arbitrage strategy task"""
        
        if preferred_dexes is None:
            preferred_dexes = ["uniswap", "sushiswap"]
        
        task = SubnetTask(
            task_type=TaskType.ARBITRAGE_STRATEGY,
            title="Develop Arbitrage Trading Strategy",
            description=f"Create an arbitrage trading strategy with minimum {min_profit*100}% profit and maximum {max_risk*100}% risk",
            requirements={
                "min_profit_threshold": min_profit,
                "max_risk_score": max_risk,
                "preferred_dexes": preferred_dexes,
                "include_gas_costs": True,
                "mev_protection": True
            },
            constraints={
                "max_trade_size": 10000,  # USD
                "supported_tokens": ["ETH", "USDC", "USDT", "DAI"],
                "execution_time_limit": 300  # seconds
            },
            priority=priority,
            requester_id=requester_id,
            deadline=datetime.now() + timedelta(hours=24)
        )
        
        return await self.submit_task(task)
    
    async def create_defi_optimization_task(
        self,
        requester_id: str,
        current_apy: float,
        risk_tolerance: str = "medium",
        priority: int = 3
    ) -> str:
        """Create DeFi yield optimization task"""
        
        task = SubnetTask(
            task_type=TaskType.DEFI_OPTIMIZATION,
            title="Optimize DeFi Yield Strategy",
            description=f"Optimize yield farming strategy to beat current {current_apy*100}% APY with {risk_tolerance} risk tolerance",
            requirements={
                "current_apy": current_apy,
                "risk_tolerance": risk_tolerance,
                "min_apy_improvement": 0.01,
                "include_gas_costs": True
            },
            constraints={
                "supported_protocols": ["aave", "compound", "yearn", "convex"],
                "max_risk_increase": 0.1,
                "portfolio_size": 50000  # USD
            },
            priority=priority,
            requester_id=requester_id,
            deadline=datetime.now() + timedelta(hours=48)
        )
        
        return await self.submit_task(task)