import { jsxs, jsx, Fragment } from "react/jsx-runtime";
import { useState, useEffect } from "react";
import PropTypes from "prop-types";
const appMenu = "_appMenu_o3o95_17";
const menuContainer = "_menuContainer_o3o95_60";
const active = "_active_o3o95_64";
const flickerGlow$3 = "_flickerGlow_o3o95_1";
const moveGradient$2 = "_moveGradient_o3o95_1";
const appButton = "_appButton_o3o95_83";
const pulse$3 = "_pulse_o3o95_1";
const styles$5 = {
  appMenu,
  menuContainer,
  active,
  flickerGlow: flickerGlow$3,
  moveGradient: moveGradient$2,
  appButton,
  pulse: pulse$3
};
const echoButton = "_echoButton_ognps_4";
const powerButton$1 = "_powerButton_ognps_4";
const ripple = "_ripple_ognps_39";
const pulse$2 = "_pulse_ognps_1";
const styles$4 = {
  echoButton,
  powerButton: powerButton$1,
  ripple,
  "ripple-effect": "_ripple-effect_ognps_1",
  pulse: pulse$2
};
const ButtonWrapper = ({ title, buttonImage, buttonText, setCurrentScreen }) => {
  const createRipple = (event) => {
    const circle = document.createElement("span");
    const diameter = Math.max(event.currentTarget.clientWidth, event.currentTarget.clientHeight);
    const radius = diameter / 2;
    circle.style.width = circle.style.height = `${diameter}px`;
    circle.style.left = `${event.clientX - event.currentTarget.offsetLeft - radius}px`;
    circle.style.top = `${event.clientY - event.currentTarget.offsetTop - radius}px`;
    circle.classList.add(styles$4.ripple);
    const rippleContainer = event.currentTarget;
    rippleContainer.appendChild(circle);
    setTimeout(() => {
      circle.remove();
    }, 600);
  };
  return /* @__PURE__ */ jsxs("div", { className: styles$4.wrapper, children: [
    " ",
    /* @__PURE__ */ jsxs(
      "button",
      {
        type: "button",
        className: styles$4.echoButton,
        onClick: (e) => {
          setCurrentScreen();
          createRipple(e);
        },
        children: [
          buttonImage && /* @__PURE__ */ jsx("img", { src: buttonImage, alt: title, className: styles$4.buttonImage }),
          " ",
          buttonText
        ]
      }
    )
  ] });
};
ButtonWrapper.propTypes = {
  title: PropTypes.string.isRequired,
  buttonText: PropTypes.string,
  buttonImage: PropTypes.string,
  setCurrentScreen: PropTypes.func.isRequired
};
const XLogo = "data:image/png;base64,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";
const discordLogo = "data:image/png;base64,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";
const telegramLogo = "data:image/png;base64,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";
const AppMenu = ({
  toggleEchoVisibility,
  closeEchoScreen,
  isUIVisible
}) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  useEffect(() => {
    if (!isUIVisible) {
      setIsMenuOpen(false);
    }
  }, [isUIVisible]);
  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };
  const buttons = [
    { href: "https://twitter.com/MoxiSKeeper", icon: XLogo, alt: "Twitter Logo" },
    { href: "https://discord.com", icon: discordLogo, alt: "Discord Logo" },
    { href: "https://telegram.org", icon: telegramLogo, alt: "Telegram Logo" }
  ];
  return /* @__PURE__ */ jsxs("div", { className: styles$5.appMenu, children: [
    /* @__PURE__ */ jsx(ButtonWrapper, { title: "Toggle ECHO", buttonText: "EChat", setCurrentScreen: toggleEchoVisibility }),
    /* @__PURE__ */ jsx(ButtonWrapper, { title: "Toggle Social Media Menu", buttonText: "Social Media", setCurrentScreen: toggleMenu }),
    /* @__PURE__ */ jsx("div", { className: `${styles$5.menuContainer} ${isMenuOpen ? styles$5.active : ""}`, children: buttons.map((button, index) => /* @__PURE__ */ jsx("a", { href: button.href, target: "_blank", rel: "noopener noreferrer", className: styles$5.appButton, children: /* @__PURE__ */ jsx("img", { src: button.icon, alt: button.alt }) }, index)) })
  ] });
};
const parentContainer = "_parentContainer_mxbti_22";
const sidebar = "_sidebar_mxbti_22";
const mainScreenContent = "_mainScreenContent_mxbti_22";
const echoContent = "_echoContent_mxbti_22";
const echoTitle = "_echoTitle_mxbti_22";
const scanLine = "_scanLine_mxbti_1";
const backgroundFade$1 = "_backgroundFade_mxbti_1";
const flickerEffect = "_flickerEffect_mxbti_1";
const typing$1 = "_typing_mxbti_1";
const blinkCaret$1 = "_blinkCaret_mxbti_1";
const holographicEffect = "_holographicEffect_mxbti_1";
const flickerGlow$2 = "_flickerGlow_mxbti_1";
const pulse$1 = "_pulse_mxbti_1";
const styles$3 = {
  parentContainer,
  sidebar,
  mainScreenContent,
  echoContent,
  echoTitle,
  scanLine,
  backgroundFade: backgroundFade$1,
  flickerEffect,
  typing: typing$1,
  blinkCaret: blinkCaret$1,
  holographicEffect,
  flickerGlow: flickerGlow$2,
  pulse: pulse$1
};
const PopupContent = ({ onClose, content, isEchoScreenVisible }) => {
  const popupStyle = {
    transform: isEchoScreenVisible ? "translate(-50%, -50%)" : "translate(30%, 0%)"
  };
  return /* @__PURE__ */ jsx("div", { className: styles$3.popupOverlay, onClick: onClose, children: /* @__PURE__ */ jsx("div", { className: styles$3.popupContent, style: popupStyle, onClick: (e) => e.stopPropagation(), children: content }) });
};
const UnifiedEchoScreen = ({
  screenTitle,
  images = { main: "", small: "" },
  isPopupVisible,
  onClosePopup,
  content,
  additionalContent = [],
  popupContent,
  closeCurrentScreen
  // Receive closeCurrentScreen function from parent
}) => {
  return /* @__PURE__ */ jsxs("div", { className: styles$3.echoScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles$3.contentWrapper, children: [
      images.main && /* @__PURE__ */ jsx("img", { src: images.main, alt: "Main visual", className: styles$3.echoImage }),
      /* @__PURE__ */ jsx("h2", { className: styles$3.echoTitle, children: screenTitle }),
      /* @__PURE__ */ jsx("p", { className: styles$3.echoContent, children: content }),
      additionalContent.map((text, index) => /* @__PURE__ */ jsx("p", { className: styles$3.echoContent, children: text }, index)),
      images.small && /* @__PURE__ */ jsx("img", { src: images.small, alt: "Small visual", className: styles$3.echoImageSmall })
    ] }),
    isPopupVisible && popupContent && /* @__PURE__ */ jsx(PopupContent, { onClose: onClosePopup, children: popupContent })
  ] });
};
const Echo = ({ screensConfig: screensConfig2, defaultScreenKey }) => {
  var _a, _b, _c, _d, _e;
  const [currentScreen, setCurrentScreen] = useState(defaultScreenKey);
  const [showPopup, setShowPopup] = useState(false);
  const [popupContent, setPopupContent] = useState(null);
  const [isEchoVisible, setIsEchoVisible] = useState(true);
  const [animationKey, setAnimationKey] = useState(Date.now());
  useEffect(() => {
    setCurrentScreen(defaultScreenKey);
  }, [defaultScreenKey]);
  const handleButtonClick = (screen) => {
    const screenConfig = screensConfig2[screen];
    if (screenConfig == null ? void 0 : screenConfig.usePopup) {
      setPopupContent(screenConfig.popupContent);
      setShowPopup(true);
    } else {
      setShowPopup(false);
    }
    setCurrentScreen(screen);
    setAnimationKey(Date.now());
  };
  const handleClosePopup = () => setShowPopup(false);
  const closeCurrentScreen = () => {
    setShowPopup(false);
    setCurrentScreen(defaultScreenKey);
  };
  return /* @__PURE__ */ jsx("div", { className: styles$3.parentContainer, children: /* @__PURE__ */ jsxs("div", { className: styles$3.mainScreenContent, children: [
    isEchoVisible && /* @__PURE__ */ jsx(
      UnifiedEchoScreen,
      {
        screenTitle: (_a = screensConfig2[currentScreen]) == null ? void 0 : _a.title,
        images: {
          main: (_b = screensConfig2[currentScreen]) == null ? void 0 : _b.image,
          small: (_c = screensConfig2[currentScreen]) == null ? void 0 : _c.image_small
        },
        isPopupVisible: showPopup,
        onClosePopup: handleClosePopup,
        content: (_d = screensConfig2[currentScreen]) == null ? void 0 : _d.content,
        additionalContent: ((_e = screensConfig2[currentScreen]) == null ? void 0 : _e.additionalContent) || [],
        popupContent,
        closeCurrentScreen
      },
      animationKey
    ),
    /* @__PURE__ */ jsx("div", { className: styles$3.sidebar, children: Object.keys(screensConfig2).map((screen) => /* @__PURE__ */ jsx(
      ButtonWrapper,
      {
        buttonText: screensConfig2[screen].buttonText,
        setCurrentScreen: () => handleButtonClick(screen),
        title: screensConfig2[screen].title
      },
      screen
    )) })
  ] }) });
};
const screensConfig = {
  Nexus: {
    title: "Nex grid initializing...",
    buttonText: "Nexus",
    usePopup: false,
    content: /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsx("h3", { children: "Nexus Interface Initialized. Please wait for the grid to load..." }),
      /* @__PURE__ */ jsx("p", { children: "Please login to activate your Nexus User Interface." })
    ] }),
    popupContent: null
    // No popup content for this screen
  }
  // Add other screens as needed
};
const echoChatScreensConfig = {
  ChatDashboard: {
    title: "Chat with ECHO",
    buttonText: "Chat",
    usePopup: true,
    content: /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsx("p", { children: "The chat system is currently initializing. Please wait while the system initializes." }),
      /* @__PURE__ */ jsx("p", { children: "Chat with ECHO by typing in the chat box below." }),
      'Where does it come from? Contrary to popular belief, Lorem Ipsum is not simply random text. It has roots in a piece of classical Latin literature from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in Virginia, looked up one of the more obscure Latin words, consectetur, from a Lorem Ipsum passage, and going through the cites of the word in classical literature, discovered the undoubtable source. Lorem Ipsum comes from sections 1.10.32 and 1.10.33 of "de Finibus Bonorum et Malorum" (The Extremes of Good and Evil) by Cicero, written in 45 BC. This book is a treatise on the theory of ethics, very popular during the Renaissance. The first line of Lorem Ipsum, "Lorem ipsum dolor sit amet..", comes from a line in section 1.10.32. The standard chunk of Lorem Ipsum used since the 1500s is reproduced below for those interested. Sections 1.10.32 and 1.10.33 from "de Finibus Bonorum et Malorum" by Cicero are also reproduced in their exact original form, accompanied by English versions from the 1914 translation by H. Rackham.'
    ] }),
    popupContent: /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx("p", { children: "The chat system is currently initializing. Please wait while the system initializes." }) })
  }
  // Add other screens as needed
};
const EchoChat = () => {
  const mergedScreensConfig = { ...screensConfig, ...echoChatScreensConfig };
  const defaultScreenKey = Object.keys(echoChatScreensConfig)[0];
  return /* @__PURE__ */ jsx(Echo, { screensConfig: mergedScreensConfig, defaultScreenKey });
};
const chatInputContainer = "_chatInputContainer_qxqfm_24";
const moveGradient$1 = "_moveGradient_qxqfm_1";
const chatInput = "_chatInput_qxqfm_24";
const sendButton = "_sendButton_qxqfm_54";
const floatingMessage = "_floatingMessage_qxqfm_81";
const moveUp = "_moveUp_qxqfm_1";
const flickerGlow$1 = "_flickerGlow_qxqfm_1";
const styles$2 = {
  chatInputContainer,
  moveGradient: moveGradient$1,
  chatInput,
  sendButton,
  floatingMessage,
  moveUp,
  flickerGlow: flickerGlow$1
};
const ChatInput = () => {
  const [message, setMessage] = useState("");
  const sendTextMessage = () => {
    if (message.trim()) {
      const messageContainer = document.createElement("div");
      messageContainer.className = styles$2.floatingMessage;
      messageContainer.textContent = message;
      document.body.appendChild(messageContainer);
      setMessage("");
      setTimeout(() => {
        document.body.removeChild(messageContainer);
      }, 3e3);
    }
  };
  const sendMessage = (e) => {
    if (e.key === "Enter") {
      sendTextMessage();
    }
  };
  return /* @__PURE__ */ jsxs("div", { className: styles$2.chatInputContainer, children: [
    /* @__PURE__ */ jsx(
      "input",
      {
        type: "text",
        placeholder: "Talk with Moxi... 'Explain this page to me.', 'What is a crawler?', 'What can I do here?', 'What is Nullblock?'",
        value: message,
        onChange: (e) => setMessage(e.target.value),
        onKeyPress: sendMessage,
        className: styles$2.chatInput
      }
    ),
    /* @__PURE__ */ jsx(
      "button",
      {
        onClick: sendTextMessage,
        className: styles$2.sendButton,
        children: "Send"
      }
    )
  ] });
};
const backgroundImage = "_backgroundImage_vga5m_60";
const bottomUIContainer = "_bottomUIContainer_vga5m_73";
const powerButtonContainer = "_powerButtonContainer_vga5m_86";
const powerButton = "_powerButton_vga5m_86";
const powerButtonImage = "_powerButtonImage_vga5m_107";
const splineObject = "_splineObject_vga5m_127";
const flickerGlow = "_flickerGlow_vga5m_1";
const moveGradient = "_moveGradient_vga5m_1";
const pulse = "_pulse_vga5m_1";
const typing = "_typing_vga5m_1";
const blinkCaret = "_blinkCaret_vga5m_1";
const backgroundFade = "_backgroundFade_vga5m_1";
const styles$1 = {
  backgroundImage,
  bottomUIContainer,
  powerButtonContainer,
  powerButton,
  powerButtonImage,
  splineObject,
  flickerGlow,
  moveGradient,
  pulse,
  typing,
  blinkCaret,
  backgroundFade
};
const powerOn = "/assets/echo_bot_night-CBKXRJLc.png";
const powerOff = "/assets/echo_bot_white-C7yP3Jt3.png";
const MoxiImage = "/assets/night_wolf_1-CbSwW3hQ.png";
const characterContainer = "_characterContainer_1ndom_7";
const styles = {
  characterContainer
};
const Moxi = () => {
  return /* @__PURE__ */ jsx("div", { className: styles.characterContainer, children: /* @__PURE__ */ jsx("img", { src: MoxiImage, alt: "Moxi" }) });
};
const Home = () => {
  const [isUIVisible, setIsUIVisible] = useState(true);
  const [showEchoChat, setShowEchoChat] = useState(false);
  useEffect(() => {
  }, []);
  const toggleUIVisibility = () => {
    setIsUIVisible(!isUIVisible);
    if (!isUIVisible) {
      setShowEchoChat(false);
    }
    setShowEchoChat(true);
  };
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsx("div", { className: styles$1.backgroundImage }),
    /* @__PURE__ */ jsx("div", { id: "fogOverlay", className: styles$1.fogOverlay }),
    /* @__PURE__ */ jsx("div", { className: styles$1.powerButtonContainer, children: /* @__PURE__ */ jsxs("button", { onClick: toggleUIVisibility, className: styles$1.powerButton, children: [
      /* @__PURE__ */ jsx("img", { src: isUIVisible ? powerOn : powerOff, alt: "Power button", className: styles$1.powerButtonImage }),
      /* @__PURE__ */ jsxs("span", { children: [
        " ",
        isUIVisible ? "Turn off" : "Turn on"
      ] })
    ] }) }),
    /* @__PURE__ */ jsxs("div", { className: styles$1.bottomUIContainer, children: [
      /* @__PURE__ */ jsx(Moxi, {}),
      isUIVisible && /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx(
          AppMenu,
          {
            toggleEchoVisibility: () => setShowEchoChat(!showEchoChat),
            closeEchoScreen: () => setShowEchoChat(false),
            isUIVisible
          }
        ),
        showEchoChat && /* @__PURE__ */ jsx(EchoChat, {})
      ] }),
      /* @__PURE__ */ jsx(ChatInput, {})
    ] })
  ] });
};
export {
  Home as default
};
