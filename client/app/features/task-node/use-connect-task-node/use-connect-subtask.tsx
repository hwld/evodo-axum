import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { useReactFlow } from "reactflow";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";
import { generateSubtaskEdge, generateSubtaskEdgeId } from "../util";

export const useConnectSubtask = () => {
  const flow = useReactFlow();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectSubtask>) => {
      return api.post("/subtask/connect", {
        ...data,
      });
    },
    onError: (err) => {
      console.error(err);
      toast.error("サブタスクをつなげることができませんでした。");
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

      flow.setEdges((old) => {
        return [...old, generateSubtaskEdge({ parentTaskId, subtaskId })];
      });
    },
    [flow, mutation]
  );

  return { connectSubtask };
};
