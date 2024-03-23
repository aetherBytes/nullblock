import React from 'react';
import Echo from '@components/echo/echo'; // Update the import path as necessary
import baseScreensConfig from '@components/echo/screens-config'; // Update this path
import echoChatScreensConfig from './echo-chat-screens-config'; // Ensure this path is correct

const EchoChat: React.FC = () => {
  const mergedScreensConfig = { ...baseScreensConfig, ...echoChatScreensConfig };

  return <Echo screensConfig={mergedScreensConfig} />;
};

export default EchoChat;

