var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => {
  __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
  return value;
};
import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { useId, Suspense } from "@lomray/consistent-suspense";
import { Meta } from "@lomray/react-head-manager";
import { Link, useLoaderData } from "react-router-dom";
import { A as API_GATEWAY, m as manager } from "../server.js";
import Skeleton from "react-loading-skeleton";
import { withStores } from "@lomray/react-mobx-manager";
import SuspenseQuery from "@lomray/react-mobx-manager/suspense-query.js";
import axios from "axios";
import { makeObservable, observable, action } from "mobx";
import "@lomray/consistent-suspense/server/index.js";
import "@lomray/react-head-manager/server/index.js";
import "@lomray/react-mobx-manager/manager-stream.js";
import "@lomray/vite-ssr-boost/node/entry.js";
import "cookie-parser";
import "isbot";
import "mobx-react-lite";
import "@lomray/vite-ssr-boost/helpers/import-route.js";
import "@lomray/vite-ssr-boost/components/scroll-to-top.js";
import "@lomray/vite-ssr-boost/components/response-status.js";
import "@lomray/react-route-manager";
import "react";
const Placeholder = ({ count = 1 }) => /* @__PURE__ */ jsx(Skeleton, { count, highlightColor: "#969696" });
class MainStore {
  /**
   * @constructor
   */
  constructor() {
    /**
     * User
     */
    __publicField(this, "user", null);
    /**
     * API request error
     */
    __publicField(this, "error", null);
    /**
     * Indicating request executing
     */
    __publicField(this, "isLoading", false);
    /**
     * Suspense service
     */
    __publicField(this, "suspense");
    /**
     * Get user
     */
    __publicField(this, "getUser", async (id) => {
      this.setIsLoading(true);
      this.setError(null);
      try {
        const { data } = await axios.request({ url: `${API_GATEWAY}/?seed=${id}` });
        const time = id === "user-1" ? 1e3 : id === "user-3" ? 3e3 : 2e3;
        await new Promise((resolve) => {
          setTimeout(resolve, time);
        });
        const [{ name, email, picture }] = data.results;
        this.setUser({
          id,
          name: Object.values(name).join(" "),
          email,
          avatar: picture.medium
        });
      } catch (e) {
        this.setError(e == null ? void 0 : e.message);
      }
      this.setIsLoading(false);
    });
    this.suspense = new SuspenseQuery(this);
    makeObservable(this, {
      user: observable,
      error: observable,
      isLoading: observable,
      setUser: action.bound,
      setError: action.bound,
      setIsLoading: action.bound
    });
  }
  /**
   * Set users
   */
  setUser(user) {
    this.user = user;
  }
  /**
   * Set error
   */
  setError(message) {
    this.error = message;
  }
  /**
   * Set loading state
   */
  setIsLoading(state) {
    this.isLoading = state;
  }
}
__publicField(MainStore, "id", "Sc");
const stores = {
  mainStore: MainStore
};
const col = "_col_1s2ah_1";
const styles = {
  col
};
const User = ({ userId, mainStore: { user, suspense, getUser } }) => {
  suspense.query(() => getUser(userId));
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    (user == null ? void 0 : user.id) === "user-1" && /* @__PURE__ */ jsxs(Meta, { children: [
      /* @__PURE__ */ jsxs("title", { children: [
        "User: ",
        user == null ? void 0 : user.id
      ] }),
      /* @__PURE__ */ jsx("meta", { name: "keywords", content: "user" })
    ] }),
    /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsx("span", { className: styles.col, children: "User from suspense:" }),
      " ",
      /* @__PURE__ */ jsxs(Link, { to: manager.makeURL("details.user", { id: user.id }), children: [
        user == null ? void 0 : user.id,
        " (",
        user == null ? void 0 : user.name,
        ")"
      ] })
    ] })
  ] });
};
const UserWrapper = withStores(User, stores);
const Details = () => {
  const { userIds } = useLoaderData();
  const id1 = useId();
  const id2 = useId();
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsxs(Meta, { children: [
      /* @__PURE__ */ jsx("title", { children: "Details page" }),
      /* @__PURE__ */ jsx("meta", { name: "description", content: "Details page" }),
      /* @__PURE__ */ jsx("body", { "data-id": id1, style: { border: "5px" } })
    ] }),
    /* @__PURE__ */ jsxs("p", { style: { border: "1px" }, children: [
      "This is about page. Stable ids: ",
      /* @__PURE__ */ jsx("strong", { children: id1 }),
      " and ",
      /* @__PURE__ */ jsx("strong", { children: id2 })
    ] }),
    /* @__PURE__ */ jsxs("p", { children: [
      "Load users for id's: ",
      /* @__PURE__ */ jsx("strong", { children: userIds.join(", ") })
    ] }),
    /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Placeholder, { count: 2 }), children: /* @__PURE__ */ jsxs(Fragment, { children: [
        /* @__PURE__ */ jsx(Suspense.NS, { children: /* @__PURE__ */ jsx(UserWrapper, { userId: "user-1" }) }),
        /* @__PURE__ */ jsx(Suspense.NS, { children: /* @__PURE__ */ jsx(UserWrapper, { userId: "user-3" }) })
      ] }) }),
      /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Placeholder, {}), children: /* @__PURE__ */ jsx(UserWrapper, { userId: "user-2" }) })
    ] }),
    /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("home"), children: "Go back" }) })
  ] });
};
Details.loader = () => {
  const userIds = ["user-1", "user-2", "user-3"];
  return { userIds };
};
export {
  Details as default
};
