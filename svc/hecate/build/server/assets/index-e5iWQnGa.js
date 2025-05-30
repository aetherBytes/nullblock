import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { Suspense, useRef, useState, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { Preload, Points, PointMaterial } from "@react-three/drei";
import * as random from "maath/random/dist/maath-random.esm.js";
import axios from "axios";
const backgroundImage$1 = "_backgroundImage_1xw9p_12";
const button$1 = "_button_1xw9p_57";
const alertText$1 = "_alertText_1xw9p_74";
const scene$1 = "_scene_1xw9p_80";
const fire$1 = "_fire_1xw9p_92";
const campfireFlicker$1 = "_campfireFlicker_1xw9p_1";
const campfireGlow$1 = "_campfireGlow_1xw9p_1";
const appContainer$1 = "_appContainer_1xw9p_153";
const lightFireFlicker$1 = "_lightFireFlicker_1xw9p_1";
const lightFireGlow$1 = "_lightFireGlow_1xw9p_1";
const socialLinks$1 = "_socialLinks_1xw9p_235";
const socialLink$1 = "_socialLink_1xw9p_235";
const nullButton$1 = "_nullButton_1xw9p_314";
const styles$2 = {
  backgroundImage: backgroundImage$1,
  button: button$1,
  alertText: alertText$1,
  scene: scene$1,
  fire: fire$1,
  campfireFlicker: campfireFlicker$1,
  campfireGlow: campfireGlow$1,
  appContainer: appContainer$1,
  "theme-null": "_theme-null_1xw9p_160",
  "theme-light": "_theme-light_1xw9p_165",
  lightFireFlicker: lightFireFlicker$1,
  lightFireGlow: lightFireGlow$1,
  socialLinks: socialLinks$1,
  socialLink: socialLink$1,
  nullButton: nullButton$1
};
const starsCanvas = "_starsCanvas_4g78i_1";
const matrix$1 = "_matrix_4g78i_10";
const cyber$1 = "_cyber_4g78i_14";
const light$1 = "_light_4g78i_18";
const styles$1 = {
  starsCanvas,
  matrix: matrix$1,
  cyber: cyber$1,
  light: light$1
};
const Stars = ({ theme = "light" }) => {
  const ref = useRef(null);
  const [sphere] = useState(() => {
    const positions = new Float32Array(5e3);
    random.inSphere(positions, { radius: 1.2 });
    return positions;
  });
  useFrame((state, delta) => {
    if (ref.current) {
      ref.current.rotation.x -= delta / 120;
      ref.current.rotation.y -= delta / 180;
    }
  });
  const getStarColor = () => {
    switch (theme) {
      case "light":
        return "#000000";
      case "matrix":
        return "#00ff00";
      case "cyber":
        return "#00ffff";
      default:
        return "#f272c8";
    }
  };
  return /* @__PURE__ */ jsx("group", { rotation: [0, 0, Math.PI / 4], children: /* @__PURE__ */ jsx(Points, { ref, positions: sphere, stride: 3, frustumCulled: true, children: /* @__PURE__ */ jsx(
    PointMaterial,
    {
      transparent: true,
      color: getStarColor(),
      size: theme === "light" ? 3e-3 : 2e-3,
      sizeAttenuation: true,
      depthWrite: false
    }
  ) }) });
};
const StarsCanvas = ({ theme = "light" }) => {
  return /* @__PURE__ */ jsx("div", { className: `${styles$1.starsCanvas} ${styles$1[theme]}`, children: /* @__PURE__ */ jsxs(Canvas, { camera: { position: [0, 0, 1] }, children: [
    /* @__PURE__ */ jsx(Suspense, { fallback: null, children: /* @__PURE__ */ jsx(Stars, { theme }) }),
    /* @__PURE__ */ jsx(Preload, { all: true })
  ] }) });
};
const backgroundImage = "_backgroundImage_epeip_13";
const button = "_button_epeip_58";
const alertText = "_alertText_epeip_75";
const scene = "_scene_epeip_81";
const fire = "_fire_epeip_93";
const campfireFlicker = "_campfireFlicker_epeip_1";
const campfireGlow = "_campfireGlow_epeip_1";
const appContainer = "_appContainer_epeip_154";
const lightFireFlicker = "_lightFireFlicker_epeip_1";
const lightFireGlow = "_lightFireGlow_epeip_1";
const socialLinks = "_socialLinks_epeip_236";
const socialLink = "_socialLink_epeip_236";
const nullButton = "_nullButton_epeip_315";
const echoContainer = "_echoContainer_epeip_347";
const hudWindow = "_hudWindow_epeip_360";
const controlPanel = "_controlPanel_epeip_365";
const controlButton = "_controlButton_epeip_368";
const disabled = "_disabled_epeip_371";
const active = "_active_epeip_374";
const userProfile = "_userProfile_epeip_382";
const profileLabel = "_profileLabel_epeip_385";
const profileValue = "_profileValue_epeip_388";
const statusIndicator = "_statusIndicator_epeip_391";
const inactive = "_inactive_epeip_394";
const alertButton = "_alertButton_epeip_397";
const expandButton = "_expandButton_epeip_403";
const disconnectButton = "_disconnectButton_epeip_409";
const matrix = "_matrix_epeip_415";
const cyber = "_cyber_epeip_473";
const light = "_light_epeip_531";
const softEcho = "_softEcho_epeip_1";
const corePulse = "_corePulse_epeip_1";
const scanlineGlow = "_scanlineGlow_epeip_1";
const scanlineWaver = "_scanlineWaver_epeip_1";
const hudTitle = "_hudTitle_epeip_631";
const bottomLeftInfo = "_bottomLeftInfo_epeip_677";
const bottomRightInfo = "_bottomRightInfo_epeip_677";
const verticalNavbar = "_verticalNavbar_epeip_690";
const screenLabel = "_screenLabel_epeip_702";
const navbarDivider = "_navbarDivider_epeip_710";
const navbarButtons = "_navbarButtons_epeip_718";
const homeButton = "_homeButton_epeip_728";
const nullLogoButton = "_nullLogoButton_epeip_729";
const batLogoButton = "_batLogoButton_epeip_730";
const socialButton = "_socialButton_epeip_731";
const homeIcon = "_homeIcon_epeip_758";
const batLogoIcon = "_batLogoIcon_epeip_825";
const navDivider = "_navDivider_epeip_838";
const navButton = "_navButton_epeip_850";
const locked = "_locked_epeip_891";
const lockIcon = "_lockIcon_epeip_902";
const nullLogoIcon = "_nullLogoIcon_epeip_907";
const socialIcon = "_socialIcon_epeip_907";
const nexus = "_nexus_epeip_948";
const hudScreen = "_hudScreen_epeip_948";
const settingsScreen = "_settingsScreen_epeip_948";
const walletInfo = "_walletInfo_epeip_976";
const nexusActions = "_nexusActions_epeip_1016";
const headerContainer = "_headerContainer_epeip_1082";
const architectTitle = "_architectTitle_epeip_1093";
const leaderboardContainer = "_leaderboardContainer_epeip_1103";
const leaderboardTitle = "_leaderboardTitle_epeip_1115";
const leaderboardButton = "_leaderboardButton_epeip_1125";
const leaderboardList = "_leaderboardList_epeip_1144";
const leaderboardItems = "_leaderboardItems_epeip_1170";
const scrollLeaderboard = "_scrollLeaderboard_epeip_1";
const leaderboardItem = "_leaderboardItem_epeip_1170";
const rank = "_rank_epeip_1196";
const camperId = "_camperId_epeip_1202";
const matrixLevel = "_matrixLevel_epeip_1211";
const headerDivider = "_headerDivider_epeip_1221";
const campGrid = "_campGrid_epeip_1226";
const campAnalysis = "_campAnalysis_epeip_1234";
const diagnosticsContainer = "_diagnosticsContainer_epeip_1244";
const containerTitle = "_containerTitle_epeip_1255";
const diagnosticsHeader = "_diagnosticsHeader_epeip_1266";
const diagnosticsContent = "_diagnosticsContent_epeip_1279";
const diagnosticsList = "_diagnosticsList_epeip_1302";
const diagnosticsItem = "_diagnosticsItem_epeip_1309";
const itemLabel = "_itemLabel_epeip_1350";
const itemValue = "_itemValue_epeip_1353";
const campContent = "_campContent_epeip_1384";
const campStatus = "_campStatus_epeip_1406";
const statusCard = "_statusCard_epeip_1412";
const statusHeaderContainer = "_statusHeaderContainer_epeip_1423";
const statusTabs = "_statusTabs_epeip_1444";
const statusTab = "_statusTab_epeip_1444";
const activeTab = "_activeTab_epeip_1467";
const tabContent = "_tabContent_epeip_1480";
const scanline = "_scanline_epeip_1";
const statusContent = "_statusContent_epeip_1501";
const vitalsContainer = "_vitalsContainer_epeip_1522";
const vitalItem = "_vitalItem_epeip_1544";
const vitalValue = "_vitalValue_epeip_1572";
const vitalLabel = "_vitalLabel_epeip_1575";
const infoButton = "_infoButton_epeip_1606";
const pulse = "_pulse_epeip_1";
const ascentDetails = "_ascentDetails_epeip_1637";
const shimmer = "_shimmer_epeip_1";
const ascentDescription = "_ascentDescription_epeip_1656";
const progressBar = "_progressBar_epeip_1662";
const progressFill = "_progressFill_epeip_1680";
const accoladesContainer = "_accoladesContainer_epeip_1697";
const accoladesTitle = "_accoladesTitle_epeip_1700";
const accoladesList = "_accoladesList_epeip_1717";
const visible = "_visible_epeip_1733";
const blurred = "_blurred_epeip_1742";
const missionsTab = "_missionsTab_epeip_1752";
const systemsTab = "_systemsTab_epeip_1752";
const defenseTab = "_defenseTab_epeip_1752";
const uplinkTab = "_uplinkTab_epeip_1752";
const missionHeader = "_missionHeader_epeip_1764";
const missionContent = "_missionContent_epeip_1774";
const availableMissions = "_availableMissions_epeip_1786";
const missionList = "_missionList_epeip_1813";
const missionItem = "_missionItem_epeip_1818";
const missionItemContent = "_missionItemContent_epeip_1842";
const missionTitle = "_missionTitle_epeip_1849";
const missionStatus = "_missionStatus_epeip_1857";
const missionReward = "_missionReward_epeip_1861";
const missionDescription = "_missionDescription_epeip_1873";
const missionText = "_missionText_epeip_1905";
const missionInstructions = "_missionInstructions_epeip_1915";
const highlight = "_highlight_epeip_1941";
const missionNote = "_missionNote_epeip_1945";
const missionExpiration = "_missionExpiration_epeip_1954";
const rewardLabel = "_rewardLabel_epeip_1960";
const expirationLabel = "_expirationLabel_epeip_1960";
const rewardValue = "_rewardValue_epeip_1967";
const expirationValue = "_expirationValue_epeip_1967";
const activeMissionDetails = "_activeMissionDetails_epeip_1974";
const systemsContent = "_systemsContent_epeip_2000";
const defenseContent = "_defenseContent_epeip_2000";
const uplinkContent = "_uplinkContent_epeip_2000";
const systemsList = "_systemsList_epeip_2025";
const systemItem = "_systemItem_epeip_2031";
const systemName = "_systemName_epeip_2040";
const defenseStatus = "_defenseStatus_epeip_2044";
const uplinkStatus = "_uplinkStatus_epeip_2044";
const defenseDescription = "_defenseDescription_epeip_2049";
const uplinkDescription = "_uplinkDescription_epeip_2049";
const tabs = "_tabs_epeip_2054";
const tab = "_tab_epeip_1480";
const profileItem = "_profileItem_epeip_2137";
const label = "_label_epeip_2149";
const value = "_value_epeip_2160";
const common = "_common_epeip_2170";
const uncommon = "_uncommon_epeip_2173";
const rare = "_rare_epeip_2176";
const epic = "_epic_epeip_2179";
const legendary = "_legendary_epeip_2182";
const none = "_none_epeip_2207";
const collapsedButton = "_collapsedButton_epeip_2212";
const withEcho = "_withEcho_epeip_2212";
const ascentLine = "_ascentLine_epeip_2215";
const lockedContent = "_lockedContent_epeip_2333";
const statusContainer = "_statusContainer_epeip_2383";
const statusLabel = "_statusLabel_epeip_2394";
const echoContent = "_echoContent_epeip_2408";
const echoStatus = "_echoStatus_epeip_2435";
const browserInfo = "_browserInfo_epeip_2443";
const browserLabel = "_browserLabel_epeip_2447";
const browserValue = "_browserValue_epeip_2451";
const echoMessage = "_echoMessage_epeip_2455";
const disconnectedContent = "_disconnectedContent_epeip_2469";
const extensionPrompt = "_extensionPrompt_epeip_2487";
const extensionLinks = "_extensionLinks_epeip_2507";
const extensionButton = "_extensionButton_epeip_2514";
const uplinkItem = "_uplinkItem_epeip_2529";
const uplinkIcon = "_uplinkIcon_epeip_2551";
const uplinkInfo = "_uplinkInfo_epeip_2560";
const uplinkName = "_uplinkName_epeip_2564";
const pending = "_pending_epeip_2583";
const uplinkModal = "_uplinkModal_epeip_2592";
const modalHeader = "_modalHeader_epeip_2617";
const closeButton = "_closeButton_epeip_2632";
const modalContent = "_modalContent_epeip_2645";
const statusSection = "_statusSection_epeip_2648";
const statusGrid = "_statusGrid_epeip_2658";
const statusItem = "_statusItem_epeip_2663";
const detailsSection = "_detailsSection_epeip_2685";
const modalOverlay = "_modalOverlay_epeip_2699";
const leaderboardModal = "_leaderboardModal_epeip_2710";
const legendWarning = "_legendWarning_epeip_2744";
const leaderboardGrid = "_leaderboardGrid_epeip_2759";
const leaderboardCard = "_leaderboardCard_epeip_2766";
const cardHeader = "_cardHeader_epeip_2779";
const vitalsGrid = "_vitalsGrid_epeip_2847";
const addLinkButton = "_addLinkButton_epeip_2854";
const echoBatChamberLogoWrapper = "_echoBatChamberLogoWrapper_epeip_2899";
const echoBatChamberLogo = "_echoBatChamberLogo_epeip_2899";
const echoBatFade = "_echoBatFade_epeip_1";
const centeredLabel = "_centeredLabel_epeip_2972";
const nullblockTitle = "_nullblockTitle_epeip_2980";
const styles = {
  backgroundImage,
  button,
  alertText,
  scene,
  fire,
  campfireFlicker,
  campfireGlow,
  appContainer,
  "theme-null": "_theme-null_epeip_161",
  "theme-light": "_theme-light_epeip_166",
  lightFireFlicker,
  lightFireGlow,
  socialLinks,
  socialLink,
  nullButton,
  echoContainer,
  "null": "_null_epeip_315",
  hudWindow,
  controlPanel,
  controlButton,
  disabled,
  active,
  userProfile,
  profileLabel,
  profileValue,
  statusIndicator,
  inactive,
  alertButton,
  expandButton,
  disconnectButton,
  matrix,
  cyber,
  light,
  softEcho,
  corePulse,
  scanlineGlow,
  scanlineWaver,
  hudTitle,
  bottomLeftInfo,
  bottomRightInfo,
  verticalNavbar,
  screenLabel,
  navbarDivider,
  navbarButtons,
  homeButton,
  nullLogoButton,
  batLogoButton,
  socialButton,
  homeIcon,
  batLogoIcon,
  navDivider,
  navButton,
  locked,
  lockIcon,
  nullLogoIcon,
  socialIcon,
  nexus,
  hudScreen,
  settingsScreen,
  walletInfo,
  nexusActions,
  headerContainer,
  architectTitle,
  leaderboardContainer,
  leaderboardTitle,
  leaderboardButton,
  leaderboardList,
  leaderboardItems,
  scrollLeaderboard,
  leaderboardItem,
  rank,
  camperId,
  matrixLevel,
  headerDivider,
  campGrid,
  campAnalysis,
  diagnosticsContainer,
  containerTitle,
  diagnosticsHeader,
  diagnosticsContent,
  diagnosticsList,
  diagnosticsItem,
  itemLabel,
  itemValue,
  campContent,
  campStatus,
  statusCard,
  statusHeaderContainer,
  statusTabs,
  statusTab,
  activeTab,
  tabContent,
  scanline,
  statusContent,
  vitalsContainer,
  vitalItem,
  vitalValue,
  vitalLabel,
  infoButton,
  pulse,
  ascentDetails,
  shimmer,
  ascentDescription,
  progressBar,
  progressFill,
  accoladesContainer,
  accoladesTitle,
  accoladesList,
  visible,
  blurred,
  missionsTab,
  systemsTab,
  defenseTab,
  uplinkTab,
  missionHeader,
  missionContent,
  availableMissions,
  missionList,
  missionItem,
  missionItemContent,
  missionTitle,
  missionStatus,
  missionReward,
  missionDescription,
  missionText,
  missionInstructions,
  highlight,
  missionNote,
  missionExpiration,
  rewardLabel,
  expirationLabel,
  rewardValue,
  expirationValue,
  activeMissionDetails,
  systemsContent,
  defenseContent,
  uplinkContent,
  systemsList,
  systemItem,
  systemName,
  defenseStatus,
  uplinkStatus,
  defenseDescription,
  uplinkDescription,
  tabs,
  tab,
  profileItem,
  label,
  value,
  common,
  uncommon,
  rare,
  epic,
  legendary,
  none,
  collapsedButton,
  withEcho,
  ascentLine,
  lockedContent,
  statusContainer,
  statusLabel,
  echoContent,
  echoStatus,
  browserInfo,
  browserLabel,
  browserValue,
  echoMessage,
  disconnectedContent,
  extensionPrompt,
  extensionLinks,
  extensionButton,
  uplinkItem,
  uplinkIcon,
  uplinkInfo,
  uplinkName,
  pending,
  uplinkModal,
  modalHeader,
  closeButton,
  modalContent,
  statusSection,
  statusGrid,
  statusItem,
  detailsSection,
  modalOverlay,
  leaderboardModal,
  legendWarning,
  leaderboardGrid,
  leaderboardCard,
  cardHeader,
  vitalsGrid,
  addLinkButton,
  echoBatChamberLogoWrapper,
  echoBatChamberLogo,
  echoBatFade,
  centeredLabel,
  nullblockTitle
};
const API_BASE_URL = "http://localhost:8000";
const fetchWalletData = async (publicKey) => {
  try {
    const baseUrl = "http://localhost:8000";
    const url = `${baseUrl}/api/wallet/${publicKey}`;
    const response = await axios.get(url);
    if (response.status !== 200) {
      throw new Error(`Unexpected response status: ${response.status}`);
    }
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error("Failed to fetch wallet data from backend:", error.message);
    } else {
      console.error("Unexpected error:", error);
    }
    throw error;
  }
};
const fetchUserProfile = async (publicKey) => {
  try {
    const baseUrl = "http://localhost:8000";
    const url = `${baseUrl}/api/wallet/health/${publicKey}`;
    const response = await axios.get(url);
    if (response.status !== 200) {
      throw new Error(`Unexpected response status: ${response.status}`);
    }
    console.log("User profile data:", response.data);
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error("Failed to fetch user profile from backend:", error.message);
    } else {
      console.error("Unexpected error:", error);
    }
    throw error;
  }
};
const fetchAscentLevel = async (publicKey) => {
  try {
    return {
      level: 1,
      name: "Net Dweller",
      description: "A newcomer to the digital realm, exploring the boundaries of the network.",
      progress: 0.35,
      accolades: [
        "First Connection",
        "Wallet Initiated",
        "Basic Navigation",
        "Token Discovery",
        "Transaction Initiate",
        "Network Explorer",
        "Data Collector",
        "Interface Familiar"
      ]
    };
  } catch (error) {
    console.error("Failed to fetch ascent level:", error);
    throw error;
  }
};
const fetchActiveMission = async (publicKey) => {
  try {
    const response = await fetch(`${API_BASE_URL}/api/missions/${publicKey}`);
    if (!response.ok) {
      throw new Error("Failed to fetch active mission");
    }
    const data = await response.json();
    return data;
  } catch (error) {
    console.error("Error fetching active mission:", error);
    return {
      id: "1",
      title: "Share on X",
      description: "Share your Base Camp on X to earn GLIMMER",
      status: "active",
      reward: "TBD GLIMMER AIRDROP",
      x_url: "https://twitter.com/intent/tweet?text=Check%20out%20my%20Base%20Camp%20in%20the%20Nullblock%20universe!%20%40Nullblock_io%20%24GLIMMER"
    };
  }
};
const xLogo = "data:image/png;base64,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";
const nullLogo = "/assets/null_logo-YcRNLHqr.png";
const echoBatWhite = "/assets/echo_bat_white-CUrRC6jy.png";
const echoBatBlack = "/assets/echo_bat_black-BIfG_SPS.png";
const SCREEN_LABELS = {
  chambers: "E.C.H.O CHAMBERS",
  camp: "CAMP",
  inventory: "CACHE",
  campaign: "CAMPAIGN",
  lab: "LAB"
};
const Echo = ({
  publicKey,
  onDisconnect,
  theme = "light",
  onClose,
  onThemeChange
}) => {
  var _a;
  const [screen, setScreen] = useState("chambers");
  const [walletData, setWalletData] = useState(null);
  const [userProfile2, setUserProfile] = useState({
    id: publicKey ? `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol` : "",
    ascent: 1,
    nether: null,
    cacheValue: 0,
    memories: 0,
    matrix: {
      level: "NONE",
      rarity: "NONE",
      status: "N/A"
    }
  });
  const [ascentLevel, setAscentLevel] = useState(null);
  const [showAscentDetails, setShowAscentDetails] = useState(false);
  useState(3);
  useState(false);
  const [showNectarDetails, setShowNectarDetails] = useState(false);
  const [showCacheValueDetails, setShowCacheValueDetails] = useState(false);
  const [showEmberConduitDetails, setShowEmberConduitDetails] = useState(false);
  const [showMemoriesDetails, setShowMemoriesDetails] = useState(false);
  const [activeMission, setActiveMission] = useState(null);
  const [showMissionDropdown, setShowMissionDropdown] = useState(false);
  useState(false);
  const missionDropdownRef = useRef(null);
  const cardRef = useRef(null);
  const [activeTab2, setActiveTab] = useState("echo");
  const [emberLinkStatus, setEmberLinkStatus] = useState({
    connected: false,
    lastSeen: null,
    browserInfo: null
  });
  useState(false);
  const [selectedUplink, setSelectedUplink] = useState(null);
  const [showLeaderboard, setShowLeaderboard] = useState(false);
  const unlockedScreens = publicKey ? ["chambers", "camp"] : ["chambers"];
  [
    {
      id: "ember",
      name: "Ember Link",
      status: emberLinkStatus.connected ? "active" : "inactive",
      icon: "🔥",
      details: {
        description: "Direct connection to the Ember network, enabling secure communication and data transfer.",
        stats: [
          {
            label: "Connection Status",
            value: emberLinkStatus.connected ? "Connected" : "Disconnected",
            status: emberLinkStatus.connected ? "active" : "inactive"
          },
          {
            label: "Last Seen",
            value: emberLinkStatus.lastSeen ? new Date(emberLinkStatus.lastSeen).toLocaleString() : "Never"
          },
          {
            label: "Browser",
            value: emberLinkStatus.browserInfo ? `${emberLinkStatus.browserInfo.name} ${emberLinkStatus.browserInfo.version}` : "Unknown"
          },
          {
            label: "Platform",
            value: ((_a = emberLinkStatus.browserInfo) == null ? void 0 : _a.platform) || "Unknown"
          }
        ]
      }
    },
    {
      id: "neural",
      name: "Neural Link",
      status: "inactive",
      icon: "🧠",
      details: {
        description: "Advanced neural interface for enhanced cognitive processing and system interaction.",
        stats: [
          {
            label: "Status",
            value: "LOCKED",
            status: "inactive"
          },
          {
            label: "Signal Strength",
            value: "N/A"
          },
          {
            label: "Latency",
            value: "N/A"
          }
        ]
      }
    },
    {
      id: "wallet",
      name: "Wallet Health",
      status: "inactive",
      icon: "💳",
      details: {
        description: "Real-time monitoring of wallet security and transaction status.",
        stats: [
          {
            label: "Status",
            value: "LOCKED",
            status: "inactive"
          },
          {
            label: "Last Transaction",
            value: "N/A"
          },
          {
            label: "Security Level",
            value: "N/A"
          }
        ]
      }
    },
    {
      id: "token",
      name: "Token Analysis",
      status: "inactive",
      icon: "🪙",
      details: {
        description: "Comprehensive analysis of token holdings and market performance.",
        stats: [
          {
            label: "Status",
            value: "LOCKED",
            status: "inactive"
          },
          {
            label: "Scan Progress",
            value: "N/A"
          },
          {
            label: "Last Update",
            value: "N/A"
          }
        ]
      }
    }
  ];
  const leaderboardData = [
    {
      id: "PervySage",
      rank: 1,
      ascent: 999,
      nether: 999999,
      cacheValue: 999999,
      memories: 999,
      matrix: {
        level: "ARCHITECT",
        rarity: "MYTHICAL",
        status: "FLAME KEEPER"
      }
    },
    {
      id: "ECHO-001",
      rank: 2,
      ascent: 5,
      nether: 1500,
      cacheValue: 2500,
      memories: 12,
      matrix: {
        level: "MASTER",
        rarity: "LEGENDARY",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-002",
      rank: 3,
      ascent: 4,
      nether: 1200,
      cacheValue: 2e3,
      memories: 10,
      matrix: {
        level: "EXPERT",
        rarity: "EPIC",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-003",
      rank: 4,
      ascent: 3,
      nether: 900,
      cacheValue: 1500,
      memories: 8,
      matrix: {
        level: "ADVANCED",
        rarity: "RARE",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-004",
      rank: 5,
      ascent: 2,
      nether: 600,
      cacheValue: 1e3,
      memories: 6,
      matrix: {
        level: "INTERMEDIATE",
        rarity: "UNCOMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-005",
      rank: 6,
      ascent: 1,
      nether: 300,
      cacheValue: 500,
      memories: 4,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-006",
      rank: 7,
      ascent: 1,
      nether: 250,
      cacheValue: 400,
      memories: 3,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-007",
      rank: 8,
      ascent: 1,
      nether: 200,
      cacheValue: 300,
      memories: 2,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-008",
      rank: 9,
      ascent: 1,
      nether: 150,
      cacheValue: 200,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-009",
      rank: 10,
      ascent: 1,
      nether: 100,
      cacheValue: 150,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-010",
      rank: 11,
      ascent: 1,
      nether: 50,
      cacheValue: 100,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    },
    {
      id: "ECHO-011",
      rank: 12,
      ascent: 1,
      nether: 25,
      cacheValue: 50,
      memories: 1,
      matrix: {
        level: "BEGINNER",
        rarity: "COMMON",
        status: "LAST FLAME"
      }
    }
  ];
  useEffect(() => {
    const loadWalletData = async () => {
      if (publicKey) {
        try {
          const data = await fetchWalletData(publicKey);
          setWalletData(data);
          try {
            const profileData = await fetchUserProfile(publicKey);
            const hasNectarToken = profileData.active_tokens.includes("NECTAR");
            setUserProfile((prev) => ({
              ...prev,
              nether: hasNectarToken ? data.balance : null,
              cacheValue: data.balance || 0,
              // Set cache value to wallet balance
              id: profileData.username ? `@${profileData.username}` : `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol`
            }));
            console.log("Profile data received:", profileData);
            console.log("Username:", profileData.username);
          } catch (profileError) {
            console.error("Failed to fetch user profile:", profileError);
            setUserProfile((prev) => ({
              ...prev,
              nether: null,
              // Set to null if we can't determine if Nectar exists
              cacheValue: data.balance || 0
              // Set cache value to wallet balance
            }));
          }
          try {
            const ascentData = await fetchAscentLevel(publicKey);
            setAscentLevel({
              level: ascentData.level,
              title: ascentData.name,
              description: ascentData.description,
              progress: ascentData.progress,
              nextLevel: ascentData.level + 1,
              nextTitle: `Level ${ascentData.level + 1}`,
              nextDescription: "Next level description will be available soon.",
              accolades: ascentData.accolades
            });
            setUserProfile((prev) => ({
              ...prev,
              ascent: ascentData.level
            }));
          } catch (ascentError) {
            console.error("Failed to fetch ascent level:", ascentError);
          }
          try {
            const missionData = await fetchActiveMission(publicKey);
            setActiveMission(missionData);
          } catch (missionError) {
            console.error("Failed to fetch active mission:", missionError);
          }
        } catch (error) {
          console.error("Failed to fetch wallet data:", error);
        }
      }
    };
    loadWalletData();
  }, [publicKey]);
  useEffect(() => {
    const setupEmberLinkConnection = () => {
      const interval = setInterval(() => {
        setEmberLinkStatus((prev) => ({
          ...prev,
          lastSeen: /* @__PURE__ */ new Date()
        }));
      }, 3e4);
      return () => clearInterval(interval);
    };
    const cleanup = setupEmberLinkConnection();
    return cleanup;
  }, []);
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (missionDropdownRef.current && !missionDropdownRef.current.contains(event.target) && cardRef.current && !cardRef.current.contains(event.target)) {
        setShowMissionDropdown(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);
  const handleCloseModal = () => {
    setSelectedUplink(null);
  };
  const handleLeaderboardClick = () => {
    setShowLeaderboard(true);
  };
  const handleCloseLeaderboard = () => {
    setShowLeaderboard(false);
  };
  const renderControlScreen = () => /* @__PURE__ */ jsxs("nav", { className: styles.verticalNavbar, children: [
    /* @__PURE__ */ jsx("div", { className: styles.nullblockTitle, children: "NULLBLOCK" }),
    /* @__PURE__ */ jsx(
      "div",
      {
        className: `${styles.screenLabel} ${screen === "chambers" ? styles.centeredLabel : ""}`,
        children: SCREEN_LABELS[screen]
      }
    ),
    /* @__PURE__ */ jsxs("div", { className: styles.navbarButtons, children: [
      /* @__PURE__ */ jsx("button", { className: styles.batLogoButton, onClick: () => setScreen("chambers"), children: /* @__PURE__ */ jsx("img", { src: echoBatWhite, alt: "Bat Logo", className: styles.batLogoIcon }) }),
      /* @__PURE__ */ jsx("button", { className: styles.nullLogoButton, onClick: () => setScreen("chambers"), children: /* @__PURE__ */ jsx("img", { src: nullLogo, alt: "Null Logo", className: styles.nullLogoIcon }) }),
      /* @__PURE__ */ jsx(
        "a",
        {
          href: "https://x.com/Nullblock_io",
          target: "_blank",
          rel: "noopener noreferrer",
          className: styles.socialButton,
          children: /* @__PURE__ */ jsx("img", { src: xLogo, alt: "X", className: styles.socialIcon })
        }
      )
    ] })
  ] });
  const renderUserProfile = () => {
    var _a2;
    return /* @__PURE__ */ jsxs("div", { className: styles.userProfile, children: [
      /* @__PURE__ */ jsxs("div", { className: styles.profileItem, children: [
        /* @__PURE__ */ jsx("span", { className: styles.label, children: "ID:" }),
        /* @__PURE__ */ jsx("span", { className: styles.value, children: userProfile2.id })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles.label, children: [
          "ASCENT:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles.infoButton,
              onClick: () => setShowAscentDetails(!showAscentDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs("div", { className: styles.ascentContainer, children: [
          /* @__PURE__ */ jsx("span", { className: styles.value, children: "Net Dweller: 1" }),
          /* @__PURE__ */ jsx("div", { className: styles.progressBar, children: /* @__PURE__ */ jsx(
            "div",
            {
              className: styles.progressFill,
              style: { width: `${35}%` }
            }
          ) })
        ] }),
        showAscentDetails && /* @__PURE__ */ jsxs("div", { className: styles.ascentDetails, children: [
          /* @__PURE__ */ jsx("div", { className: styles.ascentDescription, children: "A digital lurker extraordinaire! You've mastered the art of watching from the shadows, observing the chaos without ever dipping your toes in. Like a cat watching a laser pointer, you're fascinated but paralyzed by indecision. At least you're not the one getting your digital assets rekt!" }),
          /* @__PURE__ */ jsx("div", { className: styles.progressText, children: "35% to next level" }),
          /* @__PURE__ */ jsxs("div", { className: styles.accoladesContainer, children: [
            /* @__PURE__ */ jsx("div", { className: styles.accoladesTitle, children: "ACCOLADES" }),
            /* @__PURE__ */ jsxs("ul", { className: styles.accoladesList, children: [
              /* @__PURE__ */ jsx("li", { className: styles.visible, children: "First Connection" }),
              /* @__PURE__ */ jsx("li", { className: styles.visible, children: "Wallet Initiated" }),
              /* @__PURE__ */ jsx("li", { className: styles.visible, children: "Basic Navigation" }),
              /* @__PURE__ */ jsx("li", { className: styles.blurred, children: "Token Discovery" }),
              /* @__PURE__ */ jsx("li", { className: styles.blurred, children: "Transaction Initiate" }),
              /* @__PURE__ */ jsx("li", { className: styles.blurred, children: "Network Explorer" }),
              /* @__PURE__ */ jsx("li", { className: styles.blurred, children: "Data Collector" }),
              /* @__PURE__ */ jsx("li", { className: styles.blurred, children: "Interface Familiar" })
            ] })
          ] })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles.label, children: [
          "NETHER:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles.infoButton,
              onClick: () => setShowNectarDetails(!showNectarDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs("span", { className: styles.value, children: [
          "₦ ",
          ((_a2 = userProfile2.nether) == null ? void 0 : _a2.toFixed(2)) || "N/A"
        ] }),
        showNectarDetails && /* @__PURE__ */ jsx("div", { className: styles.ascentDetails, children: /* @__PURE__ */ jsx("div", { className: styles.ascentDescription, children: "NETHER: Magic internet money from the void. Born from nothing, worth everything, and somehow gaining value by the second. The integration has passed the event horizon - good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion and an irresistible urge to dive deeper into the code." }) })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles.label, children: [
          "cache value:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles.infoButton,
              onClick: () => setShowCacheValueDetails(!showCacheValueDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsx("span", { className: styles.value, children: "₦ N/A" }),
        showCacheValueDetails && /* @__PURE__ */ jsx("div", { className: styles.ascentDetails, children: /* @__PURE__ */ jsx("div", { className: styles.ascentDescription, children: "Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement agents. This is the total worth of all valuable assets in your wallet - coins, tokens, and other digital goodies that caught our eye. Coming soon: Categories for services, participant offerings, biological enhancements, and agent capabilities. Think of it as your personal inventory of everything worth something in the Nullblock universe. Don't spend it all in one place!" }) })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles.label, children: [
          "MEMORIES:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles.infoButton,
              onClick: () => setShowMemoriesDetails(!showMemoriesDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsx("span", { className: styles.value, children: userProfile2.memories }),
        showMemoriesDetails && /* @__PURE__ */ jsx("div", { className: styles.ascentDetails, children: /* @__PURE__ */ jsx("div", { className: styles.ascentDescription, children: "Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah, right - another poor...soul. You need to collect the artifacts that tell your story in the Nullblock universe. Each memory is a unique representation of your achievements, collectibles, and digital identity. Collect them all to unlock the secret of why you're here... or don't, I'm not your digital conscience." }) })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles.label, children: [
          "E.C:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles.infoButton,
              onClick: () => setShowEmberConduitDetails(!showEmberConduitDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsx("span", { className: styles.value, children: userProfile2.matrix.status }),
        showEmberConduitDetails && /* @__PURE__ */ jsx("div", { className: `${styles.ascentDetails} ${styles.rightAligned}`, children: /* @__PURE__ */ jsx("div", { className: styles.ascentDescription, children: "Ember Conduit: A medium to speak into flame. This ancient technology allows direct communication with the primordial forces of the Nullblock universe. Through an Ember Conduit, users can channel energy, access forbidden knowledge, and potentially reshape reality itself. Warning: Unauthorized use may result in spontaneous combustion or worse." }) })
      ] })
    ] });
  };
  const renderLockedScreen = () => /* @__PURE__ */ jsxs("div", { className: styles.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles.hudTitle, children: "ACCESS RESTRICTED" }),
      /* @__PURE__ */ jsx("div", { className: styles.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.lockedContent, children: [
      /* @__PURE__ */ jsx("p", { children: "This feature is currently locked." }),
      /* @__PURE__ */ jsx("p", { children: "Return to camp and await further instructions." })
    ] })
  ] });
  const renderEchoTab = () => {
    var _a2, _b, _c;
    return /* @__PURE__ */ jsx("div", { className: styles.echoContent, children: emberLinkStatus.connected ? /* @__PURE__ */ jsxs(Fragment, { children: [
      /* @__PURE__ */ jsxs("div", { className: styles.echoStatus, children: [
        /* @__PURE__ */ jsxs("div", { className: styles.statusContainer, children: [
          /* @__PURE__ */ jsx("span", { className: styles.statusLabel, children: "Ember Link Status:" }),
          /* @__PURE__ */ jsx("span", { className: styles.active, children: "Connected" })
        ] }),
        /* @__PURE__ */ jsxs("div", { className: styles.browserInfo, children: [
          /* @__PURE__ */ jsx("span", { className: styles.browserLabel, children: "Browser:" }),
          /* @__PURE__ */ jsxs("span", { className: styles.browserValue, children: [
            (_a2 = emberLinkStatus.browserInfo) == null ? void 0 : _a2.name,
            " ",
            (_b = emberLinkStatus.browserInfo) == null ? void 0 : _b.version,
            " (",
            (_c = emberLinkStatus.browserInfo) == null ? void 0 : _c.platform,
            ")"
          ] })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.echoMessage, children: [
        /* @__PURE__ */ jsx("p", { children: "E.C.H.O system is active and operational." }),
        /* @__PURE__ */ jsx("p", { children: "Welcome to the interface, agent." })
      ] })
    ] }) : /* @__PURE__ */ jsxs("div", { className: styles.disconnectedContent, children: [
      /* @__PURE__ */ jsx("div", { className: styles.echoStatus, children: /* @__PURE__ */ jsxs("div", { className: styles.statusContainer, children: [
        /* @__PURE__ */ jsx("span", { className: styles.statusLabel, children: "Ember Link Status:" }),
        /* @__PURE__ */ jsx("span", { className: styles.inactive, children: "Disconnected" })
      ] }) }),
      /* @__PURE__ */ jsxs("div", { className: styles.extensionPrompt, children: [
        /* @__PURE__ */ jsx("h4", { children: "Browser Extension Required" }),
        /* @__PURE__ */ jsx("p", { children: "To establish a secure connection, you need to install the Aether browser extension." }),
        /* @__PURE__ */ jsx("p", { children: "Choose your browser to download the extension:" }),
        /* @__PURE__ */ jsxs("div", { className: styles.extensionLinks, children: [
          /* @__PURE__ */ jsx(
            "a",
            {
              href: "https://chrome.google.com/webstore/detail/aether",
              target: "_blank",
              rel: "noopener noreferrer",
              className: styles.extensionButton,
              children: "Chrome Extension"
            }
          ),
          /* @__PURE__ */ jsx(
            "a",
            {
              href: "https://addons.mozilla.org/en-US/firefox/addon/aether",
              target: "_blank",
              rel: "noopener noreferrer",
              className: styles.extensionButton,
              children: "Firefox Extension"
            }
          )
        ] })
      ] })
    ] }) });
  };
  const renderCampScreen = () => {
    var _a2;
    return /* @__PURE__ */ jsxs("div", { className: styles.hudScreen, children: [
      /* @__PURE__ */ jsxs("div", { className: styles.headerContainer, children: [
        /* @__PURE__ */ jsx("h2", { className: styles.hudTitle, children: "CAMP" }),
        /* @__PURE__ */ jsx("div", { className: styles.headerDivider }),
        /* @__PURE__ */ jsx("h2", { className: styles.architectTitle, children: "ARCHITECT VIEW" }),
        /* @__PURE__ */ jsxs("div", { className: styles.leaderboardContainer, children: [
          /* @__PURE__ */ jsx("div", { className: styles.leaderboardTitle, children: /* @__PURE__ */ jsx("button", { className: styles.leaderboardButton, onClick: handleLeaderboardClick, children: "ASCENDANTS" }) }),
          /* @__PURE__ */ jsx("div", { className: styles.leaderboardList, children: /* @__PURE__ */ jsxs("div", { className: styles.leaderboardItems, children: [
            leaderboardData.map((entry) => /* @__PURE__ */ jsxs(
              "div",
              {
                className: styles.leaderboardItem,
                onClick: () => {
                  setSelectedUplink({
                    id: entry.id,
                    name: entry.id,
                    status: "active",
                    icon: "👤",
                    details: {
                      description: `Status profile for ${entry.id}`,
                      stats: [
                        {
                          label: "Rank",
                          value: `#${entry.rank}`
                        },
                        {
                          label: "Ascent",
                          value: entry.ascent
                        },
                        {
                          label: "Nether",
                          value: `₦ ${entry.nether}`
                        },
                        {
                          label: "Cache Value",
                          value: `₦ ${entry.cacheValue}`
                        },
                        {
                          label: "Memories",
                          value: entry.memories
                        },
                        {
                          label: "Matrix Level",
                          value: entry.matrix.level
                        },
                        {
                          label: "Rarity",
                          value: entry.matrix.rarity
                        },
                        {
                          label: "Status",
                          value: entry.matrix.status
                        }
                      ]
                    }
                  });
                },
                children: [
                  /* @__PURE__ */ jsxs("span", { className: styles.rank, children: [
                    "#",
                    entry.rank
                  ] }),
                  /* @__PURE__ */ jsx("span", { className: styles.camperId, children: entry.id }),
                  /* @__PURE__ */ jsx("span", { className: styles.matrixLevel, children: entry.matrix.level })
                ]
              },
              entry.id
            )),
            leaderboardData.map((entry) => /* @__PURE__ */ jsxs(
              "div",
              {
                className: styles.leaderboardItem,
                onClick: () => {
                  setSelectedUplink({
                    id: entry.id,
                    name: entry.id,
                    status: "active",
                    icon: "👤",
                    details: {
                      description: `Status profile for ${entry.id}`,
                      stats: [
                        {
                          label: "Rank",
                          value: `#${entry.rank}`
                        },
                        {
                          label: "Ascent",
                          value: entry.ascent
                        },
                        {
                          label: "Nether",
                          value: `₦ ${entry.nether}`
                        },
                        {
                          label: "Cache Value",
                          value: `₦ ${entry.cacheValue}`
                        },
                        {
                          label: "Memories",
                          value: entry.memories
                        },
                        {
                          label: "Matrix Level",
                          value: entry.matrix.level
                        },
                        {
                          label: "Rarity",
                          value: entry.matrix.rarity
                        },
                        {
                          label: "Status",
                          value: entry.matrix.status
                        }
                      ]
                    }
                  });
                },
                children: [
                  /* @__PURE__ */ jsxs("span", { className: styles.rank, children: [
                    "#",
                    entry.rank
                  ] }),
                  /* @__PURE__ */ jsx("span", { className: styles.camperId, children: entry.id }),
                  /* @__PURE__ */ jsx("span", { className: styles.matrixLevel, children: entry.matrix.level })
                ]
              },
              `${entry.id}-duplicate`
            ))
          ] }) })
        ] })
      ] }),
      /* @__PURE__ */ jsx("div", { className: styles.campContent, children: /* @__PURE__ */ jsxs("div", { className: styles.campGrid, children: [
        /* @__PURE__ */ jsx("div", { className: styles.campAnalysis, children: /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsContainer, children: [
          /* @__PURE__ */ jsx("h2", { className: styles.containerTitle, children: "SUBNETS" }),
          /* @__PURE__ */ jsx("div", { className: styles.diagnosticsHeader, children: /* @__PURE__ */ jsx("h3", { children: "SUBNETS" }) }),
          /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsContent, children: [
            /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsList, children: [
              /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsItem, onClick: () => setSelectedUplink({
                id: "id",
                name: "ID",
                status: "active",
                icon: "🆔",
                details: {
                  description: "Your unique identifier in the Nullblock universe. This is your digital fingerprint, your signature in the void.",
                  stats: [
                    {
                      label: "Status",
                      value: "Active"
                    },
                    {
                      label: "Type",
                      value: "ECHO ID"
                    }
                  ]
                }
              }), children: [
                /* @__PURE__ */ jsx("span", { className: styles.itemLabel, children: "🆔 ID" }),
                /* @__PURE__ */ jsxs("span", { className: styles.itemValue, children: [
                  "ECHO-",
                  userProfile2.id || "0000"
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsItem, onClick: () => setSelectedUplink({
                id: "ascent",
                name: "ASCENT",
                status: "active",
                icon: "↗️",
                details: {
                  description: "A digital lurker extraordinaire! You've mastered the art of watching from the shadows, observing the chaos without ever dipping your toes in. Like a cat watching a laser pointer, you're fascinated but paralyzed by indecision. At least you're not the one getting your digital assets rekt!",
                  stats: [
                    {
                      label: "Level",
                      value: "Net Dweller: 1"
                    },
                    {
                      label: "Progress",
                      value: "35%"
                    }
                  ]
                }
              }), children: [
                /* @__PURE__ */ jsxs("span", { className: styles.itemLabel, children: [
                  /* @__PURE__ */ jsx("span", { className: styles.ascentLine }),
                  " ASCENT"
                ] }),
                /* @__PURE__ */ jsx("span", { className: styles.itemValue, children: "Net Dweller: 1" })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsItem, onClick: () => {
                var _a3;
                return setSelectedUplink({
                  id: "nether",
                  name: "NETHER",
                  status: "active",
                  icon: "₦",
                  details: {
                    description: "NETHER: Magic internet money from the void. Born from nothing, worth everything, and somehow gaining value by the second. The integration has passed the event horizon - good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion and an irresistible urge to dive deeper into the code.",
                    stats: [
                      {
                        label: "Balance",
                        value: `₦ ${((_a3 = userProfile2.nether) == null ? void 0 : _a3.toFixed(2)) || "N/A"}`
                      },
                      {
                        label: "Status",
                        value: userProfile2.nether ? "Active" : "Inactive"
                      }
                    ]
                  }
                });
              }, children: [
                /* @__PURE__ */ jsx("span", { className: styles.itemLabel, children: "₦ NETHER" }),
                /* @__PURE__ */ jsxs("span", { className: styles.itemValue, children: [
                  "₦ ",
                  ((_a2 = userProfile2.nether) == null ? void 0 : _a2.toFixed(2)) || "N/A"
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsItem, onClick: () => setSelectedUplink({
                id: "cache",
                name: "CACHE VALUE",
                status: "active",
                icon: "💰",
                details: {
                  description: "Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement agents. This is the total worth of all valuable assets in your wallet - coins, tokens, and other digital goodies that caught our eye. Coming soon: Categories for services, participant offerings, biological enhancements, and agent capabilities. Think of it as your personal inventory of everything worth something in the Nullblock universe. Don't spend it all in one place!",
                  stats: [
                    {
                      label: "Value",
                      value: "₦ N/A"
                    },
                    {
                      label: "Status",
                      value: "Pending"
                    }
                  ]
                }
              }), children: [
                /* @__PURE__ */ jsx("span", { className: styles.itemLabel, children: "💰 CACHE VALUE" }),
                /* @__PURE__ */ jsx("span", { className: styles.itemValue, children: "₦ N/A" })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsItem, onClick: () => setSelectedUplink({
                id: "memories",
                name: "MEMORIES",
                status: "active",
                icon: "🧠",
                details: {
                  description: "Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah, right - another poor...soul. You need to collect the artifacts that tell your story in the Nullblock universe. Each memory is a unique representation of your achievements, collectibles, and digital identity. Collect them all to unlock the secret of why you're here... or don't, I'm not your digital conscience.",
                  stats: [
                    {
                      label: "Count",
                      value: userProfile2.memories
                    },
                    {
                      label: "Status",
                      value: userProfile2.memories > 0 ? "Active" : "Empty"
                    }
                  ]
                }
              }), children: [
                /* @__PURE__ */ jsx("span", { className: styles.itemLabel, children: "🧠 MEMORIES" }),
                /* @__PURE__ */ jsx("span", { className: styles.itemValue, children: userProfile2.memories })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.diagnosticsItem, onClick: () => setSelectedUplink({
                id: "ec",
                name: "EMBER CONDUIT",
                status: "active",
                icon: "🔥",
                details: {
                  description: "Ember Conduit: A medium to speak into flame. This ancient technology allows direct communication with the primordial forces of the Nullblock universe. Through an Ember Conduit, users can channel energy, access forbidden knowledge, and potentially reshape reality itself. Warning: Unauthorized use may result in spontaneous combustion or worse.",
                  stats: [
                    {
                      label: "Status",
                      value: userProfile2.matrix.status
                    },
                    {
                      label: "Type",
                      value: "Ember Conduit"
                    }
                  ]
                }
              }), children: [
                /* @__PURE__ */ jsx("span", { className: styles.itemLabel, children: "🔥 EMBER CONDUIT" }),
                /* @__PURE__ */ jsx("span", { className: styles.itemValue, children: userProfile2.matrix.status })
              ] })
            ] }),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: styles.addLinkButton,
                onClick: () => alert("No Ember Conduit loaded"),
                children: "🔗 ADD NET"
              }
            )
          ] })
        ] }) }),
        /* @__PURE__ */ jsx("div", { className: styles.divider }),
        /* @__PURE__ */ jsx("div", { className: styles.campStatus, children: /* @__PURE__ */ jsxs("div", { className: styles.statusCard, children: [
          /* @__PURE__ */ jsxs("div", { className: styles.statusTabs, children: [
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles.statusTab} ${activeTab2 === "echo" ? styles.activeTab : ""}`,
                onClick: () => setActiveTab("echo"),
                children: "E.C.H.O"
              }
            ),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles.statusTab} ${activeTab2 === "systems" ? styles.activeTab : ""}`,
                onClick: () => setActiveTab("systems"),
                children: "NYX"
              }
            ),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles.statusTab} ${activeTab2 === "defense" ? styles.activeTab : ""}`,
                onClick: () => setActiveTab("defense"),
                children: "LEGION"
              }
            ),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles.statusTab} ${activeTab2 === "missions" ? styles.activeTab : ""}`,
                onClick: () => setActiveTab("missions"),
                children: "MISSIONS"
              }
            )
          ] }),
          /* @__PURE__ */ jsxs("div", { className: styles.tabContent, children: [
            activeTab2 === "echo" && renderEchoTab(),
            activeTab2 === "systems" && /* @__PURE__ */ jsx("div", { className: styles.systemsTab, children: /* @__PURE__ */ jsxs("div", { className: styles.lockedContent, children: [
              /* @__PURE__ */ jsx("p", { children: "This feature is currently locked." }),
              /* @__PURE__ */ jsx("p", { children: "Return to camp and await further instructions." })
            ] }) }),
            activeTab2 === "defense" && /* @__PURE__ */ jsx("div", { className: styles.defenseTab, children: /* @__PURE__ */ jsxs("div", { className: styles.lockedContent, children: [
              /* @__PURE__ */ jsx("p", { children: "This feature is currently locked." }),
              /* @__PURE__ */ jsx("p", { children: "Return to camp and await further instructions." })
            ] }) }),
            activeTab2 === "missions" && /* @__PURE__ */ jsxs("div", { className: styles.missionsTab, children: [
              /* @__PURE__ */ jsx("div", { className: styles.missionHeader, children: /* @__PURE__ */ jsxs("div", { className: styles.active, children: [
                /* @__PURE__ */ jsx("span", { className: styles.missionLabel, children: "ACTIVE:" }),
                /* @__PURE__ */ jsx("span", { className: styles.missionTitle, children: (activeMission == null ? void 0 : activeMission.title) || "Share on X" })
              ] }) }),
              /* @__PURE__ */ jsxs("div", { className: styles.missionContent, children: [
                /* @__PURE__ */ jsxs("div", { className: styles.availableMissions, children: [
                  /* @__PURE__ */ jsx("h4", { children: "AVAILABLE MISSIONS" }),
                  /* @__PURE__ */ jsxs("div", { className: styles.missionList, children: [
                    /* @__PURE__ */ jsxs("div", { className: `${styles.missionItem} ${styles.active}`, children: [
                      /* @__PURE__ */ jsxs("div", { className: styles.missionItemContent, children: [
                        /* @__PURE__ */ jsx("span", { className: styles.missionTitle, children: "Share on X" }),
                        /* @__PURE__ */ jsx("span", { className: styles.missionStatus, children: "ACTIVE" })
                      ] }),
                      /* @__PURE__ */ jsx("span", { className: styles.missionReward, children: "TBD NETHER AIRDROP" })
                    ] }),
                    /* @__PURE__ */ jsxs("div", { className: `${styles.missionItem} ${styles.blurred}`, children: [
                      /* @__PURE__ */ jsxs("div", { className: styles.missionItemContent, children: [
                        /* @__PURE__ */ jsx("span", { className: styles.missionTitle, children: "Mission 2" }),
                        /* @__PURE__ */ jsx("span", { className: styles.missionStatus, children: "LOCKED" })
                      ] }),
                      /* @__PURE__ */ jsx("span", { className: `${styles.missionReward} ${styles.blurred}`, children: "??? NETHER" })
                    ] }),
                    /* @__PURE__ */ jsxs("div", { className: `${styles.missionItem} ${styles.blurred}`, children: [
                      /* @__PURE__ */ jsxs("div", { className: styles.missionItemContent, children: [
                        /* @__PURE__ */ jsx("span", { className: styles.missionTitle, children: "Mission 3" }),
                        /* @__PURE__ */ jsx("span", { className: styles.missionStatus, children: "LOCKED" })
                      ] }),
                      /* @__PURE__ */ jsx("span", { className: `${styles.missionReward} ${styles.blurred}`, children: "??? NETHER" })
                    ] })
                  ] })
                ] }),
                /* @__PURE__ */ jsxs("div", { className: styles.missionDescription, children: [
                  /* @__PURE__ */ jsx("h4", { children: "MISSION BRIEF" }),
                  /* @__PURE__ */ jsx("p", { className: styles.missionText, children: `"Welcome, Camper, to your first trial. Tend the flame carefully. Share your Base Camp on X—let its glow haunt the realm. More souls drawn, more NETHER gained. Don't let it fade."` }),
                  /* @__PURE__ */ jsxs("div", { className: styles.missionInstructions, children: [
                    /* @__PURE__ */ jsx("h4", { children: "QUALIFICATION REQUIREMENTS" }),
                    /* @__PURE__ */ jsxs("ul", { children: [
                      /* @__PURE__ */ jsxs("li", { children: [
                        "Follow",
                        /* @__PURE__ */ jsx("span", { className: styles.highlight, children: "@Nullblock_io" })
                      ] }),
                      /* @__PURE__ */ jsxs("li", { children: [
                        "Tweet out the cashtag ",
                        /* @__PURE__ */ jsx("span", { className: styles.highlight, children: "$NETHER" })
                      ] }),
                      /* @__PURE__ */ jsxs("li", { children: [
                        "Include the official CA: ",
                        /* @__PURE__ */ jsx("span", { className: styles.highlight, children: "TBD" })
                      ] })
                    ] }),
                    /* @__PURE__ */ jsx("p", { className: styles.missionNote, children: "Airdrop amount will be determined by post engagement and creativity." })
                  ] }),
                  /* @__PURE__ */ jsxs("div", { className: styles.missionReward, children: [
                    /* @__PURE__ */ jsx("span", { className: styles.rewardLabel, children: "REWARD:" }),
                    /* @__PURE__ */ jsx("span", { className: styles.rewardValue, children: "TBD NETHER AIRDROP" })
                  ] }),
                  /* @__PURE__ */ jsxs("div", { className: styles.missionExpiration, children: [
                    /* @__PURE__ */ jsx("span", { className: styles.expirationLabel, children: "EXPIRES:" }),
                    /* @__PURE__ */ jsx("span", { className: styles.expirationValue, children: "TBD" })
                  ] })
                ] })
              ] })
            ] })
          ] })
        ] }) })
      ] }) }),
      selectedUplink && /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx("div", { className: styles.modalOverlay, onClick: handleCloseModal }),
        /* @__PURE__ */ jsxs("div", { className: styles.uplinkModal, children: [
          /* @__PURE__ */ jsxs("div", { className: styles.modalHeader, children: [
            /* @__PURE__ */ jsx("h3", { children: selectedUplink.name }),
            /* @__PURE__ */ jsx("button", { className: styles.closeButton, onClick: handleCloseModal, children: "×" })
          ] }),
          /* @__PURE__ */ jsxs("div", { className: styles.modalContent, children: [
            /* @__PURE__ */ jsxs("div", { className: styles.statusSection, children: [
              /* @__PURE__ */ jsx("h4", { children: "Status Overview" }),
              /* @__PURE__ */ jsx("div", { className: styles.statusGrid, children: selectedUplink.details.stats.map((stat, index) => /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: stat.label }),
                /* @__PURE__ */ jsx("div", { className: `${styles.value} ${stat.status || ""}`, children: stat.value })
              ] }, index)) })
            ] }),
            /* @__PURE__ */ jsxs("div", { className: styles.detailsSection, children: [
              /* @__PURE__ */ jsx("h4", { children: "Details" }),
              /* @__PURE__ */ jsx("p", { children: selectedUplink.details.description })
            ] })
          ] })
        ] })
      ] }),
      showLeaderboard && /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx("div", { className: styles.modalOverlay, onClick: handleCloseLeaderboard }),
        /* @__PURE__ */ jsxs("div", { className: styles.leaderboardModal, children: [
          /* @__PURE__ */ jsxs("div", { className: styles.modalHeader, children: [
            /* @__PURE__ */ jsx("h3", { children: "TOP CAMPERS" }),
            /* @__PURE__ */ jsx("button", { className: styles.closeButton, onClick: handleCloseLeaderboard, children: "×" })
          ] }),
          /* @__PURE__ */ jsx("div", { className: styles.legendWarning, children: /* @__PURE__ */ jsx("p", { children: "⚠️ These are the legends of the last Flame. The current Conduit has yet to awaken." }) }),
          /* @__PURE__ */ jsx("div", { className: styles.leaderboardGrid, children: leaderboardData.map((entry) => /* @__PURE__ */ jsxs("div", { className: styles.leaderboardCard, children: [
            /* @__PURE__ */ jsxs("div", { className: styles.cardHeader, children: [
              /* @__PURE__ */ jsxs("span", { className: styles.rank, children: [
                "#",
                entry.rank
              ] }),
              /* @__PURE__ */ jsx("span", { className: styles.camperId, children: entry.id })
            ] }),
            /* @__PURE__ */ jsxs("div", { className: styles.statusGrid, children: [
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "ASCENT" }),
                /* @__PURE__ */ jsx("div", { className: styles.value, children: entry.ascent })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "NETHER" }),
                /* @__PURE__ */ jsxs("div", { className: styles.value, children: [
                  "₦ ",
                  entry.nether
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "CACHE VALUE" }),
                /* @__PURE__ */ jsxs("div", { className: styles.value, children: [
                  "₦ ",
                  entry.cacheValue
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "MEMORIES" }),
                /* @__PURE__ */ jsx("div", { className: styles.value, children: entry.memories })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "MATRIX LEVEL" }),
                /* @__PURE__ */ jsx("div", { className: styles.value, children: entry.matrix.level })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "RARITY" }),
                /* @__PURE__ */ jsx("div", { className: styles.value, children: entry.matrix.rarity })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles.label, children: "STATUS" }),
                /* @__PURE__ */ jsx("div", { className: styles.value, children: entry.matrix.status })
              ] })
            ] })
          ] }, entry.id)) })
        ] })
      ] })
    ] });
  };
  const renderInventoryScreen = () => /* @__PURE__ */ jsxs("div", { className: styles.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles.hudTitle, children: "CACHE" }),
      /* @__PURE__ */ jsx("div", { className: styles.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.inventorySection, children: [
      /* @__PURE__ */ jsx("h3", { children: "WEAPONS" }),
      /* @__PURE__ */ jsxs("div", { className: styles.emptyState, children: [
        /* @__PURE__ */ jsx("p", { children: "No weapons found." }),
        /* @__PURE__ */ jsx("p", { children: "Complete missions to acquire gear." })
      ] })
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.inventorySection, children: [
      /* @__PURE__ */ jsx("h3", { children: "SUPPLIES" }),
      /* @__PURE__ */ jsxs("div", { className: styles.emptyState, children: [
        /* @__PURE__ */ jsx("p", { children: "Cache empty." }),
        /* @__PURE__ */ jsx("p", { children: "Gather resources to expand inventory." })
      ] })
    ] })
  ] });
  const renderCampaignScreen = () => /* @__PURE__ */ jsxs("div", { className: styles.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles.hudTitle, children: "CAMPAIGN" }),
      /* @__PURE__ */ jsx("div", { className: styles.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.realityContent, children: [
      /* @__PURE__ */ jsxs("div", { className: styles.realityStatus, children: [
        /* @__PURE__ */ jsx("h3", { children: "PROGRESS" }),
        /* @__PURE__ */ jsxs("p", { children: [
          "Current Level: ",
          /* @__PURE__ */ jsx("span", { children: "1" })
        ] }),
        /* @__PURE__ */ jsxs("p", { children: [
          "Completion: ",
          /* @__PURE__ */ jsx("span", { children: "0%" })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.missions, children: [
        /* @__PURE__ */ jsx("h3", { children: "OBJECTIVES" }),
        /* @__PURE__ */ jsx("p", { className: styles.placeholder, children: "No active missions" }),
        /* @__PURE__ */ jsx("p", { className: styles.placeholder, children: "Complete training to begin" })
      ] })
    ] })
  ] });
  const renderLabScreen = () => /* @__PURE__ */ jsxs("div", { className: styles.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles.hudTitle, children: "LAB" }),
      /* @__PURE__ */ jsx("div", { className: styles.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.interfaceContent, children: [
      /* @__PURE__ */ jsxs("div", { className: styles.interfaceSection, children: [
        /* @__PURE__ */ jsx("h3", { children: "SYSTEMS" }),
        /* @__PURE__ */ jsxs("p", { children: [
          "Phantom: ",
          /* @__PURE__ */ jsx("span", { className: styles.connected, children: "CONNECTED" })
        ] }),
        /* @__PURE__ */ jsxs("p", { children: [
          "Core: ",
          /* @__PURE__ */ jsx("span", { className: styles.initializing, children: "INITIALIZING" })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles.interfaceSection, children: [
        /* @__PURE__ */ jsx("h3", { children: "CONFIGURATIONS" }),
        /* @__PURE__ */ jsx("p", { className: styles.placeholder, children: "No active modifications" }),
        /* @__PURE__ */ jsx("p", { className: styles.placeholder, children: "Run diagnostics to begin" })
      ] })
    ] })
  ] });
  const renderChambersScreen = () => /* @__PURE__ */ jsxs("div", { className: styles.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles.hudTitle, children: "ECHO CHAMBERS" }),
      /* @__PURE__ */ jsx("div", { className: styles.headerDivider })
    ] }),
    /* @__PURE__ */ jsx("div", { className: styles.campContent, children: /* @__PURE__ */ jsxs("div", { style: { textAlign: "center", marginTop: "2rem" }, children: [
      /* @__PURE__ */ jsx("h3", { children: "Welcome to the ECHO Chambers" }),
      /* @__PURE__ */ jsx("p", { children: "This is the default screen. Connect your wallet to unlock CAMP and other features." })
    ] }) }),
    /* @__PURE__ */ jsx("div", { className: styles.echoBatChamberLogoWrapper, children: /* @__PURE__ */ jsx("img", { src: echoBatBlack, alt: "ECHO Bat", className: styles.echoBatChamberLogo }) })
  ] });
  const renderScreen = () => {
    if (!unlockedScreens.includes(screen)) {
      return renderLockedScreen();
    }
    switch (screen) {
      case "chambers":
        return renderChambersScreen();
      case "camp":
        return renderCampScreen();
      case "inventory":
        return renderInventoryScreen();
      case "campaign":
        return renderCampaignScreen();
      case "lab":
        return renderLabScreen();
      default:
        return null;
    }
  };
  return /* @__PURE__ */ jsxs("div", { className: `${styles.echoContainer} ${styles[theme]}`, children: [
    renderControlScreen(),
    /* @__PURE__ */ jsx("div", { className: styles.hudWindow, children: renderScreen() })
  ] });
};
const Home = () => {
  const [walletConnected, setWalletConnected] = useState(false);
  const [publicKey, setPublicKey] = useState(null);
  const [showEcho, setShowEcho] = useState(true);
  const [messages, setMessages] = useState([]);
  const [messageIndex, setMessageIndex] = useState(0);
  const [hasPhantom, setHasPhantom] = useState(false);
  const [currentRoom, setCurrentRoom] = useState("/logs");
  useState(false);
  const [echoScreenSelected, setEchoScreenSelected] = useState(false);
  const [currentTheme, setCurrentTheme] = useState("light");
  const [showConnectButton, setShowConnectButton] = useState(false);
  const [textComplete, setTextComplete] = useState(false);
  useEffect(() => {
    const savedPublicKey = localStorage.getItem("walletPublickey");
    const lastAuth = localStorage.getItem("lastAuthTime");
    localStorage.getItem("hasSeenEcho");
    const savedTheme = localStorage.getItem("currentTheme");
    if (savedPublicKey && lastAuth && isSessionValid()) {
      setPublicKey(savedPublicKey);
      setWalletConnected(true);
    } else {
      setWalletConnected(false);
      setPublicKey(null);
    }
    if (savedTheme) {
      setCurrentTheme(savedTheme);
    }
  }, []);
  useEffect(() => {
    console.log("Text complete effect triggered:", { textComplete, walletConnected, showEcho });
    if (textComplete && !walletConnected && !showEcho) {
      console.log("Setting showConnectButton to true");
      setShowConnectButton(true);
    }
  }, [textComplete, walletConnected, showEcho]);
  useEffect(() => {
    if (!walletConnected && !showEcho) {
      const fallbackTimer = setTimeout(() => {
        console.log("Fallback timer triggered - showing connect button");
        setShowConnectButton(true);
      }, 5e3);
      return () => clearTimeout(fallbackTimer);
    }
  }, [walletConnected, showEcho]);
  const addMessage = (message) => {
    setMessages((prev) => [...prev, message]);
  };
  const handleUserInput = async (input) => {
    addMessage({
      id: messages.length + 1,
      text: input,
      type: "user"
    });
    try {
      const response = await fetch("http://localhost:8000/api/command", {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          command: input,
          room: currentRoom
        })
      });
      if (!response.ok) {
        throw new Error("Failed to process command");
      }
      const data = await response.json();
      if (input.toLowerCase() === "/clear") {
        setMessages([]);
        return;
      }
      data.messages.forEach((msg, index) => {
        setTimeout(() => {
          if (msg.type === "action") {
            if (msg.action === "connect_wallet") {
              addMessage({
                id: messages.length + 2 + index,
                text: msg.text,
                type: msg.type,
                action: manualConnect,
                actionText: "Connect"
              });
            } else if (msg.action === "disconnect_wallet") {
              addMessage({
                id: messages.length + 2 + index,
                text: msg.text,
                type: msg.type,
                action: handleDisconnect,
                actionText: "Disconnect"
              });
            }
          } else {
            addMessage({
              id: messages.length + 2 + index,
              text: msg.text,
              type: msg.type
            });
          }
        }, 500 * (index + 1));
      });
    } catch (error) {
      console.error("Error processing command:", error);
      addMessage({
        id: messages.length + 2,
        text: "Error: Command processing failed",
        type: "critical"
      });
    }
  };
  const handleRoomChange = (room) => {
    if (room.startsWith("/theme/")) {
      const themeId = room.split("/theme/")[1];
      setCurrentTheme(themeId);
      const themeName = themeId === "null" ? "NULL" : "LIGHT";
      addMessage({
        id: messages.length + 1,
        text: `System: Theme changed to ${themeName}`,
        type: "update"
      });
      return;
    }
    setCurrentRoom(room);
    if (room.startsWith("/echo")) {
      setEchoScreenSelected(true);
    }
    addMessage({
      id: messages.length + 1,
      text: `System: Switched to ${room}`,
      type: "update"
    });
  };
  useEffect(() => {
    var _a;
    const phantomExists = "phantom" in window && ((_a = window.phantom) == null ? void 0 : _a.solana);
    setHasPhantom(!!phantomExists);
    if (phantomExists) {
      checkWalletConnection();
    }
    const getInitialMessages = () => {
      const baseMessages = [
        {
          id: 1,
          text: "System: Initializing...",
          type: "message"
        }
      ];
      if (phantomExists) {
        return [
          ...baseMessages,
          {
            id: 2,
            text: "System: Interface detected.",
            type: "message"
          },
          {
            id: 3,
            text: "System Update: Awaiting connection.",
            type: "update"
          },
          {
            id: 4,
            text: "Connect",
            type: "action",
            action: manualConnect,
            actionText: "Connect"
          }
        ];
      } else {
        return [
          ...baseMessages,
          {
            id: 2,
            text: "Error: Interface not found.",
            type: "critical"
          },
          {
            id: 3,
            text: "System: Interface required for access.",
            type: "message"
          },
          {
            id: 4,
            text: "Acquire Interface",
            type: "action",
            action: () => window.open("https://phantom.app/", "_blank"),
            actionText: "Install Interface"
          }
        ];
      }
    };
    const initialMessages = getInitialMessages();
    const displayNextMessage = () => {
      if (messageIndex < initialMessages.length) {
        addMessage(initialMessages[messageIndex]);
        setMessageIndex((prev) => prev + 1);
      }
    };
    if (!walletConnected && messageIndex < initialMessages.length) {
      const timer = setTimeout(displayNextMessage, messageIndex * 400);
      return () => clearTimeout(timer);
    }
  }, [messageIndex, walletConnected]);
  const requestSignature = async (provider, publicKey2) => {
    try {
      const message = `Authenticate ECHO Interface
Timestamp: ${Date.now()}`;
      const encodedMessage = new TextEncoder().encode(message);
      await provider.signMessage(encodedMessage, "utf8");
    } catch (error) {
      throw new Error("Authentication failed");
    }
  };
  const SESSION_TIMEOUT = 30 * 60 * 1e3;
  const isSessionValid = () => {
    const lastAuth = localStorage.getItem("lastAuthTime");
    if (!lastAuth) return false;
    const timeSinceAuth = Date.now() - parseInt(lastAuth);
    return timeSinceAuth < SESSION_TIMEOUT;
  };
  const updateAuthTime = () => {
    localStorage.setItem("lastAuthTime", Date.now().toString());
  };
  const checkWalletConnection = async () => {
    var _a;
    if ("phantom" in window) {
      const provider = (_a = window.phantom) == null ? void 0 : _a.solana;
      if (provider) {
        const savedPublicKey = localStorage.getItem("walletPublickey");
        const lastAuth = localStorage.getItem("lastAuthTime");
        if (savedPublicKey && lastAuth && isSessionValid()) {
          try {
            await provider.connect({ onlyIfTrusted: true });
            setPublicKey(savedPublicKey);
            setWalletConnected(true);
            localStorage.setItem("walletPublickey", savedPublicKey);
            localStorage.setItem("hasSeenEcho", "true");
            updateAuthTime();
            addMessage({
              id: messages.length + 1,
              text: "System: Connected. Loading interface...",
              type: "message"
            });
          } catch (error) {
            console.log("Auto-reconnect failed:", error);
          }
        }
        localStorage.removeItem("walletPublickey");
        localStorage.removeItem("lastAuthTime");
        localStorage.removeItem("hasSeenEcho");
        setWalletConnected(false);
        setPublicKey(null);
      }
    }
  };
  const manualConnect = async () => {
    var _a;
    if ("phantom" in window) {
      const provider = (_a = window.phantom) == null ? void 0 : _a.solana;
      if (provider) {
        try {
          const resp = await provider.connect();
          const walletPubKey = resp.publicKey.toString();
          await requestSignature(provider, walletPubKey);
          setPublicKey(walletPubKey);
          setWalletConnected(true);
          localStorage.setItem("walletPublickey", walletPubKey);
          localStorage.setItem("hasSeenEcho", "true");
          updateAuthTime();
          addMessage({
            id: messages.length + 1,
            text: "System: Connected. Loading interface...",
            type: "message"
          });
        } catch (error) {
          console.error("Connection error:", error);
          localStorage.removeItem("walletPublickey");
          localStorage.removeItem("lastAuthTime");
          localStorage.removeItem("hasSeenEcho");
          setWalletConnected(false);
          setPublicKey(null);
          addMessage({
            id: messages.length + 1,
            text: "Error: Authentication failed. Retry required.",
            type: "critical"
          });
        }
      }
    }
  };
  const handleDisconnect = async () => {
    var _a;
    if ("phantom" in window) {
      const provider = (_a = window.phantom) == null ? void 0 : _a.solana;
      if (provider) {
        try {
          await provider.disconnect();
          setWalletConnected(false);
          setPublicKey(null);
          localStorage.removeItem("walletPublickey");
          localStorage.removeItem("lastAuthTime");
          localStorage.removeItem("hasSeenEcho");
          setMessages([{
            id: 1,
            text: "System: Interface disconnected.",
            type: "message"
          }, {
            id: 2,
            text: "System Alert: Session terminated. Re-authentication required.",
            type: "alert"
          }]);
          setMessageIndex(0);
          setTextComplete(false);
          setShowConnectButton(false);
        } catch (error) {
          console.error("Error disconnecting from Phantom:", error);
        }
      }
    }
  };
  return /* @__PURE__ */ jsxs("div", { className: `${styles$2.appContainer} ${styles$2[`theme-${currentTheme}`]}`, children: [
    /* @__PURE__ */ jsx("div", { className: styles$2.backgroundImage }),
    /* @__PURE__ */ jsx(StarsCanvas, { theme: currentTheme }),
    /* @__PURE__ */ jsx("div", { className: `${styles$2.scene} ${showEcho ? styles$2.echoActive : ""}` }),
    showEcho && /* @__PURE__ */ jsx(
      Echo,
      {
        publicKey,
        onDisconnect: handleDisconnect,
        theme: currentTheme,
        onClose: () => {
          setShowEcho(false);
          setTextComplete(false);
          setShowConnectButton(false);
        },
        onThemeChange: (theme) => {
          if (theme === "cyber") {
            setCurrentTheme("null");
          } else {
            setCurrentTheme(theme);
          }
        },
        messages,
        onUserInput: handleUserInput,
        currentRoom,
        onRoomChange: handleRoomChange
      }
    )
  ] });
};
export {
  Home as default
};
