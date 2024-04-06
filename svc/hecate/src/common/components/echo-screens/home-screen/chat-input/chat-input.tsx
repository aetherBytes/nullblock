import React, { useState } from 'react';
import styles from './chat-input.module.scss'; // Make sure to import your CSS module correctly

const ChatInput = () => {
  const [message, setMessage] = useState('');

  const sendTextMessage = () => {
    if (message.trim()) {
      // Trigger the float animation for the message
      const messageContainer = document.createElement('div');
      messageContainer.className = styles.floatingMessage; // Use styles from module
      messageContainer.textContent = message;
      document.body.appendChild(messageContainer);

      // Reset message input
      setMessage('');

      // Remove the element after animation completes
      setTimeout(() => {
        document.body.removeChild(messageContainer);
      }, 3000);
    }
  };

  const sendMessage = (e) => {
    if (e.key === 'Enter') {
      sendTextMessage();
    }
  };

  return (
    <div className={styles.chatInputContainer}>
      <input
        type="text"
        placeholder="Talk with Moxi... 'Explain this page to me.', 'What is a crawler?', 'What can I do here?', 'What is Nullblock?'"
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        onKeyPress={sendMessage}
        className={styles.chatInput}
      />
      <button
        onClick={sendTextMessage}
        className={styles.sendButton} // Ensure you have styles for this
      >
        Send
      </button>
    </div>
  );
};

export default ChatInput;

