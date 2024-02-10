import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useReconnectBlockTask = () => {
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ReconnectBlockTask>) => {
      return api.put("/block-task/reconnect", { ...data });
    },
    onError: (err) => {
      console.log("errorr");
      console.error(err);
      toast.error("ブロックタスクをつなぐことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });
  return mutation;
};
