import { jsx } from "react/jsx-runtime";
import Navigate from "@lomray/vite-ssr-boost/components/navigate.js";
import { useState, useEffect } from "react";
import { m as manager } from "../server.js";
import "@lomray/consistent-suspense/server/index.js";
import "@lomray/react-head-manager";
import "@lomray/react-head-manager/server/index.js";
import "@lomray/react-mobx-manager";
import "@lomray/react-mobx-manager/manager-stream.js";
import "@lomray/vite-ssr-boost/node/entry.js";
import "cookie-parser";
import "isbot";
import "mobx-react-lite";
import "@lomray/vite-ssr-boost/helpers/import-route.js";
import "@lomray/vite-ssr-boost/components/scroll-to-top.js";
import "react-router-dom";
import "@lomray/vite-ssr-boost/components/response-status.js";
import "@lomray/react-route-manager";
import "@lomray/consistent-suspense";
const Redirect = () => {
  const [shouldRedirect, setShouldRedirect] = useState(false);
  useEffect(() => {
    const timer = setTimeout(() => {
      setShouldRedirect(true);
    }, 2e3);
    return () => {
      clearTimeout(timer);
    };
  }, []);
  return /* @__PURE__ */ jsx(Navigate, { to: manager.makeURL("home") }) || /* @__PURE__ */ jsx("div", { children: "Wait 2 sec..." });
};
export {
  Redirect as default
};
