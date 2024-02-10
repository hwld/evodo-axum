import { Connection, Edge } from "reactflow";
import { useCallback, useRef } from "react";
import { useDisconnectSubtask } from "./use-disconnect-subtask";
import { useReconnectSubtask } from "./use-reconnect-subtask";
import { subtaskHandle, blockTaskHandle } from "../util";
import { useReconnectBlockTask } from "./use-reconnect-block-task";
import { useDisconnectBlockTask } from "./use-disconnect-block-task";

export const useUpdateTaskNodeEdge = () => {
  const edgeUpdateSuccessful = useRef(true);
  const { reconnectSubtask } = useReconnectSubtask();
  const { disconnectSubtask } = useDisconnectSubtask();
  const { reconnectBlockTask } = useReconnectBlockTask();
  const { disconnectBlockTask } = useDisconnectBlockTask();

  const reconnect = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      if (
        !(newConnection.source && newConnection.target) ||
        oldEdge.sourceHandle !== newConnection.sourceHandle
      ) {
        return;
      }

      if (newConnection.sourceHandle === subtaskHandle) {
        reconnectSubtask({
          oldSubtaskEdge: oldEdge,
          newParentTaskId: newConnection.source,
          newSubtaskId: newConnection.target,
        });
      } else if (newConnection.sourceHandle === blockTaskHandle) {
        reconnectBlockTask({
          oldBlockTaskEdge: oldEdge,
          newBlockingTaskId: newConnection.source,
          newBlockedTaskId: newConnection.target,
        });
      }
    },
    [reconnectBlockTask, reconnectSubtask]
  );

  const disconnect = useCallback(
    (edge: Edge) => {
      // 更新が完了していたら何もしない
      if (edgeUpdateSuccessful.current) {
        return;
      }

      if (edge.sourceHandle === subtaskHandle) {
        disconnectSubtask(edge);
      } else if (edge.sourceHandle === blockTaskHandle) {
        disconnectBlockTask(edge);
      }
    },
    [disconnectBlockTask, disconnectSubtask]
  );

  const handleEdgeUpdateStart = useCallback(() => {
    edgeUpdateSuccessful.current = false;
  }, []);

  const handleEdgeUpdate = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      reconnect(oldEdge, newConnection);
      edgeUpdateSuccessful.current = true;
    },
    [reconnect]
  );

  const handleEdgeUpdateEnd = useCallback(
    (_: unknown, edge: Edge) => {
      disconnect(edge);
      edgeUpdateSuccessful.current = true;
    },
    [disconnect]
  );

  return { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd };
};
