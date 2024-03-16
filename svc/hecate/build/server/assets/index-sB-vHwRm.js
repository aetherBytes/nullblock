import { jsx, jsxs } from "react/jsx-runtime";
import { useState } from "react";
const echoButton$1 = "_echoButton_jb5jl_1";
const styles$3 = {
  echoButton: echoButton$1
};
const ButtonWrapper = ({ title, buttonText, setCurrentScreen }) => /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx("button", { type: "button", className: styles$3.echoButton, onClick: () => setCurrentScreen(title), children: buttonText }) });
const parentContainer = "_parentContainer_az6sz_53";
const sidebar = "_sidebar_az6sz_60";
const pulse = "_pulse_az6sz_1";
const mainScreenContent = "_mainScreenContent_az6sz_68";
const flickerGlow = "_flickerGlow_az6sz_1";
const moveGradient = "_moveGradient_az6sz_1";
const backgroundFade = "_backgroundFade_az6sz_1";
const echoScreen = "_echoScreen_az6sz_81";
const echoTitle = "_echoTitle_az6sz_92";
const echoContent = "_echoContent_az6sz_92";
const typing = "_typing_az6sz_1";
const blinkCaret = "_blinkCaret_az6sz_1";
const echoImage = "_echoImage_az6sz_99";
const echoImageSmall = "_echoImageSmall_az6sz_99";
const styles$2 = {
  parentContainer,
  sidebar,
  pulse,
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
  echoImageSmall
};
const PopupContent = ({ onClose, content, isEchoScreenVisible }) => {
  const popupStyle = {
    transform: isEchoScreenVisible ? "translate(-50%, -50%)" : "translate(30%, 0%)"
  };
  return /* @__PURE__ */ jsx("div", { className: styles$2.popupOverlay, onClick: onClose, children: /* @__PURE__ */ jsx("div", { className: styles$2.popupContent, style: popupStyle, onClick: (e) => e.stopPropagation(), children: content }) });
};
const UnifiedEchoScreen = ({
  screenTitle,
  images,
  isPopupVisible,
  onClosePopup,
  content,
  additionalContent = [],
  popupContent
}) => {
  return /* @__PURE__ */ jsxs("div", { className: styles$2.echoScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles$2.contentWrapper, children: [
      images.main && /* @__PURE__ */ jsx("img", { src: images.main, alt: "Main visual", className: styles$2.echoImage }),
      /* @__PURE__ */ jsx("h2", { className: styles$2.echoTitle, children: screenTitle }),
      /* @__PURE__ */ jsx("p", { className: styles$2.echoContent, children: content }),
      additionalContent.map((text, index) => /* @__PURE__ */ jsx("p", { className: styles$2.echoContent, children: text }, index)),
      images.small && /* @__PURE__ */ jsx("img", { src: images.small, alt: "Small visual", className: styles$2.echoImageSmall })
    ] }),
    isPopupVisible && popupContent && /* @__PURE__ */ jsx(PopupContent, { onClose: onClosePopup, children: popupContent })
  ] });
};
const screensConfig = {
  Dashboard: {
    title: "ECHO initializing...",
    buttonText: "Home",
    usePopup: false,
    content: /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx("p", { children: "Dashboard content" }) }),
    popupContent: null
    // No popup content for this screen
  },
  Why: {
    title: "Why has The Destroyer arrived?",
    buttonText: "Why?",
    usePopup: false,
    content: /* @__PURE__ */ jsx("div", {}),
    popupContent: null
  },
  LORD: {
    title: "Modifications vendor coming soon!",
    buttonText: "Vendor",
    usePopup: false,
    content: /* @__PURE__ */ jsx("div", {}),
    popupContent: null
    // No popup content for this screen
  }
  // Add other screens as needed
};
const Echo = () => {
  const [currentScreen, setCurrentScreen] = useState("Dashboard");
  const [showPopup, setShowPopup] = useState(false);
  const [popupContent, setPopupContent] = useState(null);
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
  return /* @__PURE__ */ jsx("div", { className: styles$2.parentContainer, children: /* @__PURE__ */ jsxs("div", { className: styles$2.mainScreenContent, children: [
    /* @__PURE__ */ jsx("div", { className: styles$2.sidebar, children: Object.keys(screensConfig).map((screen) => /* @__PURE__ */ jsx(
      ButtonWrapper,
      {
        buttonText: screensConfig[screen].buttonText,
        setCurrentScreen: () => handleButtonClick(screen),
        title: screensConfig[screen].title
      },
      screen
    )) }),
    /* @__PURE__ */ jsx(
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
const characterContainer = "_characterContainer_wxotl_9";
const float = "_float_wxotl_1";
const styles$1 = {
  characterContainer,
  float
};
const MoxiImage = "/assets/moxi_lets_go-FnYeyNqL.png";
const Moxi = () => {
  return /* @__PURE__ */ jsx("div", { className: styles$1.characterContainer, children: /* @__PURE__ */ jsx("img", { src: MoxiImage }) });
};
const backgroundImage = "_backgroundImage_18e42_1";
const buttonsContainer = "_buttonsContainer_18e42_10";
const echoButton = "_echoButton_18e42_20";
const styles = {
  backgroundImage,
  buttonsContainer,
  echoButton
};
const XLogo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC4AAAAlCAYAAAA9ftv0AAAACXBIWXMAAADQAAAA0AF5Y8+UAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAABHxJREFUWIXNmF1oHFUUx+89d+7n7Nd8ZJfdfG1waRJ2G0ODCYE8pMWqYPPWvFgQyZPiW0AEUUgRq/RFBI19Kz6oRXwpRYRCq+IXig9aQiwoEkgkVGhNdJs2m8xcH8yGTbIfs8ls2gP/l3vvnPlx9r9nzgzSWqPD0vj4uBGPx08LIT40DGMeAG4RQv6klP7IOX87lUqNNMoxMjIS01qjHYtDQ0OqVdCWZZ2ilP6BENL1JIS4mkwmH9l9fSqVKiilLsbj8ed3gKdSqaNCiJ9yuVwsbOhIJHIWY+w3gi6LELJi2/YTyWTyUSnlNGPsO4yxTyld6OjokDvApZTvIIQ05/yLgYEBMyzoaDT6UlDgegKATcdxHi/n3b4BY+xm+RDn/Ouuri7roNCu6w4BwMZBoTHGG9Fo9LktZ5jd3d3Ht/80u29AKb2ZyWR6DwLOOf/8oNCU0ruJROKsaZrTjLHLlNI7juMMI601mpiYUDW8tppIJJ7dD3Q2m81ijL0wbFJRec+yrGe2rTI5OUkAYL3WBVLKK+l0ursZcKXUVJjQALCZSCSmqnn8RoML10zTPJ/L5dqCgAshzoUFTQi5Y1nW05X5K7vKmwGT3BVCXHBd91iDFvhuWOC2bZ/enb+yj/cAQKmZhIyxX4UQb9m2fbK3tzdamZgx9lpY4EqpUzXBtdbINM3zB/EgY+w3wzCuCCEumKZ5NcSKP1kXPJ/PM875l2HdMCzFYrHHdoMDQggVCoUuKeWlpaWl8eHh4Ukp5WfoIQrXdW/tWdRao9HRUVnuuQBwn1L6ezOzRStlGMbazMwM1LQKY2zuQUPWAP++WteCcuUB4NNgP9zhBqX0h2rr2+DxeHyWEFI8PKRgwTm/XnWjsvyxWOxF9BDYoywAKNYasas98d5/0MBlCSE+rvVk3ruAEI5EIq9ijJt6irZCtm2fDAxeMQIcFUJ8QgipOTW2UoyxuWptsCy8VeUdEY/Hz/i+H0UIUc/z+j3PmyqVSnzPwRZGLBY7s7q6+lGtfaPaou/7sWKxONs6rPrBGPt5enr6Ut1D1X6GfD7PKKXz6AFYZOstZ6zRvF9zo62tbZAQ8s9hgyulZoO8qNTdtCxrjBBy+7CgKaVzQT+NNDzQ09PTLaW83OqhixDydzqd7g8CHQi8oj0WhBBvcM6/YowthQkNAPccxzkRlKUp8LIymUynEOJ6iJW+77ruRLMcgQ/m8/mIUuoVAPg3THu4rnu8WehA4O3t7UeUUucIIX+FaQ/G2Jzruvv+Uoaz2Wy2VCr5Sil/ZWVFra+vtyGEjpRKpWNa6xMbGxv9WmuMQgqMsS+lfM+27ZcXFxfv7TtRZ2dnRin1AQBsoha3OyHEt47jDO+3ylWtkk6n+4UQFwFgLUxYjLHPOb9m2/ZTCP0/G4UKXlZfX58TiURe4Jxfq/c9sREsY+wXpdTrmUymLyzYHR7XWte0UaFQiCwvL48Vi8VBABj0fb/T87wUQkhqrRnGeBMhtAYAtw3DWPQ8b15KeSORSHyzsLCw3Kxtm4n/APuYZnlaNAykAAAAAElFTkSuQmCC";
const discordLogo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAmCAYAAACGeMg8AAAACXBIWXMAAARZAAAEWQFZ2yVJAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAAA41JREFUWIXV2ctr3UUUB/DPXNvEJk018QGJRSL4QrRg1dKFWNFSLOiioKAutNiKxSoWEfwDuqjiA3Uhgm5ciM+1uChIu/FRaXWRQtVSilStitpUah5NxsXchOt9/H6/SaNJDhy4zMz3zHn8Zuacc0OMUS6FELqwDhuwHmvxYIxxX6acW/EBDuEz7MfnMcaJbKVijJUYPbgP7+NPxCbeX1VWg8y9beScwoe4H71VZYWyiIQQrsfjeAAXlvjlaZyHy3Ax+rC8PjeJ0/gNJzCO10rkjeI9vB5j/KZwZYnHdmNaq9cWgvcU6lpgxM11Ly60ATN8FuuzDJE+jwOLQPlm/hrLcgzZtgiU7sQ7KxmCXvy4CBTuxL9gVbPeNa20E4NtxhcLXYJdLaNt3oqTFt7rZfy7pqg0R2QrLp2bo/5X6sf2f400RCPgsIX3dlX+HrWWw47bFoFyubyp3af1sKVHD83+qkejSzpAC+3hXB7FisaIbJAOUC5NSQng6TlgZ+ivuoypOWD7cAdmDbkrU8A0XsJQjHE1BnAPjmXI+AFb0F+XMYg98g3ajNlP65C8kD7VIbUZxPEK+J9weQcZj2bqMjJTiqySMsuqwC9JdUwHRe6tIGNrSfnwaYY+U7iI/Gv3yRIllkuHsBN+TP2AFsjITVo31rBGHh0umowxTuJowZJjMca/S/Y4kqnTDTVcnQnqqrCm+xzxVdY00jU1DGeC1hVNhhD6cVXBkuEQQlk+d0umTsM1DGWCtoUQegrmH8Oygvkanug0GULolm6uHBoi3f25L+o72pSc0sVxpgJ+XEOe1FRivzUHfU6Q2jO5wCg11LbgCtyE56UbqSp+Ai9Ln9Gw9KDum6Muo0FKEXpLQrfYabIm1SFLnmpS72qp01QNZY/TUqAzNakhnUMn/wtNznGPUzWMZAD2YrVUA7wrXbXzRWP4CJukJvjHGdgRUjqwW/U+71e4WzpfK6Vs9018K6/hPS29YW9Lnf4LpItnM76oKOMsnkP37N8KIYS1eEP19OAIdsUYP5kZCCEM4DpcKXl1ACvq02P4Q+piHpXqiF8bsBvxah1fhQ5iR4zxALR7Wber1jKdwLVF6XgOS/nZeIV9f8YOTZlFJ6E9eKbEoFfmy4iGfV8sMeBZrGyLLRF8Ph7R+hfDcW0ayfNgSJ/W3O+glEQWF2MZm6yR8qnvcOd8G9Gwz+1SF/EF3FgV9w8AjTOb/Pen8QAAAABJRU5ErkJggg==";
const telegramLogo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAoCAYAAACM/rhtAAAACXBIWXMAAAEnAAABJwGNvPDMAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAABJFJREFUWIXNmFGIVVUUhr/17zuaqZVaipRZIBYUlPmUJoqIYj0YDfZSJlYQKPYgZVQERflUYOlLZeGDRlhEA1mEWYlWolOR0FhQKRE4CTLljI02Ou4e7rlz9t1zzr3nzIzahf1y91prf2ev/+y19jHvPf+Hn5mNB3c3+Olgv0L/Tu99H977SzqAcaBnDHUZ8sE4BEy+lGCjQWsNdUZgA0No66UAq4BbZehoBNQttAEqC4O5rosJZuCWGzocgZ0W2ghMrtka9nkyd+Yiwbmlhr6LwPqEtgDTogcZZag7sfn+Qu/aPMPtjcD6hd4FZub4zA80+PKFAptl2MeGzgdg5w3bCcxq5Cv0Us0HKotHGuwmoR2G+ut3ze0B5ka2FUBxDMMdSPx6gTEjBXa90FuGzkZg30JlSYb9dNBqwKL/Jxk6V/W1Xd57hgs2RehVQ2cinXWAa40Bqj6VRUJbgZbBc255ml49OWRA4KpEKz0R2BFwKwGX4WOgpwzbDYzJ0d+bKSC3lQYExoLWGzoRgXWC1gKjc/zGG/a+4dqBK/PiBwf0sZo+i4KNAq0xdCwC6wI9DYxt9OIY6jB0GLi6gd3M4HjZNvB/EzAH7iFDRyKwU9WyxITG/u5eQ38bOgpc19hWa9L0uhUNAZOy1Jo8eQh2Rug1YEqzBxPakJyDncCMZlkyrK12kANTcwGhsthw7RHYWaG3gekF5DDJsE9TCVTF3sSnxdDJWptVNxcY3WnYlxll6T3g5oJanRXIoQeYU9DvrkB/r0RzSGjj4NPfPgFmF1kg0dsKQ/8MdCFUFhf1FXohKG9LIkCti07/fcC84mC0CG0OYpwD11rUv6o/t7/WegGX18UPzp5ecPeVCQxMNdxXYUMA7uGSMSamJdI+GzRvqC/Zuf0lA8+Nz0XQujIxEmm0Bv7rMwAHttcb1gaVBVk1tD6o1hj6N4QTerEsXKK/N4LydnvGRrAgXszQz6Anwja8llKhbZGtF9rc7KHy9affkjjHM5uLZOH5hg7FC1fB3V7DPkh2ui8DbntWX1dQJjOCOO9k2gTGBswX2m6oNwM2a/wVv3XlALU6KG8rGwLGbxboccN9beiU1bfu4TiR18EUS699mL79XFsYMIKV0PP5u2i7gDuGkN6WpJHwhn7MtSso5Npdthv0bHBvCC5E7gC4R4FxBQHnBPrbOFzAP5Oz8mCqV7fM0E8Zu3pS6PVmZZIgK+CWDhNwoFqcBq4JdsGBe8DQ79npd+3gHslqaA33TRAzv+Etlo60XoOey0jXtPp0D37bhTYBtyT2Eyy9ve1uuHZBvUyw9ILUCVxWP5/exqoNrbvHsI9SiDqt7gmbi6zyVhrQe099ULcqgL/C0B/BcTE7mLsh6azzPrH1k/MJpDRgcurXduSHWlkS2hTs3pYc31Hg7jfsi1ACQpuarlsUsCrsgXuDh8rCpBOuNbrHgYkFHvRW0GNQWVRoY8oAEnx5Mmy3oV+CtD9YJlbhNUsCmuEOZlSTtqF2MyMKmEDOs/Tu4Q11FEntUIcli5b6mdmNoGVgp6B/h/e+p3SQgr//AORCvOgEun8VAAAAAElFTkSuQmCC";
const Home = () => {
  return /* @__PURE__ */ jsxs("div", { className: styles.backgroundImage, children: [
    /* @__PURE__ */ jsx(Moxi, {}),
    /* @__PURE__ */ jsx(Echo, {}),
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
          href: "https://twitter.com",
          target: "_blank",
          rel: "noopener noreferrer",
          className: styles.echoButton,
          children: /* @__PURE__ */ jsx("img", { src: discordLogo, alt: "Discord Logo" })
        }
      ),
      /* @__PURE__ */ jsx(
        "a",
        {
          href: "https://twitter.com",
          target: "_blank",
          rel: "noopener noreferrer",
          className: styles.echoButton,
          children: /* @__PURE__ */ jsx("img", { src: telegramLogo, alt: "Telegram Logo" })
        }
      )
    ] })
  ] });
};
export {
  Home as default
};
