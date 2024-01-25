import { useNavigate } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api";
import { schemas } from "~/api/schema";

export const useSignup = () => {
  const navigate = useNavigate();
  return useMutation({
    mutationFn: (data: z.infer<typeof schemas.CreateUser>) => {
      return api.post("/signup", { ...data });
    },
    onError: (err) => {
      console.error(err);
      window.alert("新規登録ができませんでした。");
    },
    onSuccess: () => {
      navigate("/", { replace: true });
    },
  });
};
