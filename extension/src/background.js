// Ember Link connection to Helios service
let wsConnection = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 5;
const RECONNECT_DELAY = 5000; // 5 seconds

// Console styling for different status types
const consoleStyles = {
  error: 'background-color: #ff0000; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  warning: 'background-color: #ff9900; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  success: 'background-color: #00cc00; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  info: 'background-color: #0099ff; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  disconnected: 'background-color: #ff0000; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  alert: 'background-color: #ff0000; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  bad: 'background-color: #ff0000; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  connected: 'background-color: #00cc00; color: white; font-weight: bold; padding: 2px 5px; border-radius: 3px;',
  default: 'color: #ffffff;'
};

// Function to establish WebSocket connection to Helios
function connectToHelios() {
  // Close existing connection if any
  if (wsConnection) {
    wsConnection.close();
  }

  // Create new WebSocket connection
  wsConnection = new WebSocket('ws://localhost:8000/ws/aether');

  // Set up event handlers
  wsConnection.onopen = () => {
    console.log('%cConnected to Helios service', consoleStyles.connected);
    reconnectAttempts = 0;
    
    // Send initial browser info
    sendBrowserInfo();
  };

  wsConnection.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      console.log('%cReceived message from Helios:', consoleStyles.info, data);
      
      // Process any commands or requests from Helios
      if (data.command) {
        handleHeliosCommand(data.command);
      }
    } catch (error) {
      console.error('%cError processing message from Helios:', consoleStyles.error, error);
    }
  };

  wsConnection.onclose = () => {
    console.log('%cDisconnected from Helios service', consoleStyles.disconnected);
    
    // Attempt to reconnect if under max attempts
    if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
      reconnectAttempts++;
      console.log(`%cReconnecting in ${RECONNECT_DELAY/1000} seconds (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`, consoleStyles.warning);
      setTimeout(connectToHelios, RECONNECT_DELAY);
    } else {
      console.error('%cMax reconnection attempts reached. Please check if Helios service is running.', consoleStyles.error);
    }
  };

  wsConnection.onerror = (error) => {
    console.error('%cWebSocket error:', consoleStyles.error, error);
  };
}

// Function to send browser information to Helios
function sendBrowserInfo() {
  if (!wsConnection || wsConnection.readyState !== WebSocket.OPEN) {
    console.error('%cCannot send browser info: WebSocket not connected', consoleStyles.error);
    return;
  }

  // Get browser information
  const browserInfo = {
    browserInfo: {
      name: navigator.userAgent,
      version: navigator.appVersion,
      platform: navigator.platform,
      language: navigator.language
    },
    activeTab: null // Will be updated when tab changes
  };

  // Send the browser info
  wsConnection.send(JSON.stringify(browserInfo));
  console.log('%cSent browser info to Helios', consoleStyles.success);
}

// Function to update active tab information
function updateActiveTabInfo(tab) {
  if (!wsConnection || wsConnection.readyState !== WebSocket.OPEN) {
    return;
  }

  const tabInfo = {
    activeTab: {
      id: tab.id,
      url: tab.url,
      title: tab.title,
      favIconUrl: tab.favIconUrl
    }
  };

  wsConnection.send(JSON.stringify(tabInfo));
  console.log('%cUpdated active tab info', consoleStyles.info);
}

// Function to handle commands from Helios
function handleHeliosCommand(command) {
  switch (command.type) {
    case 'getTabInfo':
      // Get information about a specific tab
      chrome.tabs.get(command.tabId, (tab) => {
        if (chrome.runtime.lastError) {
          console.error('%cError getting tab info:', consoleStyles.error, chrome.runtime.lastError);
          return;
        }
        updateActiveTabInfo(tab);
      });
      break;
      
    case 'ping':
      // Respond to ping to check connection
      wsConnection.send(JSON.stringify({ type: 'pong' }));
      console.log('%cResponded to ping', consoleStyles.success);
      break;
      
    case 'alert':
      // Handle alert command
      console.log('%cAlert received:', consoleStyles.alert, command.message);
      break;
      
    case 'error':
      // Handle error command
      console.error('%cError received:', consoleStyles.error, command.message);
      break;
      
    default:
      console.warn('%cUnknown command received:', consoleStyles.warning, command);
  }
}

// Listen for tab updates
chrome.tabs.onActivated.addListener((activeInfo) => {
  chrome.tabs.get(activeInfo.tabId, (tab) => {
    if (chrome.runtime.lastError) {
      console.error('%cError getting tab info:', consoleStyles.error, chrome.runtime.lastError);
      return;
    }
    updateActiveTabInfo(tab);
  });
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete') {
    updateActiveTabInfo(tab);
  }
});

// Initialize connection when extension is installed or browser starts
chrome.runtime.onInstalled.addListener(() => {
  console.log('%cEmber Link extension installed/updated', consoleStyles.success);
  connectToHelios();
});

// Connect when browser starts
connectToHelios(); 