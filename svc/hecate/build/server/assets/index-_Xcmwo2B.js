var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => {
  __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
  return value;
};
import { jsx } from "react/jsx-runtime";
import { withStores } from "@lomray/react-mobx-manager";
import { E as ErrorBoundary, D as DefaultSuspense } from "./index-RhZ6fv6P.js";
import SuspenseQuery from "@lomray/react-mobx-manager/suspense-query.js";
import "@lomray/consistent-suspense";
import "../server.js";
import "@lomray/consistent-suspense/server/index.js";
import "@lomray/react-head-manager";
import "@lomray/react-head-manager/server/index.js";
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
import "react";
class MainStore {
  /**
   * @constructor
   */
  constructor() {
    /**
     * Suspense service
     */
    __publicField(this, "suspense");
    /**
     * Get user
     */
    __publicField(this, "getUser", async () => {
      await new Promise((resolve) => {
        setTimeout(resolve, 2e3);
      });
      throw new Error("Ooops. I'm caught error? :)");
    });
    this.suspense = new SuspenseQuery(this);
  }
}
const stores = {
  mainStore: MainStore
};
const ErrorBoundaryPage = ({ mainStore: { suspense, getUser } }) => {
  suspense.query(() => getUser());
  return /* @__PURE__ */ jsx("div", { children: "This line never be executed." });
};
ErrorBoundaryPage.ErrorBoundary = ErrorBoundary;
ErrorBoundaryPage.Suspense = DefaultSuspense;
const ErrorBoundaryPageWrapper = withStores(ErrorBoundaryPage, stores);
export {
  ErrorBoundaryPageWrapper as default
};
