import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Edge, useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { endpoints, schemas } from "~/api/schema";
import { generateSubtaskEdge, generateSubtaskEdgeId } from "../util";
import { useTaskNodeViewAction } from "../task-node-view-provider";
import { isErrorFromPath } from "@zodios/core";

export const useReconnectSubtask = () => {
  const flow = useReactFlow();
  const { setTaskNodeEdges } = useTaskNodeViewAction();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ReconnectSubtask>) => {
      return api.put("/subtask/reconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);

      const message = isErrorFromPath(
        endpoints,
        "put",
        "/subtask/reconnect",
        err
      )
        ? getErrorMessage(err.response.data.error_type)
        : "サブタスクをつなげることができませんでした";

      toast.error(message);
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

      setTaskNodeEdges((eds) => [
        ...eds.filter((e) => e.id !== oldSubtaskEdge.id),
        generateSubtaskEdge({
          parentTaskId: newParentTaskId,
          subtaskId: newSubtaskId,
        }),
      ]);
    },
    [flow, mutation, setTaskNodeEdges]
  );

  return { reconnectSubtask };
};

const getErrorMessage = (
  type: z.infer<typeof schemas.ConnectSubtaskErrorType>
): string => {
  switch (type) {
    case "MultipleMainTask": {
      return "複数のメインタスクを持たせることはできません";
    }
    case "BlockedByMainTask": {
      return "ブロックしているタスクをサブタスクにすることはできません";
    }
    case "CircularTask": {
      return "タスクを循環させることはできません";
    }
    case "TaskNotFound": {
      return "タスクが存在しません";
    }
    default: {
      throw new Error(type satisfies never);
    }
  }
};
