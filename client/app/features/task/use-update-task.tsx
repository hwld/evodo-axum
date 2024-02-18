import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useUpdateTask = () => {
  const revalidator = useRevalidator();

  return useMutation({
    mutationFn: (data: z.infer<typeof schemas.UpdateTask> & { id: string }) => {
      return api.put(
        "/tasks/:id",
        { title: data.title, description: data.description },
        { params: { id: data.id } }
      );
    },
    onError: (e) => {
      console.error(e);
      toast.error("タスクを更新できませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });
};
