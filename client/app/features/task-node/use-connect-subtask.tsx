import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useConnectSubtask = () => {
  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectSubtask>) => {
      return api.post("/subtask/connect", {
        ...data,
      });
    },
    onError: (err) => {
      console.error(err);
      window.alert("サブタスクをつなげることができませんでした。");
    },
  });
  return mutation;
};
