import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";
import { Connection, Edge, useReactFlow } from "reactflow";
import { z } from "zod";
import { api } from "~/api/index.client";
import { schemas } from "~/api/schema";
import { generateSubtaskEdgeId } from "./util";

type UseConnectSubtaskArgs = {
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
};
export const useConnectTaskNode = ({ setEdges }: UseConnectSubtaskArgs) => {
  const flow = useReactFlow();

  const connectSubtack = useMutation({
    mutationFn: (data: z.infer<typeof schemas.ConnectSubtask>) => {
      return api.post("/subtask/connect", {
        ...data,
      });
    },
    onError: (err) => {
      console.error(err);
      window.alert("サブタスクをつなげることができませんでした。");
    },
  });

  const handleConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) {
        return;
      }

      // TODO: 文字列を直接扱いたくない・・・
      if (connection.sourceHandle === "sub") {
        const parentTaskId = connection.source;
        const subtaskId = connection.target;

        const newEdgeId = generateSubtaskEdgeId({ parentTaskId, subtaskId });
        if (flow.getEdges().find((e) => e.id === newEdgeId)) {
          return;
        }

        // TODO: connectionが循環していないかを確認する
        connectSubtack.mutate(
          {
            parent_task_id: parentTaskId,
            subtask_id: subtaskId,
          },
          {
            onSuccess: () => {
              setEdges((old) => {
                return [
                  ...old,
                  { id: newEdgeId, source: parentTaskId, target: subtaskId },
                ];
              });
            },
          }
        );
      }
    },
    [connectSubtack, flow, setEdges]
  );

  return { handleConnect };
};
