import { jsx, jsxs, Fragment } from "react/jsx-runtime";
import { Suspense } from "@lomray/consistent-suspense";
import { F as Fallback } from "../server.js";
import { useNavigate, useRouteError } from "react-router-dom";
const DefaultSuspense = ({ children }) => /* @__PURE__ */ jsx(Suspense, { fallback: /* @__PURE__ */ jsx(Fallback, {}), children });
const ErrorBoundary = () => {
  const navigate = useNavigate();
  const error = useRouteError();
  const message = error && error.message || "Unknown error";
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsxs("div", { children: [
      "Boom! Error: ",
      message
    ] }),
    /* @__PURE__ */ jsx("div", { className: "mr20", children: /* @__PURE__ */ jsx("button", { type: "button", onClick: () => navigate(-1), children: "Go back" }) })
  ] });
};
export {
  DefaultSuspense as D,
  ErrorBoundary as E
};
