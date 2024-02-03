import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { api } from "~/api/index.client";

export const useDeleteTask = () => {
  return useMutation({
    mutationFn: ({ taskId }: { taskId: string }) => {
      return api.delete("/tasks/:id", undefined, {
        params: { id: taskId },
      });
    },
    onError: (err) => {
      console.error(err);
      toast.error("タスクを削除できませんでした。");
    },
  });
};
