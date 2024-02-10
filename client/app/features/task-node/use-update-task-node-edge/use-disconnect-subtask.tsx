import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useDisconnectSubtask = () => {
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.DisconnectSubtask>) => {
      return api.delete("/subtask/disconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("サブタスクを切り離すことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  return mutation;
};
