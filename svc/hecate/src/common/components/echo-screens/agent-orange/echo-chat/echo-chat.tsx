import React from 'react';
import Echo from '@components/echo/echo'; // Adjust the import path as necessary
import baseScreensConfig from '@components/echo/screens-config'; // Update this path
import echoChatScreensConfig from './echo-chat-screens-config'; // Ensure this path is correct

const EchoChat: React.FC = () => {
  const mergedScreensConfig = { ...baseScreensConfig, ...echoChatScreensConfig };
  const defaultScreenKey = Object.keys(echoChatScreensConfig)[0]; // First key from the child config

  return <Echo screensConfig={mergedScreensConfig} defaultScreenKey={defaultScreenKey} />;
};

export default EchoChat;


