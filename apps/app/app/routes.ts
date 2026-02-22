import { type RouteConfig, layout, route } from "@react-router/dev/routes";

export default [
  layout("layouts/auth.tsx", [
    route("sign-in", "routes/sign-in.tsx"),
    route("sign-up", "routes/sign-up.tsx"),
  ]),
  layout("layouts/app.tsx", [
    route("/", "routes/index.tsx"),
    route("pricing", "routes/pricing.tsx"),
  ]),
] satisfies RouteConfig;
