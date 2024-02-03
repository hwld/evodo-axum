import { useNavigate } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useSignup = () => {
  const navigate = useNavigate();
  return useMutation({
    mutationFn: (data: z.infer<typeof schemas.CreateUser>) => {
      return api.post("/auth/signup", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("新規登録ができませんでした。");
    },
    onSuccess: () => {
      navigate("/", { replace: true });
    },
  });
};
