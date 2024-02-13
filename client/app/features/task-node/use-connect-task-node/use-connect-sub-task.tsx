import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { endpoints, schemas } from "~/api/schema";
import { generateSubTaskEdge, generateSubTaskEdgeId } from "../util";
import { useTaskNodeViewAction } from "../task-node-view-provider";
import { isErrorFromPath } from "@zodios/core";

export const useConnectSubTask = () => {
  const flow = useReactFlow();
  const { setTaskNodeEdges } = useTaskNodeViewAction();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectSubTask>) => {
      return api.post("/sub-task/connect", {
        ...data,
      });
    },
    onError: (err) => {
      console.error(err);

      const message = isErrorFromPath(
        endpoints,
        "post",
        "/sub-task/connect",
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

  const connectSubTask = useCallback(
    ({
      parentTaskId,
      subTaskId,
    }: {
      parentTaskId: string;
      subTaskId: string;
    }) => {
      const newEdgeId = generateSubTaskEdgeId({
        parentTaskId,
        subTaskId,
      });
      if (flow.getEdges().find((e) => e.id === newEdgeId)) {
        return;
      }

      mutation.mutate({
        parent_task_id: parentTaskId,
        sub_task_id: subTaskId,
      });

      setTaskNodeEdges((old) => {
        return [
          ...old,
          generateSubTaskEdge({ parentTaskId, subTaskId: subTaskId }),
        ];
      });
    },
    [flow, mutation, setTaskNodeEdges]
  );

  return { connectSubTask: connectSubTask };
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
