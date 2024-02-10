import { Edge, Node } from "@xyflow/react";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { TaskNode as TaskNodeComponent, TaskNodeViewData } from "./task-node";

export const nodeTypes = { task: TaskNodeComponent } as const;

type SubtaskConnection = { parentTaskId: string; subtaskId: string };
export const generateSubtaskEdgeId = ({
  parentTaskId,
  subtaskId,
}: SubtaskConnection) => {
  return `subtask-${parentTaskId}-${subtaskId}`;
};

type BlockTaskConnection = { blockingTaskId: string; blockedTaskId: string };
export const generateBlockTaskEdgeId = ({
  blockingTaskId,
  blockedTaskId,
}: BlockTaskConnection) => {
  return `blockTask-${blockingTaskId}-${blockedTaskId}`;
};

export const subtaskHandle = "sub";
export const blockTaskHandle = "block";

export const generateSubtaskEdge = ({
  parentTaskId,
  subtaskId,
}: SubtaskConnection): Edge => {
  return {
    id: generateSubtaskEdgeId({ parentTaskId, subtaskId }),
    source: parentTaskId,
    target: subtaskId,
    sourceHandle: subtaskHandle,
    targetHandle: "",
  };
};

export const generateBlockTaskEdge = ({
  blockingTaskId,
  blockedTaskId,
}: BlockTaskConnection): Edge => {
  return {
    id: generateBlockTaskEdgeId({ blockingTaskId, blockedTaskId }),
    source: blockingTaskId,
    target: blockedTaskId,
    sourceHandle: blockTaskHandle,
    targetHandle: "",
  };
};

export const generateTaskNode = ({
  task,
  node_info,
  type = "normal",
}: TaskNodeData & {
  type?: "main" | "sub" | "normal";
}): Node<TaskNodeViewData> => {
  return {
    type: "task",
    id: task.id,
    data: {
      type,
      title: task.title,
      taskId: task.id,
      status: task.status,
    },
    position: { x: node_info.x, y: node_info.y },
  };
};

export type TaskNodeData = z.infer<typeof schemas.TaskNode>;
export const buildTaskNodes = (
  taskNodes: TaskNodeData[]
): Node<TaskNodeViewData>[] => {
  const allSubtasks = taskNodes.map(({ task }) => task.subtask_ids).flat();

  return taskNodes.map(({ task, node_info }): Node<TaskNodeViewData> => {
    const getType = () => {
      if (task.subtask_ids.length > 0) {
        return "main";
      } else if (allSubtasks.includes(task.id)) {
        return "sub";
      } else {
        return "normal";
      }
    };

    return generateTaskNode({ task, node_info, type: getType() });
  });
};

export const buildTaskNodeEdges = (taskNodes: TaskNodeData[]): Edge[] => {
  return taskNodes
    .map(({ task }): Edge[] => {
      const subtaskEdges = task.subtask_ids.map((subtaskId): Edge => {
        return generateSubtaskEdge({
          parentTaskId: task.id,
          subtaskId: subtaskId,
        });
      });

      const blockTaskEdges = task.blocked_task_ids.map(
        (blockedTaskId): Edge => {
          return generateBlockTaskEdge({
            blockingTaskId: task.id,
            blockedTaskId,
          });
        }
      );

      return [...subtaskEdges, ...blockTaskEdges];
    })
    .flat();
};
