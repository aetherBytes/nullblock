import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { Suspense, useRef, useState, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { Preload, Points, PointMaterial } from "@react-three/drei";
import * as random from "maath/random/dist/maath-random.esm.js";
import axios from "axios";
const backgroundImage$1 = "_backgroundImage_10dyy_15";
const button$1 = "_button_10dyy_59";
const alertText$1 = "_alertText_10dyy_76";
const scene$1 = "_scene_10dyy_82";
const echoActive$1 = "_echoActive_10dyy_92";
const fire$1 = "_fire_10dyy_92";
const nyx$1 = "_nyx_10dyy_95";
const campfireFlicker$1 = "_campfireFlicker_10dyy_1";
const campfireGlow$1 = "_campfireGlow_10dyy_1";
const nyxGlow$1 = "_nyxGlow_10dyy_1";
const robot$1 = "_robot_10dyy_178";
const trader1$1 = "_trader1_10dyy_178";
const trader2$1 = "_trader2_10dyy_178";
const trader3$1 = "_trader3_10dyy_178";
const appContainer$1 = "_appContainer_10dyy_182";
const lightFireFlicker$1 = "_lightFireFlicker_10dyy_1";
const lightFireGlow$1 = "_lightFireGlow_10dyy_1";
const connectButtonContainer$1 = "_connectButtonContainer_10dyy_264";
const fadeIn$1 = "_fadeIn_10dyy_1";
const connectButton$1 = "_connectButton_10dyy_264";
const styles$3 = {
  backgroundImage: backgroundImage$1,
  button: button$1,
  alertText: alertText$1,
  scene: scene$1,
  echoActive: echoActive$1,
  fire: fire$1,
  nyx: nyx$1,
  campfireFlicker: campfireFlicker$1,
  campfireGlow: campfireGlow$1,
  nyxGlow: nyxGlow$1,
  robot: robot$1,
  trader1: trader1$1,
  trader2: trader2$1,
  trader3: trader3$1,
  appContainer: appContainer$1,
  "theme-null": "_theme-null_10dyy_189",
  "theme-light": "_theme-light_10dyy_194",
  lightFireFlicker: lightFireFlicker$1,
  lightFireGlow: lightFireGlow$1,
  connectButtonContainer: connectButtonContainer$1,
  fadeIn: fadeIn$1,
  connectButton: connectButton$1
};
const starsCanvas = "_starsCanvas_4g78i_1";
const matrix$1 = "_matrix_4g78i_10";
const cyber$1 = "_cyber_4g78i_14";
const light$2 = "_light_4g78i_18";
const styles$2 = {
  starsCanvas,
  matrix: matrix$1,
  cyber: cyber$1,
  light: light$2
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
  return /* @__PURE__ */ jsx("div", { className: `${styles$2.starsCanvas} ${styles$2[theme]}`, children: /* @__PURE__ */ jsxs(Canvas, { camera: { position: [0, 0, 1] }, children: [
    /* @__PURE__ */ jsx(Suspense, { fallback: null, children: /* @__PURE__ */ jsx(Stars, { theme }) }),
    /* @__PURE__ */ jsx(Preload, { all: true })
  ] }) });
};
const backgroundImage = "_backgroundImage_1hmof_16";
const button = "_button_1hmof_60";
const alertText = "_alertText_1hmof_77";
const scene = "_scene_1hmof_83";
const echoActive = "_echoActive_1hmof_93";
const fire = "_fire_1hmof_93";
const nyx = "_nyx_1hmof_96";
const campfireFlicker = "_campfireFlicker_1hmof_1";
const campfireGlow = "_campfireGlow_1hmof_1";
const nyxGlow = "_nyxGlow_1hmof_1";
const robot = "_robot_1hmof_179";
const trader1 = "_trader1_1hmof_179";
const trader2 = "_trader2_1hmof_179";
const trader3 = "_trader3_1hmof_179";
const appContainer = "_appContainer_1hmof_183";
const lightFireFlicker = "_lightFireFlicker_1hmof_1";
const lightFireGlow = "_lightFireGlow_1hmof_1";
const connectButtonContainer = "_connectButtonContainer_1hmof_265";
const fadeIn = "_fadeIn_1hmof_1";
const connectButton = "_connectButton_1hmof_265";
const echoContainer = "_echoContainer_1hmof_330";
const hudWindow = "_hudWindow_1hmof_343";
const controlPanel = "_controlPanel_1hmof_348";
const controlButton = "_controlButton_1hmof_351";
const disabled = "_disabled_1hmof_354";
const active = "_active_1hmof_357";
const userProfile = "_userProfile_1hmof_365";
const profileLabel = "_profileLabel_1hmof_368";
const profileValue = "_profileValue_1hmof_371";
const statusIndicator = "_statusIndicator_1hmof_374";
const inactive = "_inactive_1hmof_377";
const alertButton = "_alertButton_1hmof_380";
const expandButton = "_expandButton_1hmof_386";
const disconnectButton = "_disconnectButton_1hmof_392";
const matrix = "_matrix_1hmof_398";
const cyber = "_cyber_1hmof_456";
const light$1 = "_light_1hmof_514";
const softEcho = "_softEcho_1hmof_1";
const corePulse = "_corePulse_1hmof_1";
const scanlineGlow = "_scanlineGlow_1hmof_1";
const scanlineWaver = "_scanlineWaver_1hmof_1";
const hudTitle = "_hudTitle_1hmof_614";
const bottomLeftInfo = "_bottomLeftInfo_1hmof_660";
const bottomRightInfo = "_bottomRightInfo_1hmof_660";
const verticalNavbar = "_verticalNavbar_1hmof_673";
const homeButton = "_homeButton_1hmof_684";
const homeIcon = "_homeIcon_1hmof_702";
const socialButton = "_socialButton_1hmof_708";
const socialIcon = "_socialIcon_1hmof_729";
const navDivider = "_navDivider_1hmof_752";
const navButton = "_navButton_1hmof_767";
const locked = "_locked_1hmof_808";
const lockIcon = "_lockIcon_1hmof_819";
const nexus = "_nexus_1hmof_824";
const hudScreen = "_hudScreen_1hmof_824";
const settingsScreen = "_settingsScreen_1hmof_824";
const walletInfo = "_walletInfo_1hmof_852";
const nexusActions = "_nexusActions_1hmof_892";
const headerContainer = "_headerContainer_1hmof_958";
const architectTitle = "_architectTitle_1hmof_969";
const leaderboardContainer = "_leaderboardContainer_1hmof_979";
const leaderboardTitle = "_leaderboardTitle_1hmof_991";
const leaderboardButton = "_leaderboardButton_1hmof_1001";
const leaderboardList = "_leaderboardList_1hmof_1020";
const leaderboardItems = "_leaderboardItems_1hmof_1046";
const scrollLeaderboard = "_scrollLeaderboard_1hmof_1";
const leaderboardItem = "_leaderboardItem_1hmof_1046";
const rank = "_rank_1hmof_1072";
const camperId = "_camperId_1hmof_1078";
const matrixLevel = "_matrixLevel_1hmof_1087";
const headerDivider = "_headerDivider_1hmof_1097";
const campGrid = "_campGrid_1hmof_1102";
const campAnalysis = "_campAnalysis_1hmof_1110";
const diagnosticsContainer = "_diagnosticsContainer_1hmof_1120";
const containerTitle = "_containerTitle_1hmof_1131";
const diagnosticsHeader = "_diagnosticsHeader_1hmof_1142";
const diagnosticsContent = "_diagnosticsContent_1hmof_1155";
const diagnosticsList = "_diagnosticsList_1hmof_1178";
const diagnosticsItem = "_diagnosticsItem_1hmof_1185";
const itemLabel = "_itemLabel_1hmof_1226";
const itemValue = "_itemValue_1hmof_1229";
const campContent = "_campContent_1hmof_1260";
const campStatus = "_campStatus_1hmof_1282";
const statusCard = "_statusCard_1hmof_1288";
const statusHeaderContainer = "_statusHeaderContainer_1hmof_1299";
const statusTabs = "_statusTabs_1hmof_1320";
const statusTab = "_statusTab_1hmof_1320";
const activeTab = "_activeTab_1hmof_1343";
const tabContent = "_tabContent_1hmof_1356";
const scanline = "_scanline_1hmof_1";
const statusContent = "_statusContent_1hmof_1377";
const vitalsContainer = "_vitalsContainer_1hmof_1398";
const vitalItem = "_vitalItem_1hmof_1420";
const vitalValue = "_vitalValue_1hmof_1448";
const vitalLabel = "_vitalLabel_1hmof_1451";
const infoButton = "_infoButton_1hmof_1482";
const pulse = "_pulse_1hmof_1";
const ascentDetails = "_ascentDetails_1hmof_1513";
const shimmer = "_shimmer_1hmof_1";
const ascentDescription = "_ascentDescription_1hmof_1532";
const progressBar = "_progressBar_1hmof_1538";
const progressFill = "_progressFill_1hmof_1556";
const accoladesContainer = "_accoladesContainer_1hmof_1573";
const accoladesTitle = "_accoladesTitle_1hmof_1576";
const accoladesList = "_accoladesList_1hmof_1593";
const visible = "_visible_1hmof_1609";
const blurred = "_blurred_1hmof_1618";
const missionsTab = "_missionsTab_1hmof_1628";
const systemsTab = "_systemsTab_1hmof_1628";
const defenseTab = "_defenseTab_1hmof_1628";
const uplinkTab = "_uplinkTab_1hmof_1628";
const missionHeader = "_missionHeader_1hmof_1640";
const missionContent = "_missionContent_1hmof_1650";
const availableMissions = "_availableMissions_1hmof_1662";
const missionList = "_missionList_1hmof_1689";
const missionItem = "_missionItem_1hmof_1694";
const missionItemContent = "_missionItemContent_1hmof_1718";
const missionTitle = "_missionTitle_1hmof_1725";
const missionStatus = "_missionStatus_1hmof_1733";
const missionReward = "_missionReward_1hmof_1737";
const missionDescription = "_missionDescription_1hmof_1749";
const missionText = "_missionText_1hmof_1781";
const missionInstructions = "_missionInstructions_1hmof_1791";
const highlight = "_highlight_1hmof_1817";
const missionNote = "_missionNote_1hmof_1821";
const missionExpiration = "_missionExpiration_1hmof_1830";
const rewardLabel = "_rewardLabel_1hmof_1836";
const expirationLabel = "_expirationLabel_1hmof_1836";
const rewardValue = "_rewardValue_1hmof_1843";
const expirationValue = "_expirationValue_1hmof_1843";
const activeMissionDetails = "_activeMissionDetails_1hmof_1850";
const systemsContent = "_systemsContent_1hmof_1876";
const defenseContent = "_defenseContent_1hmof_1876";
const uplinkContent = "_uplinkContent_1hmof_1876";
const systemsList = "_systemsList_1hmof_1901";
const systemItem = "_systemItem_1hmof_1907";
const systemName = "_systemName_1hmof_1916";
const defenseStatus = "_defenseStatus_1hmof_1920";
const uplinkStatus = "_uplinkStatus_1hmof_1920";
const defenseDescription = "_defenseDescription_1hmof_1925";
const uplinkDescription = "_uplinkDescription_1hmof_1925";
const tabs = "_tabs_1hmof_1930";
const tab = "_tab_1hmof_1356";
const profileItem = "_profileItem_1hmof_2013";
const label = "_label_1hmof_2025";
const value = "_value_1hmof_2036";
const common = "_common_1hmof_2046";
const uncommon = "_uncommon_1hmof_2049";
const rare = "_rare_1hmof_2052";
const epic = "_epic_1hmof_2055";
const legendary = "_legendary_1hmof_2058";
const none = "_none_1hmof_2083";
const collapsedButton = "_collapsedButton_1hmof_2088";
const withEcho = "_withEcho_1hmof_2088";
const ascentLine = "_ascentLine_1hmof_2091";
const lockedContent = "_lockedContent_1hmof_2209";
const statusContainer = "_statusContainer_1hmof_2259";
const statusLabel = "_statusLabel_1hmof_2270";
const echoContent = "_echoContent_1hmof_2284";
const echoStatus = "_echoStatus_1hmof_2311";
const browserInfo = "_browserInfo_1hmof_2319";
const browserLabel = "_browserLabel_1hmof_2323";
const browserValue = "_browserValue_1hmof_2327";
const echoMessage = "_echoMessage_1hmof_2331";
const disconnectedContent = "_disconnectedContent_1hmof_2345";
const extensionPrompt = "_extensionPrompt_1hmof_2363";
const extensionLinks = "_extensionLinks_1hmof_2383";
const extensionButton = "_extensionButton_1hmof_2390";
const uplinkItem = "_uplinkItem_1hmof_2405";
const uplinkIcon = "_uplinkIcon_1hmof_2427";
const uplinkInfo = "_uplinkInfo_1hmof_2436";
const uplinkName = "_uplinkName_1hmof_2440";
const pending = "_pending_1hmof_2459";
const uplinkModal = "_uplinkModal_1hmof_2468";
const modalHeader = "_modalHeader_1hmof_2493";
const closeButton = "_closeButton_1hmof_2508";
const modalContent = "_modalContent_1hmof_2521";
const statusSection = "_statusSection_1hmof_2524";
const statusGrid = "_statusGrid_1hmof_2534";
const statusItem = "_statusItem_1hmof_2539";
const detailsSection = "_detailsSection_1hmof_2561";
const modalOverlay = "_modalOverlay_1hmof_2575";
const leaderboardModal = "_leaderboardModal_1hmof_2586";
const legendWarning = "_legendWarning_1hmof_2620";
const leaderboardGrid = "_leaderboardGrid_1hmof_2635";
const leaderboardCard = "_leaderboardCard_1hmof_2642";
const cardHeader = "_cardHeader_1hmof_2655";
const vitalsGrid = "_vitalsGrid_1hmof_2723";
const addLinkButton = "_addLinkButton_1hmof_2730";
const styles$1 = {
  backgroundImage,
  button,
  alertText,
  scene,
  echoActive,
  fire,
  nyx,
  campfireFlicker,
  campfireGlow,
  nyxGlow,
  robot,
  trader1,
  trader2,
  trader3,
  appContainer,
  "theme-null": "_theme-null_1hmof_190",
  "theme-light": "_theme-light_1hmof_195",
  lightFireFlicker,
  lightFireGlow,
  connectButtonContainer,
  fadeIn,
  connectButton,
  echoContainer,
  "null": "_null_1hmof_343",
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
  light: light$1,
  softEcho,
  corePulse,
  scanlineGlow,
  scanlineWaver,
  hudTitle,
  bottomLeftInfo,
  bottomRightInfo,
  verticalNavbar,
  homeButton,
  homeIcon,
  socialButton,
  socialIcon,
  navDivider,
  navButton,
  locked,
  lockIcon,
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
  addLinkButton
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
const nyxImage = "/assets/nyx_head-h4za-lDt.png";
const Echo = ({
  publicKey,
  onDisconnect,
  theme = "light",
  onClose,
  onThemeChange
}) => {
  var _a;
  const [screen, setScreen] = useState("camp");
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
  const unlockedScreens = ["camp"];
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
  const handleScreenChange = (newScreen) => {
    if (unlockedScreens.includes(newScreen)) {
      setScreen(newScreen);
    }
  };
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
  const handleDisconnect = async () => {
    var _a2;
    if ("phantom" in window) {
      const provider = (_a2 = window.phantom) == null ? void 0 : _a2.solana;
      if (provider) {
        try {
          await provider.disconnect();
          localStorage.removeItem("walletPublickey");
          localStorage.removeItem("hasSeenEcho");
          localStorage.removeItem("chatCollapsedState");
          onDisconnect();
        } catch (error) {
          console.error("Error disconnecting from Phantom:", error);
        }
      }
    }
  };
  const handleCloseModal = () => {
    setSelectedUplink(null);
  };
  const handleLeaderboardClick = () => {
    setShowLeaderboard(true);
  };
  const handleCloseLeaderboard = () => {
    setShowLeaderboard(false);
  };
  const renderControlScreen = () => /* @__PURE__ */ jsxs("nav", { className: styles$1.verticalNavbar, children: [
    /* @__PURE__ */ jsx(
      "button",
      {
        onClick: () => setActiveTab("systems"),
        className: styles$1.homeButton,
        children: /* @__PURE__ */ jsx("img", { src: nyxImage, alt: "Home", className: styles$1.homeIcon })
      }
    ),
    /* @__PURE__ */ jsx(
      "a",
      {
        href: "https://x.com/Nullblock_io",
        target: "_blank",
        rel: "noopener noreferrer",
        className: styles$1.socialButton,
        children: /* @__PURE__ */ jsx("img", { src: xLogo, alt: "X", className: styles$1.socialIcon })
      }
    ),
    /* @__PURE__ */ jsx("div", { className: styles$1.navDivider }),
    /* @__PURE__ */ jsx("button", { onClick: () => handleScreenChange("camp"), className: styles$1.navButton, children: "CAMP" }),
    /* @__PURE__ */ jsxs(
      "button",
      {
        onClick: () => handleScreenChange("inventory"),
        className: `${styles$1.navButton} ${!unlockedScreens.includes("inventory") ? styles$1.locked : ""}`,
        disabled: !unlockedScreens.includes("inventory"),
        children: [
          "CACHE ",
          /* @__PURE__ */ jsx("span", { className: styles$1.lockIcon, children: "[LOCKED]" })
        ]
      }
    ),
    /* @__PURE__ */ jsxs(
      "button",
      {
        onClick: () => handleScreenChange("campaign"),
        className: `${styles$1.navButton} ${!unlockedScreens.includes("campaign") ? styles$1.locked : ""}`,
        disabled: !unlockedScreens.includes("campaign"),
        children: [
          "CAMPAIGN ",
          /* @__PURE__ */ jsx("span", { className: styles$1.lockIcon, children: "[LOCKED]" })
        ]
      }
    ),
    /* @__PURE__ */ jsxs(
      "button",
      {
        onClick: () => handleScreenChange("lab"),
        className: `${styles$1.navButton} ${!unlockedScreens.includes("lab") ? styles$1.locked : ""}`,
        disabled: !unlockedScreens.includes("lab"),
        children: [
          "LAB ",
          /* @__PURE__ */ jsx("span", { className: styles$1.lockIcon, children: "[LOCKED]" })
        ]
      }
    ),
    /* @__PURE__ */ jsx("button", { onClick: handleDisconnect, className: styles$1.navButton, children: "DISCONNECT" })
  ] });
  const renderUserProfile = () => {
    var _a2;
    return /* @__PURE__ */ jsxs("div", { className: styles$1.userProfile, children: [
      /* @__PURE__ */ jsxs("div", { className: styles$1.profileItem, children: [
        /* @__PURE__ */ jsx("span", { className: styles$1.label, children: "ID:" }),
        /* @__PURE__ */ jsx("span", { className: styles$1.value, children: userProfile2.id })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles$1.label, children: [
          "ASCENT:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles$1.infoButton,
              onClick: () => setShowAscentDetails(!showAscentDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs("div", { className: styles$1.ascentContainer, children: [
          /* @__PURE__ */ jsx("span", { className: styles$1.value, children: "Net Dweller: 1" }),
          /* @__PURE__ */ jsx("div", { className: styles$1.progressBar, children: /* @__PURE__ */ jsx(
            "div",
            {
              className: styles$1.progressFill,
              style: { width: `${35}%` }
            }
          ) })
        ] }),
        showAscentDetails && /* @__PURE__ */ jsxs("div", { className: styles$1.ascentDetails, children: [
          /* @__PURE__ */ jsx("div", { className: styles$1.ascentDescription, children: "A digital lurker extraordinaire! You've mastered the art of watching from the shadows, observing the chaos without ever dipping your toes in. Like a cat watching a laser pointer, you're fascinated but paralyzed by indecision. At least you're not the one getting your digital assets rekt!" }),
          /* @__PURE__ */ jsx("div", { className: styles$1.progressText, children: "35% to next level" }),
          /* @__PURE__ */ jsxs("div", { className: styles$1.accoladesContainer, children: [
            /* @__PURE__ */ jsx("div", { className: styles$1.accoladesTitle, children: "ACCOLADES" }),
            /* @__PURE__ */ jsxs("ul", { className: styles$1.accoladesList, children: [
              /* @__PURE__ */ jsx("li", { className: styles$1.visible, children: "First Connection" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.visible, children: "Wallet Initiated" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.visible, children: "Basic Navigation" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.blurred, children: "Token Discovery" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.blurred, children: "Transaction Initiate" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.blurred, children: "Network Explorer" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.blurred, children: "Data Collector" }),
              /* @__PURE__ */ jsx("li", { className: styles$1.blurred, children: "Interface Familiar" })
            ] })
          ] })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles$1.label, children: [
          "NETHER:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles$1.infoButton,
              onClick: () => setShowNectarDetails(!showNectarDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs("span", { className: styles$1.value, children: [
          "₦ ",
          ((_a2 = userProfile2.nether) == null ? void 0 : _a2.toFixed(2)) || "N/A"
        ] }),
        showNectarDetails && /* @__PURE__ */ jsx("div", { className: styles$1.ascentDetails, children: /* @__PURE__ */ jsx("div", { className: styles$1.ascentDescription, children: "NETHER: Magic internet money from the void. Born from nothing, worth everything, and somehow gaining value by the second. The integration has passed the event horizon - good luck trying to spend it. Warning: Prolonged exposure may cause reality distortion and an irresistible urge to dive deeper into the code." }) })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles$1.label, children: [
          "cache value:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles$1.infoButton,
              onClick: () => setShowCacheValueDetails(!showCacheValueDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsx("span", { className: styles$1.value, children: "₦ N/A" }),
        showCacheValueDetails && /* @__PURE__ */ jsx("div", { className: styles$1.ascentDetails, children: /* @__PURE__ */ jsx("div", { className: styles$1.ascentDescription, children: "Cache Value: Your digital treasure trove, evaluated by our ever-watchful procurement agents. This is the total worth of all valuable assets in your wallet - coins, tokens, and other digital goodies that caught our eye. Coming soon: Categories for services, participant offerings, biological enhancements, and agent capabilities. Think of it as your personal inventory of everything worth something in the Nullblock universe. Don't spend it all in one place!" }) })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles$1.label, children: [
          "MEMORIES:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles$1.infoButton,
              onClick: () => setShowMemoriesDetails(!showMemoriesDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsx("span", { className: styles$1.value, children: userProfile2.memories }),
        showMemoriesDetails && /* @__PURE__ */ jsx("div", { className: styles$1.ascentDetails, children: /* @__PURE__ */ jsx("div", { className: styles$1.ascentDescription, children: "Oh no, no memories found? Wait... who are you? Where am I? *checks digital wallet* Ah, right - another poor...soul. You need to collect the artifacts that tell your story in the Nullblock universe. Each memory is a unique representation of your achievements, collectibles, and digital identity. Collect them all to unlock the secret of why you're here... or don't, I'm not your digital conscience." }) })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.profileItem, children: [
        /* @__PURE__ */ jsxs("span", { className: styles$1.label, children: [
          "E.C:",
          /* @__PURE__ */ jsx(
            "button",
            {
              className: styles$1.infoButton,
              onClick: () => setShowEmberConduitDetails(!showEmberConduitDetails),
              children: "?"
            }
          )
        ] }),
        /* @__PURE__ */ jsx("span", { className: styles$1.value, children: userProfile2.matrix.status }),
        showEmberConduitDetails && /* @__PURE__ */ jsx("div", { className: `${styles$1.ascentDetails} ${styles$1.rightAligned}`, children: /* @__PURE__ */ jsx("div", { className: styles$1.ascentDescription, children: "Ember Conduit: A medium to speak into flame. This ancient technology allows direct communication with the primordial forces of the Nullblock universe. Through an Ember Conduit, users can channel energy, access forbidden knowledge, and potentially reshape reality itself. Warning: Unauthorized use may result in spontaneous combustion or worse." }) })
      ] })
    ] });
  };
  const renderLockedScreen = () => /* @__PURE__ */ jsxs("div", { className: styles$1.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles$1.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles$1.hudTitle, children: "ACCESS RESTRICTED" }),
      /* @__PURE__ */ jsx("div", { className: styles$1.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles$1.lockedContent, children: [
      /* @__PURE__ */ jsx("p", { children: "This feature is currently locked." }),
      /* @__PURE__ */ jsx("p", { children: "Return to camp and await further instructions." })
    ] })
  ] });
  const renderEchoTab = () => {
    var _a2, _b, _c;
    return /* @__PURE__ */ jsx("div", { className: styles$1.echoContent, children: emberLinkStatus.connected ? /* @__PURE__ */ jsxs(Fragment, { children: [
      /* @__PURE__ */ jsxs("div", { className: styles$1.echoStatus, children: [
        /* @__PURE__ */ jsxs("div", { className: styles$1.statusContainer, children: [
          /* @__PURE__ */ jsx("span", { className: styles$1.statusLabel, children: "Ember Link Status:" }),
          /* @__PURE__ */ jsx("span", { className: styles$1.active, children: "Connected" })
        ] }),
        /* @__PURE__ */ jsxs("div", { className: styles$1.browserInfo, children: [
          /* @__PURE__ */ jsx("span", { className: styles$1.browserLabel, children: "Browser:" }),
          /* @__PURE__ */ jsxs("span", { className: styles$1.browserValue, children: [
            (_a2 = emberLinkStatus.browserInfo) == null ? void 0 : _a2.name,
            " ",
            (_b = emberLinkStatus.browserInfo) == null ? void 0 : _b.version,
            " (",
            (_c = emberLinkStatus.browserInfo) == null ? void 0 : _c.platform,
            ")"
          ] })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.echoMessage, children: [
        /* @__PURE__ */ jsx("p", { children: "E.C.H.O system is active and operational." }),
        /* @__PURE__ */ jsx("p", { children: "Welcome to the interface, agent." })
      ] })
    ] }) : /* @__PURE__ */ jsxs("div", { className: styles$1.disconnectedContent, children: [
      /* @__PURE__ */ jsx("div", { className: styles$1.echoStatus, children: /* @__PURE__ */ jsxs("div", { className: styles$1.statusContainer, children: [
        /* @__PURE__ */ jsx("span", { className: styles$1.statusLabel, children: "Ember Link Status:" }),
        /* @__PURE__ */ jsx("span", { className: styles$1.inactive, children: "Disconnected" })
      ] }) }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.extensionPrompt, children: [
        /* @__PURE__ */ jsx("h4", { children: "Browser Extension Required" }),
        /* @__PURE__ */ jsx("p", { children: "To establish a secure connection, you need to install the Aether browser extension." }),
        /* @__PURE__ */ jsx("p", { children: "Choose your browser to download the extension:" }),
        /* @__PURE__ */ jsxs("div", { className: styles$1.extensionLinks, children: [
          /* @__PURE__ */ jsx(
            "a",
            {
              href: "https://chrome.google.com/webstore/detail/aether",
              target: "_blank",
              rel: "noopener noreferrer",
              className: styles$1.extensionButton,
              children: "Chrome Extension"
            }
          ),
          /* @__PURE__ */ jsx(
            "a",
            {
              href: "https://addons.mozilla.org/en-US/firefox/addon/aether",
              target: "_blank",
              rel: "noopener noreferrer",
              className: styles$1.extensionButton,
              children: "Firefox Extension"
            }
          )
        ] })
      ] })
    ] }) });
  };
  const renderCampScreen = () => {
    var _a2;
    return /* @__PURE__ */ jsxs("div", { className: styles$1.hudScreen, children: [
      /* @__PURE__ */ jsxs("div", { className: styles$1.headerContainer, children: [
        /* @__PURE__ */ jsx("h2", { className: styles$1.hudTitle, children: "CAMP" }),
        /* @__PURE__ */ jsx("div", { className: styles$1.headerDivider }),
        /* @__PURE__ */ jsx("h2", { className: styles$1.architectTitle, children: "ARCHITECT VIEW" }),
        /* @__PURE__ */ jsxs("div", { className: styles$1.leaderboardContainer, children: [
          /* @__PURE__ */ jsx("div", { className: styles$1.leaderboardTitle, children: /* @__PURE__ */ jsx("button", { className: styles$1.leaderboardButton, onClick: handleLeaderboardClick, children: "ASCENDANTS" }) }),
          /* @__PURE__ */ jsx("div", { className: styles$1.leaderboardList, children: /* @__PURE__ */ jsxs("div", { className: styles$1.leaderboardItems, children: [
            leaderboardData.map((entry) => /* @__PURE__ */ jsxs(
              "div",
              {
                className: styles$1.leaderboardItem,
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
                  /* @__PURE__ */ jsxs("span", { className: styles$1.rank, children: [
                    "#",
                    entry.rank
                  ] }),
                  /* @__PURE__ */ jsx("span", { className: styles$1.camperId, children: entry.id }),
                  /* @__PURE__ */ jsx("span", { className: styles$1.matrixLevel, children: entry.matrix.level })
                ]
              },
              entry.id
            )),
            leaderboardData.map((entry) => /* @__PURE__ */ jsxs(
              "div",
              {
                className: styles$1.leaderboardItem,
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
                  /* @__PURE__ */ jsxs("span", { className: styles$1.rank, children: [
                    "#",
                    entry.rank
                  ] }),
                  /* @__PURE__ */ jsx("span", { className: styles$1.camperId, children: entry.id }),
                  /* @__PURE__ */ jsx("span", { className: styles$1.matrixLevel, children: entry.matrix.level })
                ]
              },
              `${entry.id}-duplicate`
            ))
          ] }) })
        ] })
      ] }),
      /* @__PURE__ */ jsx("div", { className: styles$1.campContent, children: /* @__PURE__ */ jsxs("div", { className: styles$1.campGrid, children: [
        /* @__PURE__ */ jsx("div", { className: styles$1.campAnalysis, children: /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsContainer, children: [
          /* @__PURE__ */ jsx("h2", { className: styles$1.containerTitle, children: "SUBNETS" }),
          /* @__PURE__ */ jsx("div", { className: styles$1.diagnosticsHeader, children: /* @__PURE__ */ jsx("h3", { children: "SUBNETS" }) }),
          /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsContent, children: [
            /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsList, children: [
              /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsItem, onClick: () => setSelectedUplink({
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
                /* @__PURE__ */ jsx("span", { className: styles$1.itemLabel, children: "🆔 ID" }),
                /* @__PURE__ */ jsxs("span", { className: styles$1.itemValue, children: [
                  "ECHO-",
                  userProfile2.id || "0000"
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsItem, onClick: () => setSelectedUplink({
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
                /* @__PURE__ */ jsxs("span", { className: styles$1.itemLabel, children: [
                  /* @__PURE__ */ jsx("span", { className: styles$1.ascentLine }),
                  " ASCENT"
                ] }),
                /* @__PURE__ */ jsx("span", { className: styles$1.itemValue, children: "Net Dweller: 1" })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsItem, onClick: () => {
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
                /* @__PURE__ */ jsx("span", { className: styles$1.itemLabel, children: "₦ NETHER" }),
                /* @__PURE__ */ jsxs("span", { className: styles$1.itemValue, children: [
                  "₦ ",
                  ((_a2 = userProfile2.nether) == null ? void 0 : _a2.toFixed(2)) || "N/A"
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsItem, onClick: () => setSelectedUplink({
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
                /* @__PURE__ */ jsx("span", { className: styles$1.itemLabel, children: "💰 CACHE VALUE" }),
                /* @__PURE__ */ jsx("span", { className: styles$1.itemValue, children: "₦ N/A" })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsItem, onClick: () => setSelectedUplink({
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
                /* @__PURE__ */ jsx("span", { className: styles$1.itemLabel, children: "🧠 MEMORIES" }),
                /* @__PURE__ */ jsx("span", { className: styles$1.itemValue, children: userProfile2.memories })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.diagnosticsItem, onClick: () => setSelectedUplink({
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
                /* @__PURE__ */ jsx("span", { className: styles$1.itemLabel, children: "🔥 EMBER CONDUIT" }),
                /* @__PURE__ */ jsx("span", { className: styles$1.itemValue, children: userProfile2.matrix.status })
              ] })
            ] }),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: styles$1.addLinkButton,
                onClick: () => alert("No Ember Conduit loaded"),
                children: "🔗 ADD NET"
              }
            )
          ] })
        ] }) }),
        /* @__PURE__ */ jsx("div", { className: styles$1.divider }),
        /* @__PURE__ */ jsx("div", { className: styles$1.campStatus, children: /* @__PURE__ */ jsxs("div", { className: styles$1.statusCard, children: [
          /* @__PURE__ */ jsxs("div", { className: styles$1.statusTabs, children: [
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles$1.statusTab} ${activeTab2 === "echo" ? styles$1.activeTab : ""}`,
                onClick: () => setActiveTab("echo"),
                children: "E.C.H.O"
              }
            ),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles$1.statusTab} ${activeTab2 === "systems" ? styles$1.activeTab : ""}`,
                onClick: () => setActiveTab("systems"),
                children: "NYX"
              }
            ),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles$1.statusTab} ${activeTab2 === "defense" ? styles$1.activeTab : ""}`,
                onClick: () => setActiveTab("defense"),
                children: "LEGION"
              }
            ),
            /* @__PURE__ */ jsx(
              "button",
              {
                className: `${styles$1.statusTab} ${activeTab2 === "missions" ? styles$1.activeTab : ""}`,
                onClick: () => setActiveTab("missions"),
                children: "MISSIONS"
              }
            )
          ] }),
          /* @__PURE__ */ jsxs("div", { className: styles$1.tabContent, children: [
            activeTab2 === "echo" && renderEchoTab(),
            activeTab2 === "systems" && /* @__PURE__ */ jsx("div", { className: styles$1.systemsTab, children: /* @__PURE__ */ jsxs("div", { className: styles$1.lockedContent, children: [
              /* @__PURE__ */ jsx("p", { children: "This feature is currently locked." }),
              /* @__PURE__ */ jsx("p", { children: "Return to camp and await further instructions." })
            ] }) }),
            activeTab2 === "defense" && /* @__PURE__ */ jsx("div", { className: styles$1.defenseTab, children: /* @__PURE__ */ jsxs("div", { className: styles$1.lockedContent, children: [
              /* @__PURE__ */ jsx("p", { children: "This feature is currently locked." }),
              /* @__PURE__ */ jsx("p", { children: "Return to camp and await further instructions." })
            ] }) }),
            activeTab2 === "missions" && /* @__PURE__ */ jsxs("div", { className: styles$1.missionsTab, children: [
              /* @__PURE__ */ jsx("div", { className: styles$1.missionHeader, children: /* @__PURE__ */ jsxs("div", { className: styles$1.active, children: [
                /* @__PURE__ */ jsx("span", { className: styles$1.missionLabel, children: "ACTIVE:" }),
                /* @__PURE__ */ jsx("span", { className: styles$1.missionTitle, children: (activeMission == null ? void 0 : activeMission.title) || "Share on X" })
              ] }) }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.missionContent, children: [
                /* @__PURE__ */ jsxs("div", { className: styles$1.availableMissions, children: [
                  /* @__PURE__ */ jsx("h4", { children: "AVAILABLE MISSIONS" }),
                  /* @__PURE__ */ jsxs("div", { className: styles$1.missionList, children: [
                    /* @__PURE__ */ jsxs("div", { className: `${styles$1.missionItem} ${styles$1.active}`, children: [
                      /* @__PURE__ */ jsxs("div", { className: styles$1.missionItemContent, children: [
                        /* @__PURE__ */ jsx("span", { className: styles$1.missionTitle, children: "Share on X" }),
                        /* @__PURE__ */ jsx("span", { className: styles$1.missionStatus, children: "ACTIVE" })
                      ] }),
                      /* @__PURE__ */ jsx("span", { className: styles$1.missionReward, children: "TBD NETHER AIRDROP" })
                    ] }),
                    /* @__PURE__ */ jsxs("div", { className: `${styles$1.missionItem} ${styles$1.blurred}`, children: [
                      /* @__PURE__ */ jsxs("div", { className: styles$1.missionItemContent, children: [
                        /* @__PURE__ */ jsx("span", { className: styles$1.missionTitle, children: "Mission 2" }),
                        /* @__PURE__ */ jsx("span", { className: styles$1.missionStatus, children: "LOCKED" })
                      ] }),
                      /* @__PURE__ */ jsx("span", { className: `${styles$1.missionReward} ${styles$1.blurred}`, children: "??? NETHER" })
                    ] }),
                    /* @__PURE__ */ jsxs("div", { className: `${styles$1.missionItem} ${styles$1.blurred}`, children: [
                      /* @__PURE__ */ jsxs("div", { className: styles$1.missionItemContent, children: [
                        /* @__PURE__ */ jsx("span", { className: styles$1.missionTitle, children: "Mission 3" }),
                        /* @__PURE__ */ jsx("span", { className: styles$1.missionStatus, children: "LOCKED" })
                      ] }),
                      /* @__PURE__ */ jsx("span", { className: `${styles$1.missionReward} ${styles$1.blurred}`, children: "??? NETHER" })
                    ] })
                  ] })
                ] }),
                /* @__PURE__ */ jsxs("div", { className: styles$1.missionDescription, children: [
                  /* @__PURE__ */ jsx("h4", { children: "MISSION BRIEF" }),
                  /* @__PURE__ */ jsx("p", { className: styles$1.missionText, children: `"Welcome, Camper, to your first trial. Tend the flame carefully. Share your Base Camp on X—let its glow haunt the realm. More souls drawn, more NETHER gained. Don't let it fade."` }),
                  /* @__PURE__ */ jsxs("div", { className: styles$1.missionInstructions, children: [
                    /* @__PURE__ */ jsx("h4", { children: "QUALIFICATION REQUIREMENTS" }),
                    /* @__PURE__ */ jsxs("ul", { children: [
                      /* @__PURE__ */ jsxs("li", { children: [
                        "Follow",
                        /* @__PURE__ */ jsx("span", { className: styles$1.highlight, children: "@Nullblock_io" })
                      ] }),
                      /* @__PURE__ */ jsxs("li", { children: [
                        "Tweet out the cashtag ",
                        /* @__PURE__ */ jsx("span", { className: styles$1.highlight, children: "$NETHER" })
                      ] }),
                      /* @__PURE__ */ jsxs("li", { children: [
                        "Include the official CA: ",
                        /* @__PURE__ */ jsx("span", { className: styles$1.highlight, children: "TBD" })
                      ] })
                    ] }),
                    /* @__PURE__ */ jsx("p", { className: styles$1.missionNote, children: "Airdrop amount will be determined by post engagement and creativity." })
                  ] }),
                  /* @__PURE__ */ jsxs("div", { className: styles$1.missionReward, children: [
                    /* @__PURE__ */ jsx("span", { className: styles$1.rewardLabel, children: "REWARD:" }),
                    /* @__PURE__ */ jsx("span", { className: styles$1.rewardValue, children: "TBD NETHER AIRDROP" })
                  ] }),
                  /* @__PURE__ */ jsxs("div", { className: styles$1.missionExpiration, children: [
                    /* @__PURE__ */ jsx("span", { className: styles$1.expirationLabel, children: "EXPIRES:" }),
                    /* @__PURE__ */ jsx("span", { className: styles$1.expirationValue, children: "TBD" })
                  ] })
                ] })
              ] })
            ] })
          ] })
        ] }) })
      ] }) }),
      selectedUplink && /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx("div", { className: styles$1.modalOverlay, onClick: handleCloseModal }),
        /* @__PURE__ */ jsxs("div", { className: styles$1.uplinkModal, children: [
          /* @__PURE__ */ jsxs("div", { className: styles$1.modalHeader, children: [
            /* @__PURE__ */ jsx("h3", { children: selectedUplink.name }),
            /* @__PURE__ */ jsx("button", { className: styles$1.closeButton, onClick: handleCloseModal, children: "×" })
          ] }),
          /* @__PURE__ */ jsxs("div", { className: styles$1.modalContent, children: [
            /* @__PURE__ */ jsxs("div", { className: styles$1.statusSection, children: [
              /* @__PURE__ */ jsx("h4", { children: "Status Overview" }),
              /* @__PURE__ */ jsx("div", { className: styles$1.statusGrid, children: selectedUplink.details.stats.map((stat, index) => /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: stat.label }),
                /* @__PURE__ */ jsx("div", { className: `${styles$1.value} ${stat.status || ""}`, children: stat.value })
              ] }, index)) })
            ] }),
            /* @__PURE__ */ jsxs("div", { className: styles$1.detailsSection, children: [
              /* @__PURE__ */ jsx("h4", { children: "Details" }),
              /* @__PURE__ */ jsx("p", { children: selectedUplink.details.description })
            ] })
          ] })
        ] })
      ] }),
      showLeaderboard && /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx("div", { className: styles$1.modalOverlay, onClick: handleCloseLeaderboard }),
        /* @__PURE__ */ jsxs("div", { className: styles$1.leaderboardModal, children: [
          /* @__PURE__ */ jsxs("div", { className: styles$1.modalHeader, children: [
            /* @__PURE__ */ jsx("h3", { children: "TOP CAMPERS" }),
            /* @__PURE__ */ jsx("button", { className: styles$1.closeButton, onClick: handleCloseLeaderboard, children: "×" })
          ] }),
          /* @__PURE__ */ jsx("div", { className: styles$1.legendWarning, children: /* @__PURE__ */ jsx("p", { children: "⚠️ These are the legends of the last Flame. The current Conduit has yet to awaken." }) }),
          /* @__PURE__ */ jsx("div", { className: styles$1.leaderboardGrid, children: leaderboardData.map((entry) => /* @__PURE__ */ jsxs("div", { className: styles$1.leaderboardCard, children: [
            /* @__PURE__ */ jsxs("div", { className: styles$1.cardHeader, children: [
              /* @__PURE__ */ jsxs("span", { className: styles$1.rank, children: [
                "#",
                entry.rank
              ] }),
              /* @__PURE__ */ jsx("span", { className: styles$1.camperId, children: entry.id })
            ] }),
            /* @__PURE__ */ jsxs("div", { className: styles$1.statusGrid, children: [
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "ASCENT" }),
                /* @__PURE__ */ jsx("div", { className: styles$1.value, children: entry.ascent })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "NETHER" }),
                /* @__PURE__ */ jsxs("div", { className: styles$1.value, children: [
                  "₦ ",
                  entry.nether
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "CACHE VALUE" }),
                /* @__PURE__ */ jsxs("div", { className: styles$1.value, children: [
                  "₦ ",
                  entry.cacheValue
                ] })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "MEMORIES" }),
                /* @__PURE__ */ jsx("div", { className: styles$1.value, children: entry.memories })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "MATRIX LEVEL" }),
                /* @__PURE__ */ jsx("div", { className: styles$1.value, children: entry.matrix.level })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "RARITY" }),
                /* @__PURE__ */ jsx("div", { className: styles$1.value, children: entry.matrix.rarity })
              ] }),
              /* @__PURE__ */ jsxs("div", { className: styles$1.statusItem, children: [
                /* @__PURE__ */ jsx("div", { className: styles$1.label, children: "STATUS" }),
                /* @__PURE__ */ jsx("div", { className: styles$1.value, children: entry.matrix.status })
              ] })
            ] })
          ] }, entry.id)) })
        ] })
      ] })
    ] });
  };
  const renderInventoryScreen = () => /* @__PURE__ */ jsxs("div", { className: styles$1.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles$1.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles$1.hudTitle, children: "CACHE" }),
      /* @__PURE__ */ jsx("div", { className: styles$1.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles$1.inventorySection, children: [
      /* @__PURE__ */ jsx("h3", { children: "WEAPONS" }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.emptyState, children: [
        /* @__PURE__ */ jsx("p", { children: "No weapons found." }),
        /* @__PURE__ */ jsx("p", { children: "Complete missions to acquire gear." })
      ] })
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles$1.inventorySection, children: [
      /* @__PURE__ */ jsx("h3", { children: "SUPPLIES" }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.emptyState, children: [
        /* @__PURE__ */ jsx("p", { children: "Cache empty." }),
        /* @__PURE__ */ jsx("p", { children: "Gather resources to expand inventory." })
      ] })
    ] })
  ] });
  const renderCampaignScreen = () => /* @__PURE__ */ jsxs("div", { className: styles$1.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles$1.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles$1.hudTitle, children: "CAMPAIGN" }),
      /* @__PURE__ */ jsx("div", { className: styles$1.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles$1.realityContent, children: [
      /* @__PURE__ */ jsxs("div", { className: styles$1.realityStatus, children: [
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
      /* @__PURE__ */ jsxs("div", { className: styles$1.missions, children: [
        /* @__PURE__ */ jsx("h3", { children: "OBJECTIVES" }),
        /* @__PURE__ */ jsx("p", { className: styles$1.placeholder, children: "No active missions" }),
        /* @__PURE__ */ jsx("p", { className: styles$1.placeholder, children: "Complete training to begin" })
      ] })
    ] })
  ] });
  const renderLabScreen = () => /* @__PURE__ */ jsxs("div", { className: styles$1.hudScreen, children: [
    /* @__PURE__ */ jsxs("div", { className: styles$1.headerContainer, children: [
      /* @__PURE__ */ jsx("h2", { className: styles$1.hudTitle, children: "LAB" }),
      /* @__PURE__ */ jsx("div", { className: styles$1.headerDivider }),
      renderUserProfile()
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles$1.interfaceContent, children: [
      /* @__PURE__ */ jsxs("div", { className: styles$1.interfaceSection, children: [
        /* @__PURE__ */ jsx("h3", { children: "SYSTEMS" }),
        /* @__PURE__ */ jsxs("p", { children: [
          "Phantom: ",
          /* @__PURE__ */ jsx("span", { className: styles$1.connected, children: "CONNECTED" })
        ] }),
        /* @__PURE__ */ jsxs("p", { children: [
          "Core: ",
          /* @__PURE__ */ jsx("span", { className: styles$1.initializing, children: "INITIALIZING" })
        ] })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: styles$1.interfaceSection, children: [
        /* @__PURE__ */ jsx("h3", { children: "CONFIGURATIONS" }),
        /* @__PURE__ */ jsx("p", { className: styles$1.placeholder, children: "No active modifications" }),
        /* @__PURE__ */ jsx("p", { className: styles$1.placeholder, children: "Run diagnostics to begin" })
      ] })
    ] })
  ] });
  const renderScreen = () => {
    if (!unlockedScreens.includes(screen)) {
      return renderLockedScreen();
    }
    switch (screen) {
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
  return /* @__PURE__ */ jsxs("div", { className: `${styles$1.echoContainer} ${styles$1[theme]}`, children: [
    renderControlScreen(),
    /* @__PURE__ */ jsx("div", { className: styles$1.hudWindow, children: renderScreen() })
  ] });
};
const digitizingText = "_digitizingText_13yxa_2";
const glitch = "_glitch_13yxa_1";
const complete = "_complete_13yxa_21";
const cursor = "_cursor_13yxa_25";
const blink = "_blink_13yxa_1";
const blue = "_blue_13yxa_33";
const light = "_light_13yxa_41";
const styles = {
  digitizingText,
  glitch,
  complete,
  cursor,
  blink,
  blue,
  light,
  "null-dark": "_null-dark_13yxa_49"
};
const DigitizingText = ({
  text,
  duration = 1e4,
  // Default 10 seconds
  onComplete,
  theme = "light"
  // Default to light theme
}) => {
  const [displayText, setDisplayText] = useState("");
  const [isVisible, setIsVisible] = useState(true);
  const [isComplete, setIsComplete] = useState(false);
  const timeoutRef = useRef(null);
  const textRef = useRef(text);
  const isAnimatingRef = useRef(false);
  useEffect(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    textRef.current = text;
    setDisplayText("");
    setIsVisible(true);
    setIsComplete(false);
    isAnimatingRef.current = false;
    animateText();
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [text]);
  const animateText = () => {
    if (isAnimatingRef.current) return;
    isAnimatingRef.current = true;
    let currentIndex = 0;
    const speed = 30;
    const addNextChar = () => {
      if (currentIndex <= text.length) {
        setDisplayText(text.substring(0, currentIndex));
        currentIndex++;
        timeoutRef.current = setTimeout(addNextChar, speed);
      } else {
        setIsComplete(true);
        isAnimatingRef.current = false;
        if (onComplete) {
          onComplete();
        }
        if (duration > 0) {
          timeoutRef.current = setTimeout(() => {
            setIsVisible(false);
          }, duration);
        }
      }
    };
    addNextChar();
  };
  if (!isVisible) return null;
  const getDigitizingTheme = () => {
    switch (theme) {
      case "null-dark":
        return "null-dark";
      case "light":
        return "light";
      case "blue":
        return "blue";
      default:
        return theme;
    }
  };
  return /* @__PURE__ */ jsxs("div", { className: `${styles.digitizingText} ${styles[getDigitizingTheme()]} ${isComplete ? styles.complete : ""}`, children: [
    displayText,
    !isComplete && /* @__PURE__ */ jsx("span", { className: styles.cursor, children: "_" })
  ] });
};
const Home = () => {
  const [walletConnected, setWalletConnected] = useState(false);
  const [publicKey, setPublicKey] = useState(null);
  const [showEcho, setShowEcho] = useState(false);
  const [messages, setMessages] = useState([]);
  const [messageIndex, setMessageIndex] = useState(0);
  const [hasPhantom, setHasPhantom] = useState(false);
  const [currentRoom, setCurrentRoom] = useState("/logs");
  useState(false);
  const [showWelcomeText, setShowWelcomeText] = useState(true);
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
      setShowEcho(true);
      setShowWelcomeText(false);
    } else {
      setShowWelcomeText(true);
      setWalletConnected(false);
      setShowEcho(false);
    }
    if (savedTheme) {
      setCurrentTheme(savedTheme);
    }
  }, []);
  useEffect(() => {
    if (showEcho) {
      setShowWelcomeText(false);
    }
  }, [showEcho]);
  useEffect(() => {
    console.log("Text complete effect triggered:", { textComplete, walletConnected, showEcho });
    if (textComplete && !walletConnected && !showEcho) {
      console.log("Setting showConnectButton to true");
      setShowConnectButton(true);
    }
  }, [textComplete, walletConnected, showEcho]);
  useEffect(() => {
    if (!walletConnected && !showEcho && showWelcomeText) {
      const fallbackTimer = setTimeout(() => {
        console.log("Fallback timer triggered - showing connect button");
        setShowConnectButton(true);
      }, 5e3);
      return () => clearTimeout(fallbackTimer);
    }
  }, [walletConnected, showEcho, showWelcomeText]);
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
      setShowWelcomeText(false);
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
            setShowEcho(true);
            setShowWelcomeText(false);
            return;
          } catch (error) {
            console.log("Auto-reconnect failed:", error);
          }
        }
        localStorage.removeItem("walletPublickey");
        localStorage.removeItem("lastAuthTime");
        localStorage.removeItem("hasSeenEcho");
        setWalletConnected(false);
        setPublicKey(null);
        setShowEcho(false);
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
          setShowEcho(true);
          localStorage.setItem("walletPublickey", walletPubKey);
          localStorage.setItem("hasSeenEcho", "true");
          updateAuthTime();
          setShowWelcomeText(false);
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
          setShowEcho(false);
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
          setShowEcho(false);
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
          setShowWelcomeText(true);
          setTextComplete(false);
          setShowConnectButton(false);
        } catch (error) {
          console.error("Error disconnecting from Phantom:", error);
        }
      }
    }
  };
  const handleTextComplete = () => {
    console.log("Text complete callback triggered");
    setTextComplete(true);
    if (!walletConnected && !showEcho) {
      console.log("Setting showConnectButton to true directly from callback");
      setShowConnectButton(true);
    }
  };
  return /* @__PURE__ */ jsxs("div", { className: `${styles$3.appContainer} ${styles$3[`theme-${currentTheme}`]}`, children: [
    /* @__PURE__ */ jsx("div", { className: styles$3.backgroundImage }),
    /* @__PURE__ */ jsx(StarsCanvas, { theme: currentTheme }),
    /* @__PURE__ */ jsxs("div", { className: `${styles$3.scene} ${showEcho ? styles$3.echoActive : ""}`, children: [
      /* @__PURE__ */ jsx("div", { className: styles$3.fire }),
      /* @__PURE__ */ jsx("div", { className: styles$3.nyx })
    ] }),
    showWelcomeText && !showEcho && /* @__PURE__ */ jsx(
      DigitizingText,
      {
        text: "Welcome to Nullblock.",
        duration: 0,
        theme: currentTheme === "null" ? "null-dark" : "light",
        onComplete: handleTextComplete
      }
    ),
    showConnectButton && !walletConnected && !showEcho && /* @__PURE__ */ jsx("div", { className: styles$3.connectButtonContainer, children: /* @__PURE__ */ jsx(
      "button",
      {
        className: `${styles$3.connectButton} ${styles$3[`theme-${currentTheme}`]}`,
        onClick: manualConnect,
        children: "Connect"
      }
    ) }),
    showEcho && /* @__PURE__ */ jsx(
      Echo,
      {
        publicKey,
        onDisconnect: handleDisconnect,
        theme: currentTheme,
        onClose: () => {
          setShowEcho(false);
          setShowWelcomeText(true);
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
