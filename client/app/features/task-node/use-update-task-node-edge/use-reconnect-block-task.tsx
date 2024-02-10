import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Edge, useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";
import { generateBlockTaskEdge, generateBlockTaskEdgeId } from "../util";
import { useTaskNodeViewAction } from "../task-node-view-provider";

export const useReconnectBlockTask = () => {
  const flow = useReactFlow();
  const { setTaskNodeEdges } = useTaskNodeViewAction();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ReconnectBlockTask>) => {
      return api.put("/block-task/reconnect", { ...data });
    },
    onError: (err) => {
      console.log("errorr");
      console.error(err);
      toast.error("ブロックタスクをつなぐことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  const reconnectBlockTask = useCallback(
    ({
      oldBlockTaskEdge,
      newBlockingTaskId,
      newBlockedTaskId,
    }: {
      oldBlockTaskEdge: Edge;
      newBlockingTaskId: string;
      newBlockedTaskId: string;
    }) => {
      const id = generateBlockTaskEdgeId({
        blockingTaskId: newBlockingTaskId,
        blockedTaskId: newBlockedTaskId,
      });
      if (flow.getEdges().find((e) => e.id === id)) {
        return;
      }

      mutation.mutate({
        old_blocking_task_id: oldBlockTaskEdge.source,
        old_blocked_task_id: oldBlockTaskEdge.target,
        new_blocking_task_id: newBlockingTaskId,
        new_blocked_task_id: newBlockedTaskId,
      });

      setTaskNodeEdges((eds) => [
        ...eds.filter((e) => e.id !== oldBlockTaskEdge.id),
        generateBlockTaskEdge({
          blockingTaskId: newBlockingTaskId,
          blockedTaskId: newBlockedTaskId,
        }),
      ]);
    },
    [flow, mutation, setTaskNodeEdges]
  );

  return { reconnectBlockTask };
};
