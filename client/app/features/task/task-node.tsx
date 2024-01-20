import { XIcon } from "lucide-react";
import {
  Handle,
  NodeProps,
  Position,
  useNodeId,
  useReactFlow,
} from "reactflow";
import { useDeleteTask } from "./use-delete-task-node";

export type TaskNodeData = {
  title: string;
  taskId: string;
};

export const TaskNode = ({ data }: NodeProps<TaskNodeData>) => {
  const nodeId = useNodeId();
  const flow = useReactFlow<TaskNodeData>();

  const deleteMutation = useDeleteTask();

  const handleDelete = () => {
    if (!nodeId) {
      return;
    }

    const node = flow.getNode(nodeId);
    if (!node) {
      return;
    }

    deleteMutation.mutate(
      { taskId: node.data.taskId },
      {
        onSuccess: () => {
          flow.deleteElements({ nodes: [{ id: nodeId }] });
        },
      }
    );
  };

  return (
    <div className="flex border border-neutral-900 rounded px-5 py-1 bg-neutral-50 items-center relative">
      <Handle
        type="target"
        position={Position.Left}
        className="!w-[8px] !h-[8px] !bg-neutral-900"
      />
      <p>{data.title}</p>
      <button
        className="hover:bg-black/10 transition-colors rounded p-[2px] absolute top-0 right-0 text-neutral-500"
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
