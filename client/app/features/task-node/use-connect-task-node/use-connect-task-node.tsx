import { useCallback } from "react";
import { Connection, Edge, useReactFlow } from "reactflow";
import {
  subtaskHandle,
  generateSubtaskEdgeId,
  generateSubtaskEdge,
  blockTaskHandle,
  generateBlockTaskEdgeId,
  generateBlockTaskEdge,
} from "../util";
import { useConnectSubtask } from "./use-connect-subtask";
import { useConnectBlockTask } from "./use-connect-block-task";

type UseConnectSubtaskArgs = {
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
};
export const useConnectTaskNode = ({ setEdges }: UseConnectSubtaskArgs) => {
  const flow = useReactFlow();
  const connectSubtack = useConnectSubtask();
  const connectBlockTask = useConnectBlockTask();

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

        connectSubtack.mutate({
          parent_task_id: parentTaskId,
          subtask_id: subtaskId,
        });

        setEdges((old) => {
          return [...old, generateSubtaskEdge({ parentTaskId, subtaskId })];
        });
      } else if (connection.sourceHandle === blockTaskHandle) {
        const blockingTaskId = connection.source;
        const blockedTaskId = connection.target;

        const newEdgeId = generateBlockTaskEdgeId({
          blockingTaskId,
          blockedTaskId,
        });
        if (flow.getEdges().find((e) => e.id === newEdgeId)) {
          return;
        }

        connectBlockTask.mutate({
          blocking_task_id: blockingTaskId,
          blocked_task_id: blockedTaskId,
        });

        setEdges((old) => {
          return [
            ...old,
            generateBlockTaskEdge({ blockingTaskId, blockedTaskId }),
          ];
        });
      }
    },
    [connectBlockTask, connectSubtack, flow, setEdges]
  );

  return { handleConnect };
};
