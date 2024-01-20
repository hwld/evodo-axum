import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Node } from "reactflow";
import { z } from "zod";
import { api } from "~/api";
import { schemas } from "~/api/schema";

type UseCreateTaskNodeArgs = { onAddNode?: (node: Node) => void };
export const useCreateTaskNode = ({ onAddNode }: UseCreateTaskNodeArgs) => {
  const client = useQueryClient();
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
    onSuccess: ({ task, node_info }) => {
      onAddNode?.({
        id: node_info.id,
        data: { label: task.title },
        position: { x: node_info.x, y: node_info.y },
      });
    },
    onSettled: () => {
      return client.invalidateQueries();
    },
  });
};
