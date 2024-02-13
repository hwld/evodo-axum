import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Edge, useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { endpoints, schemas } from "~/api/schema";
import { generateSubTaskEdge, generateSubTaskEdgeId } from "../util";
import { useTaskNodeViewAction } from "../task-node-view-provider";
import { isErrorFromPath } from "@zodios/core";

export const useReconnectSubTask = () => {
  const flow = useReactFlow();
  const { setTaskNodeEdges } = useTaskNodeViewAction();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ReconnectSubTask>) => {
      return api.put("/sub-task/reconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);

      const message = isErrorFromPath(
        endpoints,
        "put",
        "/sub-task/reconnect",
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

  const reconnectSubTask = useCallback(
    ({
      oldSubTaskEdge,
      newMainTaskId,
      newSubTaskId,
    }: {
      oldSubTaskEdge: Edge;
      newMainTaskId: string;
      newSubTaskId: string;
    }) => {
      const id = generateSubTaskEdgeId({
        mainTaskId: newMainTaskId,
        subTaskId: newSubTaskId,
      });
      // Edgeの重複を確認する
      if (flow.getEdges().find((e) => e.id === id)) {
        return;
      }

      mutation.mutate({
        old_main_task_id: oldSubTaskEdge.source,
        old_sub_task_id: oldSubTaskEdge.target,
        new_main_task_id: newMainTaskId,
        new_sub_task_id: newSubTaskId,
      });

      setTaskNodeEdges((eds) => [
        ...eds.filter((e) => e.id !== oldSubTaskEdge.id),
        generateSubTaskEdge({
          mainTaskId: newMainTaskId,
          subTaskId: newSubTaskId,
        }),
      ]);
    },
    [flow, mutation, setTaskNodeEdges]
  );

  return { reconnectSubTask };
};

const getErrorMessage = (
  type: z.infer<typeof schemas.ConnectSubTaskErrorType>
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
