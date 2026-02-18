import { Navigate, Outlet } from "react-router";
import { AuthProvider, useAuth } from "~/lib/auth";

function ProtectedContent() {
  const { user, loading, error } = useAuth();

  if (error) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-destructive/5 p-4">
        <div className="max-w-md rounded-lg border border-destructive/20 bg-card p-6 shadow-sm">
          <h1 className="mb-2 text-lg font-semibold text-destructive">
            Configuration Error
          </h1>
          <p className="text-sm text-muted-foreground">{error}</p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex h-screen w-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/sign-in" replace />;
  }

  return <Outlet />;
}

export default function AppLayout() {
  return (
    <AuthProvider>
      <ProtectedContent />
    </AuthProvider>
  );
}
