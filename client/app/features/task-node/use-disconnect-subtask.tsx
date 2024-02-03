import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useDisconnectSubtask = () => {
  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.DisconnectSubtask>) => {
      return api.delete("/subtask/disconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      window.alert("サブタスクを切り離すことができませんでした。");
    },
  });

  return mutation;
};
