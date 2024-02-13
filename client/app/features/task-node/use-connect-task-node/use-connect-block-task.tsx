import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { endpoints, schemas } from "~/api/schema";
import { generateBlockTaskEdge, generateBlockTaskEdgeId } from "../util";
import { useTaskNodeViewAction } from "../task-node-view-provider";
import { isErrorFromPath } from "@zodios/core";

export const useConnectBlockTask = () => {
  const flow = useReactFlow();
  const { setTaskNodeEdges } = useTaskNodeViewAction();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectBlockTask>) => {
      return api.post("/block-task/connect", { ...data });
    },
    onError: (err) => {
      console.error(err);

      const message = isErrorFromPath(
        endpoints,
        "post",
        "/block-task/connect",
        err
      )
        ? getErrorMessage(err.response.data.error_type)
        : "ブロックタスクをつなげることができませんでした";

      toast.error(message);
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  const connectBlockTask = useCallback(
    ({
      blockingTaskId,
      blockedTaskId,
    }: {
      blockingTaskId: string;
      blockedTaskId: string;
    }) => {
      const newEdgeId = generateBlockTaskEdgeId({
        blockingTaskId,
        blockedTaskId,
      });
      if (flow.getEdges().find((e) => e.id === newEdgeId)) {
        return;
      }

      mutation.mutate({
        blocking_task_id: blockingTaskId,
        blocked_task_id: blockedTaskId,
      });

      setTaskNodeEdges((old) => {
        return [
          ...old,
          generateBlockTaskEdge({ blockingTaskId, blockedTaskId }),
        ];
      });
    },
    [flow, mutation, setTaskNodeEdges]
  );

  return { connectBlockTask };
};

const getErrorMessage = (
  type: z.infer<typeof schemas.ConnectBlockTaskErrorType>
): string => {
  switch (type) {
    case "TaskNotFound": {
      return "タスクが存在しません";
    }
    case "CircularTask": {
      return "タスクを循環させることはできません";
    }
    case "IsSubtask": {
      return "サブタスクをブロックすることはできません";
    }
    default: {
      throw new Error(type satisfies never);
    }
  }
};
