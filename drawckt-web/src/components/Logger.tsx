import React, { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import './Logger.css';

interface LogEntry {
  level: string;
  message: string;
  timestamp: Date;
}

const Logger: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isMinimized, setIsMinimized] = useState(true);
  const [settingsPanelContainer, setSettingsPanelContainer] = useState<HTMLElement | null>(null);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const minimizedLoggerRef = useRef<HTMLDivElement>(null);
  const expandedLoggerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Intercept console methods to capture logs
    const originalLog = console.log;
    const originalInfo = console.info;
    const originalWarn = console.warn;
    const originalError = console.error;

    const addLog = (level: string, ...args: any[]) => {
      const message = args.map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
      ).join(' ');
      
      setLogs(prev => {
        const newLogs = [...prev, {
          level,
          message,
          timestamp: new Date(),
        }];
        // Keep only last 1000 logs
        return newLogs.slice(-1000);
      });
    };

    console.log = (...args) => {
      originalLog(...args);
      addLog('log', ...args);
    };

    console.info = (...args) => {
      originalInfo(...args);
      addLog('info', ...args);
    };

    console.warn = (...args) => {
      originalWarn(...args);
      addLog('warn', ...args);
    };

    console.error = (...args) => {
      originalError(...args);
      addLog('error', ...args);
    };

    return () => {
      console.log = originalLog;
      console.info = originalInfo;
      console.warn = originalWarn;
      console.error = originalError;
    };
  }, []);

  useEffect(() => {
    // Auto-scroll to bottom when new logs arrive
    if (logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs]);

  // Find settings-panel container for minimized logger
  // Place it in .panel.settings-panel (not .settings-panel content area) so it stays fixed at bottom
  useEffect(() => {
    const findContainer = () => {
      // Target the outer .panel.settings-panel container, not the scrollable content area
      const container = document.querySelector('.panel.settings-panel') as HTMLElement;
      setSettingsPanelContainer(container);
    };

    // Initial find
    findContainer();

    // Watch for container to appear/disappear
    const observer = new MutationObserver(findContainer);
    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });

    // Also watch for window resize/layout changes
    window.addEventListener('resize', findContainer);

    return () => {
      observer.disconnect();
      window.removeEventListener('resize', findContainer);
    };
  }, []);

  // Handle click outside and ESC key when expanded
  useEffect(() => {
    if (!isMinimized) {
      // When expanded, clicking outside logger or pressing ESC should minimize
      const handleClickOutside = (event: MouseEvent) => {
        if (expandedLoggerRef.current && !expandedLoggerRef.current.contains(event.target as Node)) {
          setIsMinimized(true);
        }
      };

      const handleKeyDown = (event: KeyboardEvent) => {
        if (event.key === 'Escape') {
          setIsMinimized(true);
        }
      };

      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);

      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
        document.removeEventListener('keydown', handleKeyDown);
      };
    }
  }, [isMinimized]);

  const getLogLevelClass = (level: string) => {
    switch (level) {
      case 'error': return 'log-error';
      case 'warn': return 'log-warn';
      case 'info': return 'log-info';
      default: return 'log-log';
    }
  };

  const clearLogs = () => {
    setLogs([]);
  };

  // Render both minimized and expanded loggers, control visibility with CSS
  const minimizedLogger = (
    <div 
      className={`logger minimized ${isMinimized ? 'visible' : 'hidden'}`}
      ref={minimizedLoggerRef}
      onClick={() => setIsMinimized(false)}
      style={{ cursor: 'pointer' }}
    >
      <button
        className="logger-toggle"
        title="Show logger"
      >
        📋 {logs.length > 0 && <span className="log-count">{logs.length}</span>}
      </button>
    </div>
  );

  const expandedLogger = (
    <div 
      className={`logger expanded ${isMinimized ? 'hidden' : 'visible'}`}
      ref={expandedLoggerRef}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="logger-header">
        <div className="logger-title">Logger</div>
        <div className="logger-actions">
          <button className="clear-btn" onClick={clearLogs} title="Clear logs">
            Clear
          </button>
          <button
            className="minimize-btn"
            onClick={() => setIsMinimized(true)}
            title="Minimize"
          >
            ✕
          </button>
        </div>
      </div>
      <div className="logger-content">
        {logs.length === 0 ? (
          <div className="logger-empty">No logs yet</div>
        ) : (
          logs.map((log, index) => (
            <div key={index} className={`log-entry ${getLogLevelClass(log.level)}`}>
              <span className="log-time">
                {log.timestamp.toLocaleTimeString()}
              </span>
              <span className="log-level">{log.level.toUpperCase()}</span>
              <span className="log-message">{log.message}</span>
            </div>
          ))
        )}
        <div ref={logsEndRef} />
      </div>
    </div>
  );

  return (
    <>
      {/* Minimized logger - portal to settings-panel if available, otherwise hide */}
      {settingsPanelContainer && createPortal(minimizedLogger, settingsPanelContainer)}
      {/* Expanded logger - always at root level */}
      {expandedLogger}
    </>
  );
};

export default Logger;

