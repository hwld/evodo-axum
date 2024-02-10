import { useCallback } from "react";
import { Connection } from "reactflow";
import { subtaskHandle, blockTaskHandle } from "../util";
import { useConnectSubtask } from "./use-connect-subtask";
import { useConnectBlockTask } from "./use-connect-block-task";

export const useConnectTaskNode = () => {
  const { connectSubtask } = useConnectSubtask();
  const { connectBlockTask } = useConnectBlockTask();

  const handleConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) {
        return;
      }

      if (connection.sourceHandle === subtaskHandle) {
        connectSubtask({
          parentTaskId: connection.source,
          subtaskId: connection.target,
        });
      } else if (connection.sourceHandle === blockTaskHandle) {
        connectBlockTask({
          blockingTaskId: connection.source,
          blockedTaskId: connection.target,
        });
      }
    },
    [connectBlockTask, connectSubtask]
  );

  return { handleConnect };
};
