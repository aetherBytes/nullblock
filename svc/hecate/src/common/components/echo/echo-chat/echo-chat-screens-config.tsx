const echoChatScreensConfig = {
  ChatDashboard: {
    title: 'Chat with ECHO',
    buttonText: 'Chat',
    usePopup: true,
    content: <div>{/* Chat screen content */}
    <p>
      The chat system is currently initializing. Please wait while the system initializes.
    </p>
    </div>,
    popupContent: (
      <div>
        <p>
          The chat system is currently initializing. Please wait while the system initializes.
        </p>
      </div>
    ),
  },
  // Add other screens as needed
};

export default echoChatScreensConfig;
