import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Edge, useReactFlow } from "reactflow";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";
import { generateSubtaskEdge, generateSubtaskEdgeId } from "../util";

export const useReconnectSubtask = () => {
  const flow = useReactFlow();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ReconnectSubtask>) => {
      return api.put("/subtask/reconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("サブタスクをつなぐことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  const reconnectSubtask = useCallback(
    ({
      oldSubtaskEdge,
      newParentTaskId,
      newSubtaskId,
    }: {
      oldSubtaskEdge: Edge;
      newParentTaskId: string;
      newSubtaskId: string;
    }) => {
      const id = generateSubtaskEdgeId({
        parentTaskId: newParentTaskId,
        subtaskId: newSubtaskId,
      });
      // Edgeの重複を確認する
      if (flow.getEdges().find((e) => e.id === id)) {
        return;
      }

      mutation.mutate({
        old_parent_task_id: oldSubtaskEdge.source,
        old_subtask_id: oldSubtaskEdge.target,
        new_parent_task_id: newParentTaskId,
        new_subtask_id: newSubtaskId,
      });

      flow.setEdges((eds) => [
        ...eds.filter((e) => e.id !== oldSubtaskEdge.id),
        generateSubtaskEdge({
          parentTaskId: newParentTaskId,
          subtaskId: newSubtaskId,
        }),
      ]);
    },
    [flow, mutation]
  );

  return { reconnectSubtask };
};
