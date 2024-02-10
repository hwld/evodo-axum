import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useConnectSubtask = () => {
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectSubtask>) => {
      return api.post("/subtask/connect", {
        ...data,
      });
    },
    onError: (err) => {
      console.error(err);
      toast.error("サブタスクをつなげることができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });
  return mutation;
};
