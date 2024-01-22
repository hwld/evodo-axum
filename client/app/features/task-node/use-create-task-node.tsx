import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api";
import { schemas } from "~/api/schema";

export const useCreateTaskNode = () => {
  return useMutation({
    mutationFn: (
      data: z.infer<typeof schemas.CreateTaskNode> & { id: string }
    ) => {
      return api.post("/task-nodes", { task: data.task, x: data.x, y: data.y });
    },
    onError: (error) => {
      console.error(error);
      window.alert("タスクノードが作成できませんでした。");
    },
  });
};
