var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => {
  __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
  return value;
};
import { jsxs, Fragment, jsx } from "react/jsx-runtime";
import { Meta } from "@lomray/react-head-manager";
import { withStores } from "@lomray/react-mobx-manager";
import { useParams, Link } from "react-router-dom";
import { E as ErrorBoundary, D as DefaultSuspense } from "./index-RhZ6fv6P.js";
import { a as API_GATEWAY, m as manager } from "../server.js";
import SuspenseQuery from "@lomray/react-mobx-manager/suspense-query.js";
import axios from "axios";
import { makeObservable, observable, action } from "mobx";
import "@lomray/consistent-suspense";
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
     * Suspense service
     */
    __publicField(this, "suspense");
    /**
     * Get user
     */
    __publicField(this, "getUser", async (id) => {
      const { data } = await axios.request({ url: `${API_GATEWAY}/?seed=${id}` });
      await new Promise((resolve) => {
        setTimeout(resolve, 2e3);
      });
      const [{ name, email, picture }] = data.results;
      this.setUser({
        id,
        name: Object.values(name).join(" "),
        email,
        avatar: picture.medium
      });
    });
    this.suspense = new SuspenseQuery(this);
    makeObservable(this, {
      user: observable,
      setUser: action.bound
    });
  }
  /**
   * Set users
   */
  setUser(user) {
    this.user = user;
  }
}
__publicField(MainStore, "id", "Sa");
const stores = {
  mainStore: MainStore
};
const avatar = "_avatar_1vdaq_1";
const styles = {
  avatar
};
const User = ({ mainStore: { user, suspense, getUser } }) => {
  const { id } = useParams();
  suspense.query(() => getUser(id));
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsxs(Meta, { children: [
      /* @__PURE__ */ jsxs("title", { children: [
        "User ",
        user == null ? void 0 : user.name
      ] }),
      /* @__PURE__ */ jsx("meta", { name: "description", content: `User description ${user.name}` })
    ] }),
    /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsxs("p", { children: [
        "Id: ",
        user == null ? void 0 : user.id
      ] }),
      /* @__PURE__ */ jsxs("p", { children: [
        "Name: ",
        user == null ? void 0 : user.name
      ] }),
      /* @__PURE__ */ jsxs("p", { children: [
        "Email: ",
        user == null ? void 0 : user.email
      ] }),
      /* @__PURE__ */ jsxs("p", { children: [
        "Avatar: ",
        /* @__PURE__ */ jsx("img", { src: user == null ? void 0 : user.avatar, className: styles.avatar, alt: "User avatar" })
      ] })
    ] }),
    /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("details"), children: "Go back" }) })
  ] });
};
User.ErrorBoundary = ErrorBoundary;
User.Suspense = DefaultSuspense;
const UserWrapper = withStores(User, stores);
export {
  UserWrapper as default
};
