import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useCreateTask = () => {
  return useMutation({
    mutationFn: (data: z.infer<typeof schemas.CreateTask>) => {
      return api.post("/tasks", { ...data });
    },
    onError: (error) => {
      console.error(error);
      window.alert("タスクが作成できませんでした。");
    },
  });
};
