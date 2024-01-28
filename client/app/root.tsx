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
import stylesheet from "~/tailwind.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: stylesheet },
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
          <Outlet />
        </QueryClientProvider>
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
