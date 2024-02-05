import type { LinksFunction } from "@remix-run/node";
import {
  Links,
  LiveReload,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLoaderData,
} from "@remix-run/react";
import { json } from "@remix-run/node";
import tailwindStylesheet from "~/tailwind.css";
import reactFlowStyles from "reactflow/dist/style.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { Toaster } from "./components/ui/sonner";
import { ReactFlowProvider } from "reactflow";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: tailwindStylesheet },
  { rel: "stylesheet", href: reactFlowStyles },
];

export const loader = async () => {
  return json({
    ENV: {
      BACKEND_URL: process.env.BACKEND_URL || "",
    },
  });
};

export default function App() {
  const [queryClient] = useState(() => new QueryClient({}));
  const data = useLoaderData<typeof loader>();

  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>evodo-axum</title>
        <Meta />
        <Links />
      </head>
      <body>
        <QueryClientProvider client={queryClient}>
          <ReactFlowProvider>
            <Outlet />
          </ReactFlowProvider>
        </QueryClientProvider>
        <Toaster />
        <ScrollRestoration />
        <Scripts />
        <LiveReload />
        <script
          dangerouslySetInnerHTML={{
            __html: `window.ENV = ${JSON.stringify(data.ENV)}`,
          }}
        />
      </body>
    </html>
  );
}
