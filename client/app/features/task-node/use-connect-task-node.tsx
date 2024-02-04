import { useCallback } from "react";
import { Connection, Edge, useReactFlow } from "reactflow";
import {
  subtaskHandle,
  generateSubtaskEdgeId,
  generateSubtaskEdge,
  buildTaskNodes,
  buildTaskNodeEdges,
} from "./util";
import { useConnectSubtask } from "./use-connect-subtask";
import { api } from "~/api/index.client";
import { toast } from "sonner";

type UseConnectSubtaskArgs = {
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
};
export const useConnectTaskNode = ({ setEdges }: UseConnectSubtaskArgs) => {
  const flow = useReactFlow();
  const connectSubtack = useConnectSubtask();

  const handleConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) {
        return;
      }

      if (connection.sourceHandle === subtaskHandle) {
        const parentTaskId = connection.source;
        const subtaskId = connection.target;

        const newEdgeId = generateSubtaskEdgeId({ parentTaskId, subtaskId });
        if (flow.getEdges().find((e) => e.id === newEdgeId)) {
          return;
        }

        const oldEdges = flow.getEdges();
        connectSubtack.mutate(
          {
            parent_task_id: parentTaskId,
            subtask_id: subtaskId,
          },
          {
            onSuccess: async () => {
              try {
                const nodes = await api.get("/task-nodes");
                flow.setNodes(buildTaskNodes(nodes));
                flow.setEdges(buildTaskNodeEdges(nodes));
              } catch (e) {
                console.error(e);
                toast.error("タスクを読み込めませんでした。");
              }
            },
            onError: () => {
              setEdges(oldEdges);
            },
          }
        );

        setEdges((old) => {
          return [...old, generateSubtaskEdge({ parentTaskId, subtaskId })];
        });
      }
    },
    [connectSubtack, flow, setEdges]
  );

  return { handleConnect };
};
