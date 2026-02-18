import { Outlet } from "react-router";

export default function AuthLayout() {
  return (
    <div className="relative flex min-h-screen items-center justify-center bg-gradient-to-br from-blue-50 via-white to-blue-100">
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute -top-40 -left-40 h-80 w-80 rounded-full bg-blue-200 opacity-20 blur-3xl" />
        <div className="absolute -bottom-40 -right-40 h-80 w-80 rounded-full bg-primary opacity-20 blur-3xl" />
      </div>
      <div className="relative z-10 w-full max-w-sm">
        <Outlet />
      </div>
    </div>
  );
}
