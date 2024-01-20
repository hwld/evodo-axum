import { useMutation, useQueryClient } from "@tanstack/react-query";
import { z } from "zod";
import { api } from "~/api";
import { schemas } from "~/api/schema";

export const useCreateTask = () => {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (data: z.infer<typeof schemas.CreateTask>) => {
      return api.post("/tasks", { ...data });
    },
    onError: (error) => {
      console.error(error);
      window.alert("タスクが作成できませんでした。");
    },
    onSettled: () => {
      return client.invalidateQueries();
    },
  });
};
