import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  isRouteErrorResponse,
  useRouteError,
} from "react-router";
import "./tailwind.css";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body className="antialiased">
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App() {
  return <Outlet />;
}

export function ErrorBoundary() {
  const error = useRouteError();

  let title = "Unexpected Error";
  let message = "Something went wrong.";

  if (isRouteErrorResponse(error)) {
    title = `${error.status} ${error.statusText}`;
    message = error.data?.toString() ?? message;
  } else if (error instanceof Error) {
    title = "Application Error";
    message = error.message;
  }

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-destructive/5 p-4">
      <div className="max-w-md rounded-lg border border-destructive/20 bg-card p-6 shadow-sm">
        <h1 className="mb-2 text-lg font-semibold text-destructive">
          {title}
        </h1>
        <p className="text-sm text-muted-foreground">{message}</p>
      </div>
    </div>
  );
}
