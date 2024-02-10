import { Connection, Edge, useReactFlow } from "reactflow";
import { useCallback, useRef } from "react";
import { useDisconnectSubtask } from "./use-disconnect-subtask";
import { useReconnectSubtask } from "./use-reconnect-subtask";
import {
  subtaskHandle,
  generateSubtaskEdgeId,
  blockTaskHandle,
  generateBlockTaskEdgeId,
} from "../util";
import { useReconnectBlockTask } from "./use-reconnect-block-task";
import { useDisconnectBlockTask } from "./use-disconnect-block-task";

type UseUpdateTaskNodeEdgeArgs = {
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
};
export const useUpdateTaskNodeEdge = ({
  setEdges,
}: UseUpdateTaskNodeEdgeArgs) => {
  const flow = useReactFlow();
  const edgeUpdateSuccessful = useRef(true);
  const reconnectSubtask = useReconnectSubtask();
  const disconnectSubtask = useDisconnectSubtask();
  const reconnectBlockTask = useReconnectBlockTask();
  const disconnectBlockTask = useDisconnectBlockTask();

  const updateEdge = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      if (
        !(newConnection.source && newConnection.target) ||
        oldEdge.sourceHandle !== newConnection.sourceHandle
      ) {
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

        reconnectSubtask.mutate({
          old_parent_task_id: oldEdge.source,
          old_subtask_id: oldEdge.target,
          new_parent_task_id: newConnection.source,
          new_subtask_id: newConnection.target,
        });

        setEdges((eds) => eds.filter((e) => e.id !== oldEdge.id));
      } else if (newConnection.sourceHandle === blockTaskHandle) {
        const newBlockingTaskId = newConnection.source;
        const newBlockedTaskId = newConnection.target;

        const id = generateBlockTaskEdgeId({
          blockingTaskId: newBlockingTaskId,
          blockedTaskId: newBlockedTaskId,
        });
        if (flow.getEdges().find((e) => e.id === id)) {
          return;
        }

        reconnectBlockTask.mutate({
          old_blocking_task_id: oldEdge.source,
          old_blocked_task_id: oldEdge.target,
          new_blocking_task_id: newConnection.source,
          new_blocked_task_id: newConnection.target,
        });

        setEdges((eds) => eds.filter((e) => e.id !== oldEdge.id));
      }
    },
    [flow, reconnectBlockTask, reconnectSubtask, setEdges]
  );

  const disconnectEdge = useCallback(
    (edge: Edge) => {
      // 更新が完了していたら何もしない
      if (edgeUpdateSuccessful.current) {
        return;
      }

      if (edge.sourceHandle === subtaskHandle) {
        disconnectSubtask.mutate({
          parent_task_id: edge.source,
          subtask_id: edge.target,
        });

        setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      } else if (edge.sourceHandle === blockTaskHandle) {
        disconnectBlockTask.mutate({
          blocking_task_id: edge.source,
          blocked_task_id: edge.target,
        });

        setEdges((eds) => eds.filter((e) => e.id !== edge.id));
      }
    },
    [disconnectBlockTask, disconnectSubtask, setEdges]
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
        disconnectEdge(edge);
      });
    },
    [disconnectEdge, updateSuccessful]
  );

  return { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd };
};
