import { jsx, jsxs } from "react/jsx-runtime";
import { useState } from "react";
const MoxiImage = "/assets/moxi_lets_go-FnYeyNqL.png";
const characterContainer = "_characterContainer_14889_15";
const float = "_float_14889_1";
const styles$3 = {
  characterContainer,
  float
};
const Moxi = ({ onToggleEcho }) => /* @__PURE__ */ jsx("div", { className: styles$3.characterContainer, onClick: onToggleEcho, style: { cursor: "pointer" }, children: /* @__PURE__ */ jsx("img", { src: MoxiImage, alt: "Moxi" }) });
const echoButton$2 = "_echoButton_1kzce_1";
const styles$2 = {
  echoButton: echoButton$2
};
const ButtonWrapper = ({ title, buttonText, setCurrentScreen }) => /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx("button", { type: "button", className: styles$2.echoButton, onClick: () => setCurrentScreen(title), children: buttonText }) });
const parentContainer = "_parentContainer_21v21_56";
const sidebar = "_sidebar_21v21_65";
const mainScreenContent = "_mainScreenContent_21v21_71";
const flickerGlow = "_flickerGlow_21v21_1";
const moveGradient = "_moveGradient_21v21_1";
const backgroundFade = "_backgroundFade_21v21_1";
const echoScreen = "_echoScreen_21v21_84";
const echoTitle = "_echoTitle_21v21_98";
const echoContent = "_echoContent_21v21_99";
const typing = "_typing_21v21_1";
const blinkCaret = "_blinkCaret_21v21_1";
const echoImage = "_echoImage_21v21_108";
const echoImageSmall = "_echoImageSmall_21v21_109";
const pulse = "_pulse_21v21_1";
const echoButton$1 = "_echoButton_21v21_154";
const styles$1 = {
  parentContainer,
  sidebar,
  mainScreenContent,
  flickerGlow,
  moveGradient,
  backgroundFade,
  echoScreen,
  echoTitle,
  echoContent,
  typing,
  blinkCaret,
  echoImage,
  echoImageSmall,
  pulse,
  echoButton: echoButton$1
};
const PopupContent = ({ onClose, content, isEchoScreenVisible }) => {
  const popupStyle = {
    transform: isEchoScreenVisible ? "translate(-50%, -50%)" : "translate(30%, 0%)"
  };
  return /* @__PURE__ */ jsx("div", { className: styles$1.popupOverlay, onClick: onClose, children: /* @__PURE__ */ jsx("div", { className: styles$1.popupContent, style: popupStyle, onClick: (e) => e.stopPropagation(), children: content }) });
};
const UnifiedEchoScreen = ({
  screenTitle,
  images = { main: "", small: "" },
  // Default images prop if not provided
  isPopupVisible,
  onClosePopup,
  content,
  additionalContent = [],
  // Default value for additionalContent
  popupContent
}) => /* @__PURE__ */ jsxs("div", { className: styles$1.echoScreen, children: [
  /* @__PURE__ */ jsxs("div", { className: styles$1.contentWrapper, children: [
    images.main && /* @__PURE__ */ jsx("img", { src: images.main, alt: "Main visual", className: styles$1.echoImage }),
    /* @__PURE__ */ jsx("h2", { className: styles$1.echoTitle, children: screenTitle }),
    /* @__PURE__ */ jsx("p", { className: styles$1.echoContent, children: content }),
    additionalContent.map((text, index) => /* @__PURE__ */ jsx("p", { className: styles$1.echoContent, children: text }, index)),
    images.small && /* @__PURE__ */ jsx("img", { src: images.small, alt: "Small visual", className: styles$1.echoImageSmall })
  ] }),
  isPopupVisible && popupContent && /* @__PURE__ */ jsx(PopupContent, { onClose: onClosePopup, children: popupContent })
] });
const screensConfig = {
  Dashboard: {
    title: "ECHO initializing...",
    buttonText: "Home",
    usePopup: false,
    content: /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx("p", { children: "The ECHO system is initializing. Please wait while the system initializes." }) }),
    popupContent: null
    // No popup content for this screen
  },
  About: {
    title: "About ECHO",
    buttonText: "About",
    usePopup: false,
    content: /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsx("p", { children: `What is Lorem Ipsum? Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum. Why do we use it? It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like). Where does it come from? Contrary to popular belief, Lorem Ipsum is not simply random text. It has roots in a piece of classical Latin literature from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in Virginia, looked up one of the more obscure Latin words, consectetur, from a Lorem Ipsum passage, and going through the cites of the word in classical literature, discovered the undoubtable source. Lorem Ipsum comes from sections 1.10.32 and 1.10.33 of "de Finibus Bonorum et Malorum" (The Extremes of Good and Evil) by Cicero, written in 45 BC. This book is a treatise on the theory of ethics, very popular during the Renaissance. The first line of Lorem Ipsum, "Lorem ipsum dolor sit amet..", comes from a line in section 1.10.32.` }),
      /* @__PURE__ */ jsx("p", { children: `What is Lorem Ipsum? Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum. Why do we use it? It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like). Where does it come from? Contrary to popular belief, Lorem Ipsum is not simply random text. It has roots in a piece of classical Latin literature from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in Virginia, looked up one of the more obscure Latin words, consectetur, from a Lorem Ipsum passage, and going through the cites of the word in classical literature, discovered the undoubtable source. Lorem Ipsum comes from sections 1.10.32 and 1.10.33 of "de Finibus Bonorum et Malorum" (The Extremes of Good and Evil) by Cicero, written in 45 BC. This book is a treatise on the theory of ethics, very popular during the Renaissance. The first line of Lorem Ipsum, "Lorem ipsum dolor sit amet..", comes from a line in section 1.10.32.` }),
      /* @__PURE__ */ jsx("p", { children: `What is Lorem Ipsum? Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum. Why do we use it? It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like). Where does it come from? Contrary to popular belief, Lorem Ipsum is not simply random text. It has roots in a piece of classical Latin literature from 45 BC, making it over 2000 years old. Richard McClintock, a Latin professor at Hampden-Sydney College in Virginia, looked up one of the more obscure Latin words, consectetur, from a Lorem Ipsum passage, and going through the cites of the word in classical literature, discovered the undoubtable source. Lorem Ipsum comes from sections 1.10.32 and 1.10.33 of "de Finibus Bonorum et Malorum" (The Extremes of Good and Evil) by Cicero, written in 45 BC. This book is a treatise on the theory of ethics, very popular during the Renaissance. The first line of Lorem Ipsum, "Lorem ipsum dolor sit amet..", comes from a line in section 1.10.32.` })
    ] }),
    popupContent: null
  },
  Inventory: {
    title: "Inventory Management System",
    buttonText: "Inventory",
    usePopup: false,
    content: /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx("p", { children: "The Inventory Management System is currently initializing. Please wait while the system initializes." }) }),
    popupContent: null
    // No popup content for this screen
  }
  // Add other screens as needed
};
const Echo = () => {
  const [currentScreen, setCurrentScreen] = useState("Dashboard");
  const [showPopup, setShowPopup] = useState(false);
  const [popupContent, setPopupContent] = useState(null);
  const [isEchoVisible, setIsEchoVisible] = useState(true);
  const [animationKey, setAnimationKey] = useState(Date.now());
  const handleButtonClick = (screen) => {
    const screenConfig = screensConfig[screen];
    if (screenConfig.usePopup) {
      setPopupContent(screenConfig.content);
      setShowPopup(true);
    } else {
      setShowPopup(false);
    }
    setCurrentScreen(screen);
    setAnimationKey(Date.now());
  };
  const handleClosePopup = () => setShowPopup(false);
  return /* @__PURE__ */ jsx("div", { className: styles$1.parentContainer, children: /* @__PURE__ */ jsxs("div", { className: styles$1.mainScreenContent, children: [
    /* @__PURE__ */ jsx("div", { className: styles$1.sidebar, children: Object.keys(screensConfig).map((screen) => /* @__PURE__ */ jsx(
      ButtonWrapper,
      {
        buttonText: screensConfig[screen].buttonText,
        setCurrentScreen: () => handleButtonClick(screen),
        title: screensConfig[screen].title
      },
      screen
    )) }),
    isEchoVisible && /* @__PURE__ */ jsx(
      UnifiedEchoScreen,
      {
        screenTitle: screensConfig[currentScreen].title,
        images: {
          main: screensConfig[currentScreen].image,
          small: screensConfig[currentScreen].image_small
        },
        isPopupVisible: showPopup,
        onClosePopup: handleClosePopup,
        content: screensConfig[currentScreen].content,
        additionalContent: screensConfig[currentScreen].additionalContent || [],
        popupContent
      },
      animationKey
    )
  ] }) });
};
const XLogo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC4AAAAlCAYAAAA9ftv0AAAACXBIWXMAAADQAAAA0AF5Y8+UAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAABHxJREFUWIXNmF1oHFUUx+89d+7n7Nd8ZJfdfG1waRJ2G0ODCYE8pMWqYPPWvFgQyZPiW0AEUUgRq/RFBI19Kz6oRXwpRYRCq+IXig9aQiwoEkgkVGhNdJs2m8xcH8yGTbIfs8ls2gP/l3vvnPlx9r9nzgzSWqPD0vj4uBGPx08LIT40DGMeAG4RQv6klP7IOX87lUqNNMoxMjIS01qjHYtDQ0OqVdCWZZ2ilP6BENL1JIS4mkwmH9l9fSqVKiilLsbj8ed3gKdSqaNCiJ9yuVwsbOhIJHIWY+w3gi6LELJi2/YTyWTyUSnlNGPsO4yxTyld6OjokDvApZTvIIQ05/yLgYEBMyzoaDT6UlDgegKATcdxHi/n3b4BY+xm+RDn/Ouuri7roNCu6w4BwMZBoTHGG9Fo9LktZ5jd3d3Ht/80u29AKb2ZyWR6DwLOOf/8oNCU0ruJROKsaZrTjLHLlNI7juMMI601mpiYUDW8tppIJJ7dD3Q2m81ijL0wbFJRec+yrGe2rTI5OUkAYL3WBVLKK+l0ursZcKXUVJjQALCZSCSmqnn8RoML10zTPJ/L5dqCgAshzoUFTQi5Y1nW05X5K7vKmwGT3BVCXHBd91iDFvhuWOC2bZ/enb+yj/cAQKmZhIyxX4UQb9m2fbK3tzdamZgx9lpY4EqpUzXBtdbINM3zB/EgY+w3wzCuCCEumKZ5NcSKP1kXPJ/PM875l2HdMCzFYrHHdoMDQggVCoUuKeWlpaWl8eHh4Ukp5WfoIQrXdW/tWdRao9HRUVnuuQBwn1L6ezOzRStlGMbazMwM1LQKY2zuQUPWAP++WteCcuUB4NNgP9zhBqX0h2rr2+DxeHyWEFI8PKRgwTm/XnWjsvyxWOxF9BDYoywAKNYasas98d5/0MBlCSE+rvVk3ruAEI5EIq9ijJt6irZCtm2fDAxeMQIcFUJ8QgipOTW2UoyxuWptsCy8VeUdEY/Hz/i+H0UIUc/z+j3PmyqVSnzPwRZGLBY7s7q6+lGtfaPaou/7sWKxONs6rPrBGPt5enr6Ut1D1X6GfD7PKKXz6AFYZOstZ6zRvF9zo62tbZAQ8s9hgyulZoO8qNTdtCxrjBBy+7CgKaVzQT+NNDzQ09PTLaW83OqhixDydzqd7g8CHQi8oj0WhBBvcM6/YowthQkNAPccxzkRlKUp8LIymUynEOJ6iJW+77ruRLMcgQ/m8/mIUuoVAPg3THu4rnu8WehA4O3t7UeUUucIIX+FaQ/G2Jzruvv+Uoaz2Wy2VCr5Sil/ZWVFra+vtyGEjpRKpWNa6xMbGxv9WmuMQgqMsS+lfM+27ZcXFxfv7TtRZ2dnRin1AQBsoha3OyHEt47jDO+3ylWtkk6n+4UQFwFgLUxYjLHPOb9m2/ZTCP0/G4UKXlZfX58TiURe4Jxfq/c9sREsY+wXpdTrmUymLyzYHR7XWte0UaFQiCwvL48Vi8VBABj0fb/T87wUQkhqrRnGeBMhtAYAtw3DWPQ8b15KeSORSHyzsLCw3Kxtm4n/APuYZnlaNAykAAAAAElFTkSuQmCC";
const discordLogo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAmCAYAAACGeMg8AAAACXBIWXMAAARZAAAEWQFZ2yVJAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAAA41JREFUWIXV2ctr3UUUB/DPXNvEJk018QGJRSL4QrRg1dKFWNFSLOiioKAutNiKxSoWEfwDuqjiA3Uhgm5ciM+1uChIu/FRaXWRQtVSilStitpUah5NxsXchOt9/H6/SaNJDhy4zMz3zHn8Zuacc0OMUS6FELqwDhuwHmvxYIxxX6acW/EBDuEz7MfnMcaJbKVijJUYPbgP7+NPxCbeX1VWg8y9beScwoe4H71VZYWyiIQQrsfjeAAXlvjlaZyHy3Ax+rC8PjeJ0/gNJzCO10rkjeI9vB5j/KZwZYnHdmNaq9cWgvcU6lpgxM11Ly60ATN8FuuzDJE+jwOLQPlm/hrLcgzZtgiU7sQ7KxmCXvy4CBTuxL9gVbPeNa20E4NtxhcLXYJdLaNt3oqTFt7rZfy7pqg0R2QrLp2bo/5X6sf2f400RCPgsIX3dlX+HrWWw47bFoFyubyp3af1sKVHD83+qkejSzpAC+3hXB7FisaIbJAOUC5NSQng6TlgZ+ivuoypOWD7cAdmDbkrU8A0XsJQjHE1BnAPjmXI+AFb0F+XMYg98g3ajNlP65C8kD7VIbUZxPEK+J9weQcZj2bqMjJTiqySMsuqwC9JdUwHRe6tIGNrSfnwaYY+U7iI/Gv3yRIllkuHsBN+TP2AFsjITVo31rBGHh0umowxTuJowZJjMca/S/Y4kqnTDTVcnQnqqrCm+xzxVdY00jU1DGeC1hVNhhD6cVXBkuEQQlk+d0umTsM1DGWCtoUQegrmH8Oygvkanug0GULolm6uHBoi3f25L+o72pSc0sVxpgJ+XEOe1FRivzUHfU6Q2jO5wCg11LbgCtyE56UbqSp+Ai9Ln9Gw9KDum6Muo0FKEXpLQrfYabIm1SFLnmpS72qp01QNZY/TUqAzNakhnUMn/wtNznGPUzWMZAD2YrVUA7wrXbXzRWP4CJukJvjHGdgRUjqwW/U+71e4WzpfK6Vs9018K6/hPS29YW9Lnf4LpItnM76oKOMsnkP37N8KIYS1eEP19OAIdsUYP5kZCCEM4DpcKXl1ACvq02P4Q+piHpXqiF8bsBvxah1fhQ5iR4zxALR7Wber1jKdwLVF6XgOS/nZeIV9f8YOTZlFJ6E9eKbEoFfmy4iGfV8sMeBZrGyLLRF8Ph7R+hfDcW0ayfNgSJ/W3O+glEQWF2MZm6yR8qnvcOd8G9Gwz+1SF/EF3FgV9w8AjTOb/Pen8QAAAABJRU5ErkJggg==";
const telegramLogo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAoCAYAAACM/rhtAAAACXBIWXMAAAEnAAABJwGNvPDMAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAABJFJREFUWIXNmFGIVVUUhr/17zuaqZVaipRZIBYUlPmUJoqIYj0YDfZSJlYQKPYgZVQERflUYOlLZeGDRlhEA1mEWYlWolOR0FhQKRE4CTLljI02Ou4e7rlz9t1zzr3nzIzahf1y91prf2ev/+y19jHvPf+Hn5mNB3c3+Olgv0L/Tu99H977SzqAcaBnDHUZ8sE4BEy+lGCjQWsNdUZgA0No66UAq4BbZehoBNQttAEqC4O5rosJZuCWGzocgZ0W2ghMrtka9nkyd+Yiwbmlhr6LwPqEtgDTogcZZag7sfn+Qu/aPMPtjcD6hd4FZub4zA80+PKFAptl2MeGzgdg5w3bCcxq5Cv0Us0HKotHGuwmoR2G+ut3ze0B5ka2FUBxDMMdSPx6gTEjBXa90FuGzkZg30JlSYb9dNBqwKL/Jxk6V/W1Xd57hgs2RehVQ2cinXWAa40Bqj6VRUJbgZbBc255ml49OWRA4KpEKz0R2BFwKwGX4WOgpwzbDYzJ0d+bKSC3lQYExoLWGzoRgXWC1gKjc/zGG/a+4dqBK/PiBwf0sZo+i4KNAq0xdCwC6wI9DYxt9OIY6jB0GLi6gd3M4HjZNvB/EzAH7iFDRyKwU9WyxITG/u5eQ38bOgpc19hWa9L0uhUNAZOy1Jo8eQh2Rug1YEqzBxPakJyDncCMZlkyrK12kANTcwGhsthw7RHYWaG3gekF5DDJsE9TCVTF3sSnxdDJWptVNxcY3WnYlxll6T3g5oJanRXIoQeYU9DvrkB/r0RzSGjj4NPfPgFmF1kg0dsKQ/8MdCFUFhf1FXohKG9LIkCti07/fcC84mC0CG0OYpwD11rUv6o/t7/WegGX18UPzp5ecPeVCQxMNdxXYUMA7uGSMSamJdI+GzRvqC/Zuf0lA8+Nz0XQujIxEmm0Bv7rMwAHttcb1gaVBVk1tD6o1hj6N4QTerEsXKK/N4LydnvGRrAgXszQz6Anwja8llKhbZGtF9rc7KHy9affkjjHM5uLZOH5hg7FC1fB3V7DPkh2ui8DbntWX1dQJjOCOO9k2gTGBswX2m6oNwM2a/wVv3XlALU6KG8rGwLGbxboccN9beiU1bfu4TiR18EUS699mL79XFsYMIKV0PP5u2i7gDuGkN6WpJHwhn7MtSso5Npdthv0bHBvCC5E7gC4R4FxBQHnBPrbOFzAP5Oz8mCqV7fM0E8Zu3pS6PVmZZIgK+CWDhNwoFqcBq4JdsGBe8DQ79npd+3gHslqaA33TRAzv+Etlo60XoOey0jXtPp0D37bhTYBtyT2Eyy9ve1uuHZBvUyw9ILUCVxWP5/exqoNrbvHsI9SiDqt7gmbi6zyVhrQe099ULcqgL/C0B/BcTE7mLsh6azzPrH1k/MJpDRgcurXduSHWlkS2hTs3pYc31Hg7jfsi1ACQpuarlsUsCrsgXuDh8rCpBOuNbrHgYkFHvRW0GNQWVRoY8oAEnx5Mmy3oV+CtD9YJlbhNUsCmuEOZlSTtqF2MyMKmEDOs/Tu4Q11FEntUIcli5b6mdmNoGVgp6B/h/e+p3SQgr//AORCvOgEun8VAAAAAElFTkSuQmCC";
const moxi_black = "data:image/png;base64,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";
const backgroundImage = "_backgroundImage_vk2q6_1";
const echoButton = "_echoButton_vk2q6_10";
const buttonsContainer = "_buttonsContainer_vk2q6_28";
const styles = {
  backgroundImage,
  echoButton,
  buttonsContainer
};
const Home = () => {
  const [isEchoVisible, setIsEchoVisible] = useState(false);
  const toggleEchoVisibility = () => setIsEchoVisible((prev) => !prev);
  return /* @__PURE__ */ jsxs("div", { className: styles.backgroundImage, children: [
    /* @__PURE__ */ jsx(Moxi, { onToggleEcho: toggleEchoVisibility }),
    /* @__PURE__ */ jsxs("div", { className: styles.buttonsContainer, children: [
      /* @__PURE__ */ jsx(
        "a",
        {
          href: "https://twitter.com/MoxiSKeeper",
          target: "_blank",
          rel: "noopener noreferrer",
          className: styles.echoButton,
          children: /* @__PURE__ */ jsx("img", { src: XLogo, alt: "Twitter Logo" })
        }
      ),
      /* @__PURE__ */ jsx(
        "a",
        {
          href: "https://discord.com",
          target: "_blank",
          rel: "noopener noreferrer",
          className: styles.echoButton,
          children: /* @__PURE__ */ jsx("img", { src: discordLogo, alt: "Discord Logo" })
        }
      ),
      /* @__PURE__ */ jsx(
        "a",
        {
          href: "https://telegram.org",
          target: "_blank",
          rel: "noopener noreferrer",
          className: styles.echoButton,
          children: /* @__PURE__ */ jsx("img", { src: telegramLogo, alt: "Telegram Logo" })
        }
      ),
      /* @__PURE__ */ jsx("button", { onClick: toggleEchoVisibility, className: styles.echoButton, children: /* @__PURE__ */ jsx("img", { src: moxi_black, alt: "Moxi Logo" }) })
    ] }),
    isEchoVisible && /* @__PURE__ */ jsx(Echo, {})
  ] });
};
export {
  Home as default
};
