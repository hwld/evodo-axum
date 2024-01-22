import { CheckCircle2Icon, CircleDashedIcon, XIcon } from "lucide-react";
import { Handle, NodeProps, Position, useReactFlow } from "reactflow";
import { useDeleteTask } from "./use-delete-task-node";
import { Checkbox } from "react-aria-components";
import clsx from "clsx";
import { Task } from "~/api/types";
import { useUpdateTask } from "../task/use-update-task";

export type TaskNodeData = {
  title: string;
  taskId: string;
  status: Task["status"];
};

export const TaskNode = ({ data, id: nodeId }: NodeProps<TaskNodeData>) => {
  const flow = useReactFlow<TaskNodeData>();
  const isChecked = data.status === "Done";

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

  const updateMutation = useUpdateTask();
  const handleUpdateStatus = () => {
    updateMutation.mutate(
      {
        ...data,
        status: data.status === "Todo" ? "Done" : "Todo",
      },
      {
        onSuccess: (task) => {
          flow.setNodes((nodes) =>
            nodes.map((node) => {
              if (node.data.taskId === task.id) {
                return { ...node, data: { ...node.data, status: task.status } };
              }
              return node;
            })
          );
        },
      }
    );
  };

  return (
    <div className="flex border text-sm text-neutral-900 border-neutral-900 min-h-[50px] rounded pl-3 pr-5 bg-neutral-50 items-center relative  break-all max-w-[300px]">
      <Handle
        type="target"
        position={Position.Left}
        className="!w-[8px] !h-[30px] !rounded !bg-neutral-900"
      />
      <Checkbox
        isSelected={isChecked}
        onChange={handleUpdateStatus}
        className="flex items-center group cursor-pointer"
      >
        {({ isSelected, isFocusVisible, isHovered }) => {
          const Icon = isSelected ? CheckCircle2Icon : CircleDashedIcon;
          return (
            <>
              <div
                className={clsx(
                  "transition-colors rounded p-1",
                  (isHovered || isFocusVisible) && "bg-neutral-200"
                )}
              >
                <Icon size={20} className={clsx("text-neutral-700")} />
              </div>
              <span className="text-neutral-700">{data.title}</span>
            </>
          );
        }}
      </Checkbox>
      <button
        className="hover:bg-black/10 transition-colors rounded p-[2px] absolute top-1 right-1 text-neutral-500"
        onClick={handleDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={15} />
      </button>
      <Handle
        type="source"
        id="block"
        position={Position.Right}
        className="!w-[8px] !h-[23px] !rounded !bg-neutral-900 !transform !top-0"
      />
      <Handle
        type="source"
        id="break"
        position={Position.Right}
        className="!w-[8px] !h-[23px] !rounded !bg-neutral-900 !transform !bottom-0 !top-auto"
      />
    </div>
  );
};
