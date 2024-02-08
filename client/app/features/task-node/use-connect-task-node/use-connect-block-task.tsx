import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useConnectBlockTask = () => {
  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectBlockTask>) => {
      return api.post("/block-task/connect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("ブロックタスクをつなげることができませんでした。");
    },
  });

  return mutation;
};
