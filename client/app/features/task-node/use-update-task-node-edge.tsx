import { Connection, Edge, useReactFlow } from "reactflow";
import { useCallback, useRef } from "react";
import { useDisconnectSubtask } from "./use-disconnect-subtask";
import { useReconnectSubtask } from "./use-reconnect-subtask";
import {
  subtaskHandle,
  generateSubtaskEdgeId,
  buildTaskNodes,
  buildTaskNodeEdges,
} from "./util";
import { api } from "~/api/index.client";
import { toast } from "sonner";

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

  const updateEdge = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      if (!(newConnection.source && newConnection.target)) {
        return;
      }

      if (newConnection.sourceHandle === subtaskHandle) {
        const newParentTaskId = newConnection.source;
        const newSubtaskId = newConnection.target;

        const id = generateSubtaskEdgeId({
          parentTaskId: newParentTaskId,
          subtaskId: newSubtaskId,
        });
        // Edgeの重複を確認する
        if (flow.getEdges().find((e) => e.id === id)) {
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
            onSuccess: async () => {
              try {
                const taskNodes = await api.get("/task-nodes");
                flow.setNodes(buildTaskNodes(taskNodes));
                flow.setEdges(buildTaskNodeEdges(taskNodes));
              } catch (e) {
                console.error(e);
                toast.error("タスクの読み込みに失敗しました。");
              }
            },
            onError: () => {
              const cacheOldEdge = oldEdge;
              setEdges((eds) => [...eds, cacheOldEdge]);
            },
          }
        );

        setEdges((eds) => eds.filter((e) => e.id !== oldEdge.id));
      }
    },
    [flow, reconnectSubtask, setEdges]
  );

  const handleEdgeUpdateStart = useCallback(() => {
    edgeUpdateSuccessful.current = false;
  }, []);

  const updateSuccessful = useCallback((callback: () => void) => {
    callback();
    edgeUpdateSuccessful.current = true;
  }, []);

  const handleEdgeUpdate = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      updateSuccessful(() => {
        updateEdge(oldEdge, newConnection);
      });
    },
    [updateEdge, updateSuccessful]
  );

  const handleEdgeUpdateEnd = useCallback(
    (_: unknown, edge: Edge) => {
      updateSuccessful(() => {
        // 更新が完了していたら何もしない
        if (edgeUpdateSuccessful.current) {
          return;
        }

        disconnectSubtask.mutate(
          {
            parent_task_id: edge.source,
            subtask_id: edge.target,
          },
          {
            onSuccess: async () => {
              try {
                const taskNodes = await api.get("/task-nodes");
                flow.setNodes(buildTaskNodes(taskNodes));
              } catch (e) {
                console.error(e);
                toast.error("タスクの読み込みに失敗しました。");
              }
            },
            onError: () => {
              const cacheEdge = edge;
              setEdges((eds) => [...eds, cacheEdge]);
            },
          }
        );
        setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      });
    },
    [disconnectSubtask, flow, setEdges, updateSuccessful]
  );

  return { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd };
};
