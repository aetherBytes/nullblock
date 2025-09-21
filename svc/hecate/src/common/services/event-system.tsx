import { TaskEvent, TaskCreationRequest, EventType } from '../../types/tasks';

export interface EventSubscription {
  id: string;
  eventType: EventType;
  callback: (event: TaskEvent) => void;
  filter?: (event: TaskEvent) => boolean;
}

export interface EventRule {
  id: string;
  name: string;
  description: string;
  eventType: EventType;
  condition: (event: TaskEvent) => boolean;
  action: (event: TaskEvent) => Promise<TaskCreationRequest | void>;
  enabled: boolean;
  priority: number;
  cooldown: number;
  lastTriggered?: Date;
}

class EventSystem {
  private subscriptions: Map<string, EventSubscription> = new Map();
  private eventQueue: TaskEvent[] = [];
  private processing: boolean = false;
  private rules: Map<string, EventRule> = new Map();

  constructor() {
    this.initializeDefaultRules();
  }

  // Event Publishing
  async publishEvent(event: Omit<TaskEvent, 'id'>): Promise<void> {
    const fullEvent: TaskEvent = {
      ...event,
      id: this.generateEventId(),
      timestamp: new Date(),
      processed: false
    };

    this.eventQueue.push(fullEvent);
    console.log(`📢 Event published: ${fullEvent.type} from ${fullEvent.source}`);

    // Process the event asynchronously
    this.processEvents();

    // Notify subscribers
    this.notifySubscribers(fullEvent);

    // Check automation rules
    this.checkAutomationRules(fullEvent);
  }

  // Event Subscription
  subscribe(subscription: Omit<EventSubscription, 'id'>): string {
    const id = this.generateSubscriptionId();
    const fullSubscription: EventSubscription = {
      ...subscription,
      id
    };

    this.subscriptions.set(id, fullSubscription);
    console.log(`🔔 New subscription: ${subscription.eventType} (${id})`);
    return id;
  }

  unsubscribe(subscriptionId: string): boolean {
    const result = this.subscriptions.delete(subscriptionId);
    if (result) {
      console.log(`🔕 Unsubscribed: ${subscriptionId}`);
    }
    return result;
  }

  // Event Processing
  private async processEvents(): Promise<void> {
    if (this.processing || this.eventQueue.length === 0) {
      return;
    }

    this.processing = true;

    try {
      while (this.eventQueue.length > 0) {
        const event = this.eventQueue.shift();
        if (event && !event.processed) {
          await this.processEvent(event);
          event.processed = true;
        }
      }
    } catch (error) {
      console.error('❌ Error processing events:', error);
    } finally {
      this.processing = false;
    }
  }

  private async processEvent(event: TaskEvent): Promise<void> {
    console.log(`⚡ Processing event: ${event.type} - ${event.data?.message || 'No message'}`);

    // Add event-specific processing logic here
    switch (event.type) {
      case 'price_change':
        await this.handlePriceChange(event);
        break;
      case 'market_opportunity':
        await this.handleMarketOpportunity(event);
        break;
      case 'user_interaction':
        await this.handleUserInteraction(event);
        break;
      case 'agent_completion':
        await this.handleAgentCompletion(event);
        break;
      case 'system_alert':
        await this.handleSystemAlert(event);
        break;
      case 'threshold_breach':
        await this.handleThresholdBreach(event);
        break;
      default:
        console.log(`ℹ️ No specific handler for event type: ${event.type}`);
    }
  }

  // Event Handlers
  private async handlePriceChange(event: TaskEvent): Promise<void> {
    const { symbol, price, change } = event.data;
    console.log(`💰 Price change: ${symbol} now at ${price} (${change})`);

    // Example: Create arbitrage task if significant price movement
    if (Math.abs(parseFloat(change.replace('%', ''))) > 5) {
      await this.publishEvent({
        type: 'market_opportunity',
        data: {
          type: 'arbitrage',
          symbol,
          price,
          change,
          urgency: 'high'
        },
        source: 'price_monitor',
        timestamp: new Date(),
        processed: false
      });
    }
  }

  private async handleMarketOpportunity(event: TaskEvent): Promise<void> {
    const { type, urgency } = event.data;
    console.log(`📈 Market opportunity detected: ${type} (${urgency})`);

    // Auto-create tasks for high urgency opportunities
    if (urgency === 'high') {
      // This would trigger task creation through the task service
      console.log(`🚀 Auto-creating task for ${type} opportunity`);
    }
  }

  private async handleUserInteraction(event: TaskEvent): Promise<void> {
    const { action, context } = event.data;
    console.log(`👤 User interaction: ${action}`);

    // Update motivation state based on user interactions
    if (action === 'chat_message') {
      // Analyze user intent and suggest tasks
      console.log('🧠 Analyzing user intent for task suggestions');
    }
  }

  private async handleAgentCompletion(event: TaskEvent): Promise<void> {
    const { agentName, taskId, result } = event.data;
    console.log(`🤖 Agent ${agentName} completed task ${taskId}: ${result}`);

    // Trigger follow-up tasks if needed
    if (result === 'success') {
      console.log('✅ Checking for follow-up tasks');
    }
  }

  private async handleSystemAlert(event: TaskEvent): Promise<void> {
    const { level, message } = event.data;
    console.log(`🚨 System alert (${level}): ${message}`);

    // Create maintenance or monitoring tasks for critical alerts
    if (level === 'critical') {
      console.log('🔧 Creating emergency response task');
    }
  }

  private async handleThresholdBreach(event: TaskEvent): Promise<void> {
    const { metric, value, threshold } = event.data;
    console.log(`⚠️ Threshold breach: ${metric} = ${value} (threshold: ${threshold})`);

    // Auto-create rebalancing or adjustment tasks
    console.log('📊 Creating adjustment task for threshold breach');
  }

  // Subscriber Notification
  private notifySubscribers(event: TaskEvent): void {
    for (const subscription of this.subscriptions.values()) {
      if (subscription.eventType === event.type || subscription.eventType === 'new_data') {
        if (!subscription.filter || subscription.filter(event)) {
          try {
            subscription.callback(event);
          } catch (error) {
            console.error(`❌ Error in event callback for ${subscription.id}:`, error);
          }
        }
      }
    }
  }

  // Automation Rules
  private initializeDefaultRules(): void {
    // Rule: Auto-create arbitrage tasks on price opportunities
    this.addRule({
      id: 'auto_arbitrage',
      name: 'Auto Arbitrage Detection',
      description: 'Automatically create arbitrage tasks when profitable opportunities are detected',
      eventType: 'market_opportunity',
      condition: (event) => event.data.type === 'arbitrage' && event.data.profit > 0.1,
      action: async (event) => {
        return {
          name: `Arbitrage Opportunity: ${event.data.symbol}`,
          description: `Automated arbitrage task for ${event.data.symbol} with ${event.data.profit}% profit potential`,
          type: 'arbitrage',
          category: 'event-triggered',
          priority: 'high',
          parameters: {
            symbol: event.data.symbol,
            expectedProfit: event.data.profit,
            exchanges: event.data.exchanges
          },
          autoStart: true
        };
      },
      enabled: true,
      priority: 1,
      cooldown: 30000 // 30 seconds
    });

    // Rule: Auto-create monitoring tasks for system alerts
    this.addRule({
      id: 'auto_monitoring',
      name: 'Auto System Monitoring',
      description: 'Create monitoring tasks for critical system alerts',
      eventType: 'system_alert',
      condition: (event) => event.data.level === 'critical',
      action: async (event) => {
        return {
          name: `System Monitoring: ${event.data.component}`,
          description: `Monitor and resolve critical issue: ${event.data.message}`,
          type: 'monitoring',
          category: 'event-triggered',
          priority: 'urgent',
          parameters: {
            component: event.data.component,
            issue: event.data.message,
            severity: event.data.level
          },
          autoStart: true
        };
      },
      enabled: true,
      priority: 2,
      cooldown: 60000 // 1 minute
    });
  }

  addRule(rule: EventRule): void {
    this.rules.set(rule.id, rule);
    console.log(`📋 Added automation rule: ${rule.name}`);
  }

  removeRule(ruleId: string): boolean {
    const result = this.rules.delete(ruleId);
    if (result) {
      console.log(`🗑️ Removed automation rule: ${ruleId}`);
    }
    return result;
  }

  private async checkAutomationRules(event: TaskEvent): Promise<void> {
    for (const rule of this.rules.values()) {
      if (!rule.enabled || rule.eventType !== event.type) {
        continue;
      }

      // Check cooldown
      if (rule.lastTriggered) {
        const timeSinceLastTrigger = Date.now() - rule.lastTriggered.getTime();
        if (timeSinceLastTrigger < rule.cooldown) {
          continue;
        }
      }

      // Check condition
      try {
        if (rule.condition(event)) {
          console.log(`🎯 Rule triggered: ${rule.name}`);

          const taskRequest = await rule.action(event);
          if (taskRequest) {
            // This would integrate with the task service to create the task
            console.log(`📋 Rule ${rule.name} would create task: ${taskRequest.name}`);

            // TODO: Integrate with actual task creation service
            // await taskService.createTask(taskRequest);
          }

          rule.lastTriggered = new Date();
        }
      } catch (error) {
        console.error(`❌ Error processing rule ${rule.name}:`, error);
      }
    }
  }

  // Helper Methods
  private generateEventId(): string {
    return `event_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateSubscriptionId(): string {
    return `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  // Public API for getting events and rules
  getRecentEvents(limit: number = 10): TaskEvent[] {
    return this.eventQueue.slice(-limit);
  }

  getRules(): EventRule[] {
    return Array.from(this.rules.values());
  }

  getSubscriptions(): EventSubscription[] {
    return Array.from(this.subscriptions.values());
  }

  // Market Event Helpers (for easy integration)
  publishPriceChange(symbol: string, price: string, change: string): void {
    this.publishEvent({
      type: 'price_change',
      data: { symbol, price, change },
      source: 'market_data',
      timestamp: new Date(),
      processed: false
    });
  }

  publishMarketOpportunity(type: string, data: any): void {
    this.publishEvent({
      type: 'market_opportunity',
      data: { type, ...data },
      source: 'opportunity_scanner',
      timestamp: new Date(),
      processed: false
    });
  }

  publishUserInteraction(action: string, context: any): void {
    this.publishEvent({
      type: 'user_interaction',
      data: { action, context },
      source: 'user_interface',
      timestamp: new Date(),
      processed: false
    });
  }

  publishAgentCompletion(agentName: string, taskId: string, result: string, data?: any): void {
    this.publishEvent({
      type: 'agent_completion',
      data: { agentName, taskId, result, ...data },
      source: `agent_${agentName}`,
      timestamp: new Date(),
      processed: false
    });
  }

  publishSystemAlert(level: string, message: string, component: string, data?: any): void {
    this.publishEvent({
      type: 'system_alert',
      data: { level, message, component, ...data },
      source: 'system_monitor',
      timestamp: new Date(),
      processed: false
    });
  }

  publishThresholdBreach(metric: string, value: number, threshold: number, data?: any): void {
    this.publishEvent({
      type: 'threshold_breach',
      data: { metric, value, threshold, ...data },
      source: 'threshold_monitor',
      timestamp: new Date(),
      processed: false
    });
  }
}

// Singleton instance
export const eventSystem = new EventSystem();
export default EventSystem;