/**
 * Centralized logging system for Hecate frontend
 * Provides console logging with optional file persistence
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

export interface LogEntry {
  timestamp: Date;
  level: LogLevel;
  component: string;
  message: string;
  data?: any;
}

class Logger {
  private logs: LogEntry[] = [];
  private maxLogs = 1000;
  private logLevel = LogLevel.INFO;

  constructor() {
    // Set log level from environment
    const envLevel = import.meta.env.VITE_LOG_LEVEL || 'INFO';
    this.logLevel = LogLevel[envLevel as keyof typeof LogLevel] || LogLevel.INFO;
    
    // Setup periodic log persistence
    if (typeof window !== 'undefined') {
      this.setupLogPersistence();
    }
  }

  private setupLogPersistence() {
    // Save logs to localStorage every 30 seconds
    setInterval(() => {
      try {
        const recentLogs = this.logs.slice(-100); // Keep last 100 entries
        localStorage.setItem('hecate_logs', JSON.stringify(recentLogs));
      } catch (error) {
        console.warn('Failed to persist logs to localStorage:', error);
      }
    }, 30000);

    // Load existing logs on startup
    try {
      const stored = localStorage.getItem('hecate_logs');
      if (stored) {
        const parsedLogs = JSON.parse(stored);
        this.logs = parsedLogs.map((log: any) => ({
          ...log,
          timestamp: new Date(log.timestamp)
        }));
      }
    } catch (error) {
      console.warn('Failed to load persisted logs:', error);
    }
  }

  private addLog(level: LogLevel, component: string, message: string, data?: any) {
    if (level < this.logLevel) return;

    const entry: LogEntry = {
      timestamp: new Date(),
      level,
      component,
      message,
      data
    };

    this.logs.push(entry);
    
    // Keep only recent logs
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }

    // Console output with styling
    this.outputToConsole(entry);
  }

  private outputToConsole(entry: LogEntry) {
    const timestamp = entry.timestamp.toISOString().substr(11, 12);
    const levelStr = LogLevel[entry.level].padEnd(5);
    const prefix = `[${timestamp}] [HECATE] ${levelStr} [${entry.component}]`;

    const styles = {
      [LogLevel.DEBUG]: 'color: #888',
      [LogLevel.INFO]: 'color: #00ffff',
      [LogLevel.WARN]: 'color: #ffff00',
      [LogLevel.ERROR]: 'color: #ff4444; font-weight: bold'
    };

    const style = styles[entry.level];

    if (entry.data) {
      console.log(`%c${prefix} ${entry.message}`, style, entry.data);
    } else {
      console.log(`%c${prefix} ${entry.message}`, style);
    }
  }

  debug(component: string, message: string, data?: any) {
    this.addLog(LogLevel.DEBUG, component, message, data);
  }

  info(component: string, message: string, data?: any) {
    this.addLog(LogLevel.INFO, component, message, data);
  }

  warn(component: string, message: string, data?: any) {
    this.addLog(LogLevel.WARN, component, message, data);
  }

  error(component: string, message: string, data?: any) {
    this.addLog(LogLevel.ERROR, component, message, data);
  }

  // Get logs for debugging/export
  getLogs(since?: Date): LogEntry[] {
    if (!since) return [...this.logs];
    return this.logs.filter(log => log.timestamp >= since);
  }

  // Get logs from last N minutes
  getRecentLogs(minutes: number = 15): LogEntry[] {
    const since = new Date(Date.now() - minutes * 60 * 1000);
    return this.getLogs(since);
  }

  // Export logs as text
  exportLogs(since?: Date): string {
    const logs = this.getLogs(since);
    return logs.map(log => 
      `${log.timestamp.toISOString()} [${LogLevel[log.level]}] [${log.component}] ${log.message}${
        log.data ? ' ' + JSON.stringify(log.data) : ''
      }`
    ).join('\n');
  }

  // Clear all logs
  clearLogs() {
    this.logs = [];
    if (typeof window !== 'undefined') {
      localStorage.removeItem('hecate_logs');
    }
  }
}

// Global logger instance
export const logger = new Logger();

// Convenience functions for different components
export const createComponentLogger = (component: string) => ({
  debug: (message: string, data?: any) => logger.debug(component, message, data),
  info: (message: string, data?: any) => logger.info(component, message, data),
  warn: (message: string, data?: any) => logger.warn(component, message, data),
  error: (message: string, data?: any) => logger.error(component, message, data),
});

// Auto-log startup
if (typeof window !== 'undefined') {
  logger.info('Frontend', '🎨 Hecate frontend logging initialized');
  logger.info('Frontend', `📝 Log level: ${LogLevel[logger['logLevel']]}`);
  logger.info('Frontend', `💾 Persisting logs to localStorage`);
}