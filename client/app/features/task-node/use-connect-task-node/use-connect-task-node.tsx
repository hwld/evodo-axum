import { useCallback } from "react";
import { Connection } from "@xyflow/react";
import { subTaskHandle, blockTaskHandle } from "../util";
import { useConnectSubTask } from "./use-connect-sub-task";
import { useConnectBlockTask } from "./use-connect-block-task";

export const useConnectTaskNode = () => {
  const { connectSubTask } = useConnectSubTask();
  const { connectBlockTask } = useConnectBlockTask();

  const handleConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) {
        return;
      }

      if (connection.sourceHandle === subTaskHandle) {
        connectSubTask({
          parentTaskId: connection.source,
          subTaskId: connection.target,
        });
      } else if (connection.sourceHandle === blockTaskHandle) {
        connectBlockTask({
          blockingTaskId: connection.source,
          blockedTaskId: connection.target,
        });
      }
    },
    [connectBlockTask, connectSubTask]
  );

  return { handleConnect };
};
