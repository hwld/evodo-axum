export const generateSubtaskEdgeId = ({
  parentTaskId,
  subtaskId,
}: {
  parentTaskId: string;
  subtaskId: string;
}) => {
  return `subtask-${parentTaskId}-${subtaskId}`;
};
