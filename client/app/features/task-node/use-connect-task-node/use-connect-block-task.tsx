import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";
import { generateBlockTaskEdge, generateBlockTaskEdgeId } from "../util";

export const useConnectBlockTask = () => {
  const flow = useReactFlow();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectBlockTask>) => {
      return api.post("/block-task/connect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("ブロックタスクをつなげることができませんでした。");
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

      flow.setEdges((old) => {
        return [
          ...old,
          generateBlockTaskEdge({ blockingTaskId, blockedTaskId }),
        ];
      });
    },
    [flow, mutation]
  );

  return { connectBlockTask };
};
