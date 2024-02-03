import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useCreateTaskNode = () => {
  return useMutation({
    mutationFn: (
      data: z.infer<typeof schemas.CreateTaskNode> & { id: string }
    ) => {
      return api.post("/task-nodes", {
        task: data.task,
        x: data.x,
        y: data.y,
      });
    },
    onError: (error) => {
      console.error(error);
      toast.error("タスクノードが作成できませんでした。");
    },
  });
};
