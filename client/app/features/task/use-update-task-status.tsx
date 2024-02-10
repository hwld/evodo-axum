import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useUpdateTaskStatus = () => {
  const revalidator = useRevalidator();

  return useMutation({
    mutationFn: (
      data: z.infer<typeof schemas.UpdateTaskStatus> & { taskId: string }
    ) => {
      return api.put(
        "/tasks/:id/update-status",
        { status: data.status },
        { params: { id: data.taskId } }
      );
    },
    onError: (err) => {
      console.error(err);
      toast.error("タスクを更新できませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });
};
