import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useDisconnectBlockTask = () => {
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.DisconnectBlockTask>) => {
      return api.delete("/block-task/disconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("ブロックタスクを切り離すことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  return mutation;
};
