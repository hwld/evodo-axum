import { redirect } from "@remix-run/node";
import { api } from "./api";

export const requireUserSession = async (request: Request) => {
  const cookie = request.headers.get("cookie");
  const { session } = await api.get("/auth/session", { headers: { cookie } });

  if (!session) {
    throw redirect("/login");
  }

  return session;
};

export const requireSignupSession = async (request: Request) => {
  const cookie = request.headers.get("cookie");
  const session = await api.get("/auth/signup-session", {
    headers: { cookie },
  });

  if (!session.session_exists) {
    throw redirect("/");
  }

  return session;
};
