import {
  CheckCircle2Icon,
  CircleDashedIcon,
  GripVerticalIcon,
  XIcon,
} from "lucide-react";
import { NodeProps, useReactFlow } from "reactflow";
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
    <div className="flex group gap-1 py-2 border text-sm text-neutral-900 border-neutral-900 min-h-[50px] rounded pl-1 pr-4 bg-neutral-50 items-center relative  break-all max-w-[300px]">
      <div className="shrink-0">
        <GripVerticalIcon className="text-neutral-300" />
      </div>
      <Checkbox
        isSelected={isChecked}
        onChange={handleUpdateStatus}
        className="flex items-center group cursor-pointer pr-1 rounded data-[hovered=true]:bg-black/5 transition-colors"
      >
        {({ isSelected }) => {
          const Icon = isSelected ? CheckCircle2Icon : CircleDashedIcon;
          return (
            <>
              <div className={clsx("transition-colors rounded p-1")}>
                <Icon size={20} className={clsx("text-neutral-700")} />
              </div>
              <span className="text-neutral-700">{data.title}</span>
            </>
          );
        }}
      </Checkbox>
      <button
        className="hover:bg-black/5 rounded p-[2px] absolute top-1 right-1 text-neutral-500 group-hover:opacity-100 opacity-0 transition-[background-color,opacity]"
        onClick={handleDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={15} />
      </button>
      <div className="shrink-0">
        <GripVerticalIcon className="text-neutral-300" />
      </div>
    </div>
  );
};
