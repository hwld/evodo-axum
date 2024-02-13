import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { endpoints, schemas } from "~/api/schema";
import { generateSubtaskEdge, generateSubtaskEdgeId } from "../util";
import { useTaskNodeViewAction } from "../task-node-view-provider";
import { isErrorFromPath } from "@zodios/core";

export const useConnectSubtask = () => {
  const flow = useReactFlow();
  const { setTaskNodeEdges } = useTaskNodeViewAction();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectSubtask>) => {
      return api.post("/subtask/connect", {
        ...data,
      });
    },
    onError: (err) => {
      console.error(err);

      const message = isErrorFromPath(
        endpoints,
        "post",
        "/subtask/connect",
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

  const connectSubtask = useCallback(
    ({
      parentTaskId,
      subtaskId,
    }: {
      parentTaskId: string;
      subtaskId: string;
    }) => {
      const newEdgeId = generateSubtaskEdgeId({ parentTaskId, subtaskId });
      if (flow.getEdges().find((e) => e.id === newEdgeId)) {
        return;
      }

      mutation.mutate({
        parent_task_id: parentTaskId,
        subtask_id: subtaskId,
      });

      setTaskNodeEdges((old) => {
        return [...old, generateSubtaskEdge({ parentTaskId, subtaskId })];
      });
    },
    [flow, mutation, setTaskNodeEdges]
  );

  return { connectSubtask };
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
