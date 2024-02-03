import { useMutation } from "@tanstack/react-query";
import { useSession } from "./use-session";
import { api } from "~/api/index.client";
import { useNavigate } from "@remix-run/react";
import { toast } from "sonner";

export const useAuth = () => {
  const session = useSession();

  const navigate = useNavigate();
  const logoutMutation = useMutation({
    mutationFn: () => {
      return api.post("/auth/logout", undefined);
    },
    onError: (err) => {
      console.error(err);
      toast.error("ログアウトできませんでした。");
    },
    onSuccess: () => {
      navigate("/login");
    },
  });

  return { session, logoutMutation };
};
