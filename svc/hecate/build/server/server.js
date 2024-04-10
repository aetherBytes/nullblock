import { StreamSuspense } from "@lomray/consistent-suspense/server/index.js";
import { Meta, MetaManagerProvider, Manager as Manager$2 } from "@lomray/react-head-manager";
import MetaServer from "@lomray/react-head-manager/server/index.js";
import { StoreManagerProvider, Manager as Manager$1 } from "@lomray/react-mobx-manager";
import ManagerStream from "@lomray/react-mobx-manager/manager-stream.js";
import entryServer from "@lomray/vite-ssr-boost/node/entry.js";
import CookieParser from "cookie-parser";
import { isbotPatterns, createIsbotFromList, list } from "isbot";
import { enableStaticRendering } from "mobx-react-lite";
import n from "@lomray/vite-ssr-boost/helpers/import-route.js";
import { jsxs, jsx, Fragment } from "react/jsx-runtime";
import ScrollToTop from "@lomray/vite-ssr-boost/components/scroll-to-top.js";
import { Outlet, useRouteError, isRouteErrorResponse, Link } from "react-router-dom";
import ResponseStatus from "@lomray/vite-ssr-boost/components/response-status.js";
import { Manager } from "@lomray/react-route-manager";
import { Suspense, ConsistentSuspenseProvider } from "@lomray/consistent-suspense";
import { lazy, StrictMode } from "react";
var StateKey = /* @__PURE__ */ ((StateKey2) => {
  StateKey2["storeManager"] = "_storeState_";
  StateKey2["metaManager"] = "_metaState_";
  return StateKey2;
})(StateKey || {});
const AppLayout = () => /* @__PURE__ */ jsxs("div", { children: [
  /* @__PURE__ */ jsx(ScrollToTop, {}),
  /* @__PURE__ */ jsx("main", { className: "main", children: /* @__PURE__ */ jsx(Outlet, {}) })
] });
const IS_PROD = true;
const API_GATEWAY = "https://randomuser.me/api";
const manager = new Manager({
  routes: {
    home: {
      url: "/"
    },
    details: {
      url: "/details",
      children: {
        user: {
          url: "/user/:id",
          params: { id: "" }
        }
      }
    },
    errorBoundary: {
      url: "/error-boundary"
    },
    nestedSuspense: {
      url: "/nested-suspense"
    },
    redirect: {
      url: "/redirect-demo"
    },
    notLazy: {
      url: "/not-lazy"
    }
  }
});
const NotFound = () => {
  const error = useRouteError();
  if (isRouteErrorResponse(error)) {
    return /* @__PURE__ */ jsxs(Fragment, { children: [
      /* @__PURE__ */ jsx(Meta, { children: /* @__PURE__ */ jsx("title", { children: "Not found" }) }),
      /* @__PURE__ */ jsx(ResponseStatus, { status: 404 }),
      /* @__PURE__ */ jsxs("div", { children: [
        "Opps. Page not found. Status: ",
        error.status
      ] }),
      /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("home"), children: "Go home" }) })
    ] });
  }
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsx("div", { children: "Something went wrong." }),
    !IS_PROD,
    !IS_PROD,
    /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("home"), children: "Go home" }) })
  ] });
};
const loadingPane = "_loadingPane_8ouku_1";
const loader = "_loader_8ouku_8";
const spin = "_spin_8ouku_1";
const styles$1 = {
  loadingPane,
  loader,
  spin
};
const Fallback = () => /* @__PURE__ */ jsx("div", { className: styles$1.loadingPane, children: /* @__PURE__ */ jsx("h2", { className: styles$1.loader, children: "🌀" }) });
const container = "_container_pkyon_1";
const text = "_text_pkyon_7";
const styles = {
  container,
  text
};
const CodeSplitting = lazy(() => import("./assets/index-3xe8KF9d.js"));
const NotLazy = () => /* @__PURE__ */ jsxs(Fragment, { children: [
  /* @__PURE__ */ jsxs("div", { className: styles.container, children: [
    /* @__PURE__ */ jsx("div", { className: styles.text, children: "Styled text" }),
    /* @__PURE__ */ jsx("div", { children: /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Fallback, {}), children: /* @__PURE__ */ jsx(CodeSplitting, {}) }) })
  ] }),
  /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx(Link, { to: manager.makeURL("home"), children: "Go back" }) })
] });
const detailsRoutes = [
  {
    index: true,
    lazy: n(() => import("./assets/index-iuHIxk7-.js"), "@pages/details/index")
  },
  {
    path: manager.path("details.user"),
    lazy: n(() => import("./assets/index-CAyabwVt.js"), "@pages/details/user")
  }
];
const routes = [
  {
    ErrorBoundary: NotFound,
    Component: AppLayout,
    pathId: "@components/layouts/app",
    children: [
      {
        index: true,
        lazy: n(() => import("./assets/index-CjL3aL39.js"), "@pages/home")
      },
      {
        path: manager.path("details"),
        children: detailsRoutes
      },
      {
        path: manager.path("errorBoundary"),
        lazy: n(() => import("./assets/index-BavhgPqO.js"), "@pages/error-boundary")
      },
      {
        path: manager.path("nestedSuspense"),
        lazy: n(() => import("./assets/index-DmMLY2qT.js"), "@pages/nested-suspense")
      },
      {
        path: manager.path("redirect"),
        lazy: n(() => import("./assets/index-PZkKyJBC.js"), "@pages/redirect")
      },
      {
        path: manager.path("notLazy"),
        Component: NotLazy,
        pathId: "@pages/not-lazy"
      }
    ]
  }
];
const App = ({ children, client, server: server2 }) => {
  const storeManager = (client == null ? void 0 : client.storeManager) ?? (server2 == null ? void 0 : server2.storeManager);
  const metaManager = (client == null ? void 0 : client.metaManager) ?? (server2 == null ? void 0 : server2.metaManager);
  return /* @__PURE__ */ jsx(ConsistentSuspenseProvider, { children: /* @__PURE__ */ jsx(StoreManagerProvider, { storeManager, children: /* @__PURE__ */ jsx(MetaManagerProvider, { manager: metaManager, children: /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Fallback, {}), children }) }) }) });
};
const AppStrict = (props) => /* @__PURE__ */ jsx(StrictMode, { children: /* @__PURE__ */ jsx(App, { ...props }) });
const patternsToRemove = new Set(["Amazon CloudFront"].map(isbotPatterns).flat());
const isBot = createIsbotFromList(list.filter((record) => !patternsToRemove.has(record)));
const server = entryServer(AppStrict, routes, {
  abortDelay: 2e4,
  init: () => ({
    /**
     * Once after create server
     */
    onServerCreated: (app) => {
      enableStaticRendering(true);
      app.use(CookieParser());
    },
    /**
     * For each request:
     * 1. Create mobx manager
     * 2. Create meta manager
     * 3. Listen stream to add mobx suspense stores to output
     */
    onRequest: async () => {
      const storeManager = new Manager$1({
        options: { shouldDisablePersist: true }
      });
      const storeManageStream = new ManagerStream(storeManager);
      const metaManager = new Manager$2();
      await storeManager.init();
      const streamSuspense = StreamSuspense.create(
        (suspenseId) => storeManageStream.take(suspenseId)
      );
      return {
        appProps: {
          storeManager,
          metaManager,
          streamSuspense
        },
        // disable 103 early hints for AWS ALB
        hasEarlyHints: false
      };
    },
    /**
     * We can control stream mode here
     */
    onRouterReady: ({ context: { req } }) => {
      var _a;
      const isStream = !isBot(req.get("user-agent") || "") && ((_a = req.cookies) == null ? void 0 : _a.isCrawler) !== "1";
      return {
        isStream
      };
    },
    /**
     * Inject header meta tags
     */
    onShellReady: ({
      context: {
        appProps: { metaManager },
        html: { header }
      }
    }) => {
      const newHead = MetaServer.inject(header, metaManager);
      return {
        header: newHead
      };
    },
    /**
     * Analyze react stream output and return additional html from callback `onRequest` in StreamSuspense
     */
    onResponse: ({
      context: {
        appProps: { streamSuspense },
        isStream
      },
      html
    }) => {
      if (!isStream) {
        return;
      }
      return streamSuspense.analyze(html);
    },
    /**
     * Return server state to client (once when app she'll ready) for:
     * 1. Mobx manager (stores)
     * 2. Meta manager
     */
    getState: ({
      context: {
        appProps: { storeManager, metaManager }
      }
    }) => {
      const storeState = storeManager.toJSON();
      const metaState = MetaServer.getState(metaManager);
      return {
        [StateKey.storeManager]: storeState,
        [StateKey.metaManager]: metaState
      };
    }
  })
});
export {
  API_GATEWAY as A,
  Fallback as F,
  server as default,
  manager as m
};
