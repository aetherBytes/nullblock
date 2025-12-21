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

  // Event Handlers - infrastructure for future real event processing
  private async handlePriceChange(_event: TaskEvent): Promise<void> {
    // Reserved for real price feed integration
  }

  private async handleMarketOpportunity(_event: TaskEvent): Promise<void> {
    // Reserved for real market opportunity detection
  }

  private async handleUserInteraction(_event: TaskEvent): Promise<void> {
    // Reserved for real user interaction analysis
  }

  private async handleAgentCompletion(_event: TaskEvent): Promise<void> {
    // Reserved for real agent completion handling
  }

  private async handleSystemAlert(_event: TaskEvent): Promise<void> {
    // Reserved for real system alert handling
  }

  private async handleThresholdBreach(_event: TaskEvent): Promise<void> {
    // Reserved for real threshold monitoring
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

  // Automation Rules - infrastructure for future real automation
  private initializeDefaultRules(): void {
    // No default rules - add real automation rules as needed
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