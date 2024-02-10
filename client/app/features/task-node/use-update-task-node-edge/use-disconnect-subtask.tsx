import { useRevalidator } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Edge, useReactFlow } from "reactflow";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";

export const useDisconnectSubtask = () => {
  const flow = useReactFlow();
  const revalidator = useRevalidator();

  const mutation = useMutation({
    mutationFn: (data: z.infer<typeof schemas.DisconnectSubtask>) => {
      return api.delete("/subtask/disconnect", { ...data });
    },
    onError: (err) => {
      console.error(err);
      toast.error("サブタスクを切り離すことができませんでした。");
    },
    onSettled: () => {
      revalidator.revalidate();
    },
  });

  const disconnectSubtask = useCallback(
    (edge: Edge) => {
      mutation.mutate({
        parent_task_id: edge.source,
        subtask_id: edge.target,
      });

      flow.setEdges((eds) => eds.filter((e) => e.id !== edge.id));
    },
    [flow, mutation]
  );

  return { disconnectSubtask };
};
