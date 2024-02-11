import { Edge, Node } from "@xyflow/react";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { TaskNode as TaskNodeComponent } from "./task-node";
import { SubtaskEdge } from "./subtask-edge";
import { BlockTaskEdge } from "./block-task-edge";
import { Task } from "../task";

export type TaskNodeViewData = {
  title: string;
  taskId: string;
  status: Task["status"];
  type: "normal" | "main" | "sub";
  isBlocked: boolean;
};

export const nodeTypes = { task: TaskNodeComponent } as const;
export const edgeTypes = { sub: SubtaskEdge, block: BlockTaskEdge } as const;

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
    type: "sub",
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
    type: "block",
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
  isBlocked,
}: TaskNodeData & {
  type?: "main" | "sub" | "normal";
  isBlocked: boolean;
}): Node<TaskNodeViewData> => {
  return {
    type: "task",
    id: task.id,
    data: {
      type,
      title: task.title,
      taskId: task.id,
      status: task.status,
      isBlocked,
    },
    position: { x: node_info.x, y: node_info.y },
  };
};

const calcAllBlockedTasks = (
  tasks: TaskNodeData[],
  blockedIds: Set<string> = new Set()
): Set<string> => {
  let result: string[] = [];
  if (blockedIds.size === 0) {
    // 直近のブロックされているタスクを取得する
    result = tasks
      .filter(({ task }) => task.status === "Todo")
      .map(({ task }) => task.blocked_task_ids)
      .flat();
  } else {
    // ブロックされているタスクのサブタスクを取得する
    result = tasks
      .filter(({ task }) => {
        return blockedIds.has(task.id);
      })
      .map(({ task }) => {
        return task.subtask_ids;
      })
      .flat();
  }

  const blockedSet = new Set(result);
  if (blockedSet.size === 0) {
    return new Set();
  }

  return new Set([...blockedSet, ...calcAllBlockedTasks(tasks, blockedSet)]);
};

export type TaskNodeData = z.infer<typeof schemas.TaskNode>;
export const buildTaskNodes = (
  taskNodes: TaskNodeData[]
): Node<TaskNodeViewData>[] => {
  const allSubtasks = new Set(
    taskNodes.map(({ task }) => task.subtask_ids).flat()
  );
  const blockedTasks = calcAllBlockedTasks(taskNodes);

  return taskNodes.map(({ task, node_info }): Node<TaskNodeViewData> => {
    const getType = () => {
      if (task.subtask_ids.length > 0) {
        return "main";
      } else if (allSubtasks.has(task.id)) {
        return "sub";
      } else {
        return "normal";
      }
    };

    return generateTaskNode({
      task,
      node_info,
      type: getType(),
      isBlocked: blockedTasks.has(task.id),
    });
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
