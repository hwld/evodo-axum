import { Edge, Node } from "reactflow";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { TaskNodeData } from "./task-node";

type SubtaskConnection = { parentTaskId: string; subtaskId: string };
export const generateSubtaskEdgeId = ({
  parentTaskId,
  subtaskId,
}: SubtaskConnection) => {
  return `subtask-${parentTaskId}-${subtaskId}`;
};

export const subtaskHandle = "sub";

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

export type TaskNode = z.infer<typeof schemas.TaskNode>;
export const buildTaskNodes = (taskNodes: TaskNode[]): Node<TaskNodeData>[] => {
  return taskNodes.map(({ task, node_info }): Node<TaskNodeData> => {
    return {
      type: "task",
      id: task.id,
      data: {
        title: task.title,
        taskId: task.id,
        status: task.status,
      },
      position: { x: node_info.x, y: node_info.y },
    };
  });
};

export const buildTaskNodeEdges = (taskNodes: TaskNode[]): Edge[] => {
  return taskNodes
    .map(({ task }): Edge[] => {
      return task.subtask_ids.map((subtaskId): Edge => {
        return generateSubtaskEdge({
          parentTaskId: task.id,
          subtaskId: subtaskId,
        });
      });
    })
    .flat();
};
