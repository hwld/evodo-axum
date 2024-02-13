import { Connection, Edge } from "@xyflow/react";
import { useCallback, useRef } from "react";
import { useDisconnectSubTask } from "./use-disconnect-sub-task";
import { useReconnectSubTask } from "./use-reconnect-sub-task";
import { subTaskHandle, blockTaskHandle } from "../util";
import { useReconnectBlockTask } from "./use-reconnect-block-task";
import { useDisconnectBlockTask } from "./use-disconnect-block-task";

export const useUpdateTaskNodeEdge = () => {
  const edgeUpdateSuccessful = useRef(true);
  const { reconnectSubTask } = useReconnectSubTask();
  const { disconnectSubTask } = useDisconnectSubTask();
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

      if (newConnection.sourceHandle === subTaskHandle) {
        reconnectSubTask({
          oldSubTaskEdge: oldEdge,
          newMainTaskId: newConnection.source,
          newSubTaskId: newConnection.target,
        });
      } else if (newConnection.sourceHandle === blockTaskHandle) {
        reconnectBlockTask({
          oldBlockTaskEdge: oldEdge,
          newBlockingTaskId: newConnection.source,
          newBlockedTaskId: newConnection.target,
        });
      }
    },
    [reconnectBlockTask, reconnectSubTask]
  );

  const disconnect = useCallback(
    (edge: Edge) => {
      // 更新が完了していたら何もしない
      if (edgeUpdateSuccessful.current) {
        return;
      }

      if (edge.sourceHandle === subTaskHandle) {
        disconnectSubTask(edge);
      } else if (edge.sourceHandle === blockTaskHandle) {
        disconnectBlockTask(edge);
      }
    },
    [disconnectBlockTask, disconnectSubTask]
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
