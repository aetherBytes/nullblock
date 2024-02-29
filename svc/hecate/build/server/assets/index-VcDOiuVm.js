var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => {
  __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
  return value;
};
import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { Suspense } from "@lomray/consistent-suspense";
import { Meta } from "@lomray/react-head-manager";
import { Link } from "react-router-dom";
import { a as API_GATEWAY, F as Fallback, m as manager } from "../server.js";
import { withStores } from "@lomray/react-mobx-manager";
import { useMemo } from "react";
import SuspenseQuery from "@lomray/react-mobx-manager/suspense-query.js";
import axios from "axios";
import { makeObservable, observable, runInAction } from "mobx";
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
    __publicField(this, "getUser", async (id, field) => {
      const { data } = await axios.request({ url: `${API_GATEWAY}/?seed=${id}` });
      await new Promise((resolve) => {
        setTimeout(resolve, id === "user-1" ? 2e3 : 1500);
      });
      const [{ name, email }] = data.results;
      runInAction(() => {
        const user = {
          id,
          name: Object.values(name).join(" "),
          email,
          avatar: ""
        };
        this.user = { [field]: user[field] };
      });
    });
    this.suspense = new SuspenseQuery(this);
    makeObservable(this, {
      user: observable
    });
  }
}
__publicField(MainStore, "id", "Sb");
const stores = {
  mainStore: MainStore
};
const User = ({ id, fields, mainStore: { user, suspense, getUser } }) => {
  const [field, ...restFields] = fields ?? [];
  if (!field) {
    return null;
  }
  suspense.query(() => getUser(id, field));
  const children = useMemo(
    () => /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Fallback, {}), children: /* @__PURE__ */ jsx(UserWrapper, { id, fields: restFields }) }),
    []
  );
  return /* @__PURE__ */ jsxs("div", { style: { paddingLeft: "50px", textAlign: "left" }, children: [
    /* @__PURE__ */ jsxs("p", { children: [
      /* @__PURE__ */ jsxs("strong", { children: [
        field,
        ":"
      ] }),
      " ",
      user == null ? void 0 : user[field]
    ] }),
    children
  ] });
};
const UserWrapper = withStores(User, stores);
const NestedSuspense = () => /* @__PURE__ */ jsxs(Fragment, { children: [
  /* @__PURE__ */ jsx(Meta, { children: /* @__PURE__ */ jsx("title", { children: "Nested suspense" }) }),
  /* @__PURE__ */ jsx("p", { children: "Wait until all suspense will be resolved." }),
  /* @__PURE__ */ jsx("div", { children: "-------" }),
  /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Fallback, {}), children: /* @__PURE__ */ jsx(UserWrapper, { id: "user-1", fields: ["id", "name", "email"] }) }),
  /* @__PURE__ */ jsx("div", { children: "-------" }),
  /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Fallback, {}), children: /* @__PURE__ */ jsx(UserWrapper, { id: "user-2", fields: ["id", "name", "email"] }) }),
  /* @__PURE__ */ jsx("div", { children: "-------" }),
  /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("home"), children: "Go back" }) })
] });
export {
  NestedSuspense as default
};
