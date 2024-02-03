import { Connection, Edge, useReactFlow } from "reactflow";
import { useCallback, useRef } from "react";
import { useDisconnectSubtask } from "./use-disconnect-subtask";
import { useReconnectSubtask } from "./use-reconnect-subtask";
import {
  subtaskHandle,
  generateSubtaskEdgeId,
  generateSubtaskEdge,
} from "./util";

type UseUpdateTaskNodeEdgeArgs = {
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
};
export const useUpdateTaskNodeEdge = ({
  setEdges,
}: UseUpdateTaskNodeEdgeArgs) => {
  const flow = useReactFlow();
  const edgeUpdateSuccessful = useRef(true);
  const disconnectSubtask = useDisconnectSubtask();
  const reconnectSubtask = useReconnectSubtask();

  const handleEdgeUpdateStart = useCallback(() => {
    edgeUpdateSuccessful.current = false;
  }, []);

  const handleEdgeUpdate = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      // TODO: 関数化してedgeUpdateSuccessfulの更新をまとめる
      if (!(newConnection.source && newConnection.target)) {
        edgeUpdateSuccessful.current = true;
        return;
      }

      if (newConnection.sourceHandle === subtaskHandle) {
        const newParentTaskId = newConnection.source;
        const newSubtaskId = newConnection.target;

        const id = generateSubtaskEdgeId({
          parentTaskId: newParentTaskId,
          subtaskId: newSubtaskId,
        });
        if (flow.getEdges().find((e) => e.id === id)) {
          edgeUpdateSuccessful.current = true;
          return;
        }

        reconnectSubtask.mutate(
          {
            old_parent_task_id: oldEdge.source,
            old_subtask_id: oldEdge.target,
            new_parent_task_id: newConnection.source,
            new_subtask_id: newConnection.target,
          },
          {
            onSuccess: () => {
              setEdges((eds) => [
                ...eds,
                generateSubtaskEdge({
                  parentTaskId: newParentTaskId,
                  subtaskId: newSubtaskId,
                }),
              ]);
            },
            onError: () => {
              const cacheOldEdge = oldEdge;
              setEdges((eds) => [...eds, cacheOldEdge]);
            },
          }
        );

        setEdges((eds) => eds.filter((e) => e.id !== oldEdge.id));
      }

      edgeUpdateSuccessful.current = true;
    },
    [flow, reconnectSubtask, setEdges]
  );

  const handleEdgeUpdateEnd = useCallback(
    (_: unknown, edge: Edge) => {
      if (!edgeUpdateSuccessful.current) {
        disconnectSubtask.mutate(
          {
            parent_task_id: edge.source,
            subtask_id: edge.target,
          },
          {
            onError: () => {
              const cacheEdge = edge;
              setEdges((eds) => [...eds, cacheEdge]);
            },
          }
        );
        setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      }
      edgeUpdateSuccessful.current = true;
    },
    [disconnectSubtask, setEdges]
  );

  return { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd };
};
