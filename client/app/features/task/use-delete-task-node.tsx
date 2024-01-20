import { useMutation } from "@tanstack/react-query";
import { api } from "~/api";

export const useDeleteTask = () => {
  return useMutation({
    mutationFn: ({ taskId }: { taskId: string }) => {
      return api.delete("/tasks/:id", undefined, { params: { id: taskId } });
    },
    onError: (err) => {
      console.error(err);
      window.alert("タスクを削除できませんでした。");
    },
  });
};
