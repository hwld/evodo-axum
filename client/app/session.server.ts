import { redirect } from "@remix-run/node";
import { serverFetch } from "./api/index.server";

export const requireUserSession = async (request: Request) => {
  const cookie = request.headers.get("cookie");
  const { session } = await serverFetch.get("/auth/session", {
    headers: { cookie },
  });

  if (!session) {
    throw redirect("/login");
  }

  return session;
};

export const requireSignupSession = async (request: Request) => {
  const cookie = request.headers.get("cookie");
  const session = await serverFetch.get("/auth/signup-session", {
    headers: { cookie },
  });

  if (!session.session_exists) {
    throw redirect("/");
  }

  return session;
};
