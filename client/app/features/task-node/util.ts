import { Edge } from "reactflow";

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
