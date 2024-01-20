import { XIcon } from "lucide-react";
import { Handle, NodeProps, Position, useReactFlow } from "reactflow";
import { useDeleteTask } from "./use-delete-task-node";

export type TaskNodeData = {
  title: string;
  taskId: string;
};

export const TaskNode = ({ data, id: nodeId }: NodeProps<TaskNodeData>) => {
  const flow = useReactFlow<TaskNodeData>();
  const deleteMutation = useDeleteTask();

  const handleDelete = () => {
    deleteMutation.mutate(
      { taskId: data.taskId },
      {
        onSuccess: () => {
          flow.deleteElements({ nodes: [{ id: nodeId }] });
        },
      }
    );
  };

  return (
    <div className="flex border border-neutral-900 rounded px-5 py-3 bg-neutral-50 items-center relative w-[300px] break-all">
      <Handle
        type="target"
        position={Position.Left}
        className="!w-[8px] !h-[8px] !bg-neutral-900"
      />
      <p>{data.title}</p>
      <button
        className="hover:bg-black/10 transition-colors rounded p-[2px] absolute top-1 right-1 text-neutral-500"
        onClick={handleDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={15} />
      </button>
      <Handle
        type="source"
        position={Position.Right}
        className="!w-[8px] !h-[8px] !rounded !bg-neutral-900"
      />
    </div>
  );
};
