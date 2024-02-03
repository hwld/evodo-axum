import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useReconnectSubtask = () => {
  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ReconnectSubtask>) => {
      return api.put("/subtask/reconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      window.alert("サブタスクをつなぐことができませんでした。");
    },
  });
  return mutation;
};
