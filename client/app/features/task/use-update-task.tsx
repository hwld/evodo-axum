import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api";
import { schemas } from "~/api/schema";

export const useUpdateTask = () => {
  return useMutation({
    mutationFn: (
      data: z.infer<typeof schemas.UpdateTask> & { taskId: string }
    ) => {
      return api.put(
        "/tasks/:id",
        { title: data.title, status: data.status },
        { params: { id: data.taskId } }
      );
    },
    onError: (err) => {
      console.error(err);
      window.alert("タスクを更新できませんでした。");
    },
  });
};
