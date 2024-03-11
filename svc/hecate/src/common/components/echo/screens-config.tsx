const screensConfig = {
  Dashboard: {
    title: "ECHO initializing...",
    buttonText: "Home",
    usePopup: false,
    content: (
      <div>
        {/* Dashboard content */}
        <p>Dashboard content</p>
      </div>
    ),
    popupContent: null, // No popup content for this screen
  },
  Why: {
    title: "Why has The Destroyer arrived?",
    buttonText: "Why?",
    usePopup: false,
    content: (
      <div>
        {/* Why screen content */}
      </div>
    ),
    popupContent: null,
  },
  LORD: {
    title: "Modifications vendor coming soon!",
    buttonText: "Vendor",
    usePopup: false,
    content: (
      <div>
        {/* LORD screen content */}
      </div>
    ),
    popupContent: null, // No popup content for this screen
  },
  // Add other screens as needed
};

export default screensConfig;
