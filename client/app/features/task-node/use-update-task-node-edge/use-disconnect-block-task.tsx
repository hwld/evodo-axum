import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Edge, useReactFlow } from "@xyflow/react";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useDisconnectBlockTask = () => {
  const flow = useReactFlow();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.DisconnectBlockTask>) => {
      return api.delete("/block-task/disconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("ブロックタスクを切り離すことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  const disconnectBlockTask = useCallback(
    (edge: Edge) => {
      mutation.mutate({
        blocking_task_id: edge.source,
        blocked_task_id: edge.target,
      });

      flow.setEdges((eds) => eds.filter((e) => e.id !== edge.id));
    },
    [flow, mutation]
  );

  return { disconnectBlockTask };
};
