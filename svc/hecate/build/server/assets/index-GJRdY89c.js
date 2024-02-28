import { jsxs, Fragment, jsx } from "react/jsx-runtime";
import { Meta } from "@lomray/react-head-manager";
import { IS_SSR_MODE } from "@lomray/vite-ssr-boost/constants/common.js";
import cn from "classnames";
import Cookies from "js-cookie";
import { useState } from "react";
import { useLoaderData, Link } from "react-router-dom";
import { A as APP_VERSION, m as manager } from "../server.js";
import "@lomray/consistent-suspense/server/index.js";
import "@lomray/react-head-manager/server/index.js";
import "@lomray/react-mobx-manager";
import "@lomray/react-mobx-manager/manager-stream.js";
import "@lomray/vite-ssr-boost/node/entry.js";
import "cookie-parser";
import "isbot";
import "mobx-react-lite";
import "@lomray/vite-ssr-boost/helpers/import-route.js";
import "@lomray/vite-ssr-boost/components/scroll-to-top.js";
import "@lomray/vite-ssr-boost/components/response-status.js";
import "@lomray/react-route-manager";
import "@lomray/consistent-suspense";
const ReactLogoImg = "/assets/react-h3aPdYU7.svg";
const logos = "_logos_5whnu_1";
const logo = "_logo_5whnu_1";
const logoBoost = "_logoBoost_5whnu_17";
const card = "_card_5whnu_22";
const navigateExplain = "_navigateExplain_5whnu_26";
const styles = {
  logos,
  logo,
  logoBoost,
  card,
  navigateExplain
};
const Home = () => {
  const { isDefaultCrawler } = useLoaderData();
  const [isCrawler, setIsCrawler] = useState(isDefaultCrawler);
  const hasVersion = !APP_VERSION.startsWith("APP_");
  const toggleCrawler = () => {
    const nextVal = isCrawler ? "0" : "1";
    setIsCrawler(nextVal === "1");
    Cookies.set("isCrawler", nextVal);
  };
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsxs(Meta, { children: [
      /* @__PURE__ */ jsx("title", { children: "Home page" }),
      /* @__PURE__ */ jsx("meta", { name: "description", content: "Home page" })
    ] }),
    /* @__PURE__ */ jsx("div", { children: "SPA, SSR, Mobx, Consistent Suspense, Meta tags" }),
    /* @__PURE__ */ jsxs("div", { children: [
      hasVersion && /* @__PURE__ */ jsxs("p", { children: [
        "Version: ",
        /* @__PURE__ */ jsx("strong", { children: APP_VERSION })
      ] }),
      /* @__PURE__ */ jsxs("p", { children: [
        "Type: ",
        IS_SSR_MODE ? "SSR" : "SPA"
      ] })
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.logos, children: [
      /* @__PURE__ */ jsx("a", { href: "https://vitejs.dev/", target: "_blank", rel: "nofollow", children: /* @__PURE__ */ jsx("img", { src: "/vite.svg", className: styles.logo, alt: "Vite logo" }) }),
      /* @__PURE__ */ jsx("a", { href: "https://react.dev", target: "_blank", rel: "nofollow", children: /* @__PURE__ */ jsx("img", { src: ReactLogoImg, className: styles.logo, alt: "React logo" }) }),
      /* @__PURE__ */ jsx("a", { href: "https://github.com/Lomray-Software/vite-ssr-boost", target: "_blank", children: /* @__PURE__ */ jsx(
        "img",
        {
          src: "https://raw.githubusercontent.com/Lomray-Software/vite-ssr-boost/prod/logo.png",
          className: cn(styles.logo, styles.logoBoost),
          alt: "SSR Boost logo"
        }
      ) }),
      /* @__PURE__ */ jsx("a", { href: "https://github.com/Lomray-Software/react-mobx-manager", target: "_blank", children: /* @__PURE__ */ jsx(
        "img",
        {
          src: "https://raw.githubusercontent.com/Lomray-Software/react-mobx-manager/prod/logo.png",
          className: cn(styles.logo, styles.logoBoost),
          alt: "Mobx Store Manager logo"
        }
      ) })
    ] }),
    /* @__PURE__ */ jsxs("div", { className: styles.card, children: [
      /* @__PURE__ */ jsxs("button", { type: "button", onClick: toggleCrawler, children: [
        "You are watching site like: ",
        /* @__PURE__ */ jsx("strong", { children: isCrawler ? "Search bot" : "Human" })
      ] }),
      /* @__PURE__ */ jsx("p", { children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("details"), children: "How to works Suspense?" }) }),
      /* @__PURE__ */ jsx("p", { children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("errorBoundary"), children: "Investigate error boundary." }) }),
      /* @__PURE__ */ jsx("p", { children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("nestedSuspense"), children: "What about nested Suspense?" }) }),
      /* @__PURE__ */ jsx("p", { children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("redirect"), children: "Redirect demo" }) }),
      /* @__PURE__ */ jsx("p", { children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("notLazy"), children: "Not lazy page demo" }) })
    ] }),
    /* @__PURE__ */ jsx("p", { className: styles.navigateExplain, children: "Click on the links to learn more" }),
    /* @__PURE__ */ jsx("p", { className: styles.navigateExplain, children: /* @__PURE__ */ jsx("a", { href: "https://github.com/Lomray-Software/vite-template", target: "_blank", rel: "nofollow", children: "Open repository" }) })
  ] });
};
Home.loader = ({ request }) => {
  var _a;
  const isDefaultCrawler = ((_a = request.headers.get("cookie")) == null ? void 0 : _a.includes("isCrawler=1")) ?? Cookies.get("isCrawler") === "1";
  return {
    isDefaultCrawler
  };
};
export {
  Home as default
};
