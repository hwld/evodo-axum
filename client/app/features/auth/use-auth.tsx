import { useMutation } from "@tanstack/react-query";
import { useSession } from "./use-session";
import { api } from "~/api";
import { useNavigate } from "@remix-run/react";

export const useAuth = () => {
  const session = useSession();

  const navigate = useNavigate();
  const logoutMutation = useMutation({
    mutationFn: () => {
      return api.post("/auth/logout", undefined);
    },
    onError: (err) => {
      console.error(err);
      window.alert("ログアウトできませんでした。");
    },
    onSuccess: () => {
      navigate("/login");
    },
  });

  return { session, logoutMutation };
};
