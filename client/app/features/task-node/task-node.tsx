import { CheckCircle2Icon, CircleDashedIcon, XIcon } from "lucide-react";
import { NodeProps, useReactFlow } from "reactflow";
import { useDeleteTask } from "./use-delete-task-node";
import { Task } from "~/api/types";
import { useUpdateTask } from "../task/use-update-task";
import { Node } from "~/components/ui/node";
import { Checkbox, CheckboxIndicator } from "@radix-ui/react-checkbox";
import { useId } from "react";
import { cn } from "~/lib/utils";

export type TaskNodeData = {
  title: string;
  taskId: string;
  status: Task["status"];
};

type Props = NodeProps<TaskNodeData>;
export const TaskNode: React.FC<Props> = ({ data, id: nodeId }) => {
  const checkboxId = useId();
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
    console.log("?");
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
    <Node
      className={cn(
        "group max-w-[450px] break-all",
        isChecked && "border-green-500"
      )}
    >
      <div className="flex items-center">
        <Checkbox
          checked={isChecked}
          onCheckedChange={handleUpdateStatus}
          id={checkboxId}
        >
          <CheckboxIndicator forceMount>
            {isChecked ? (
              <CheckCircle2Icon className="text-green-500" />
            ) : (
              <CircleDashedIcon className="text-muted-foreground" />
            )}
          </CheckboxIndicator>
        </Checkbox>
        <label className="pl-2 cursor-pointer" htmlFor={checkboxId}>
          {data.title}
        </label>
      </div>
      <button
        className="hover:bg-black/5 rounded p-[2px] absolute top-1 right-1 text-neutral-500 group-hover:opacity-100 opacity-0 transition-[background-color,opacity]"
        onClick={handleDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={15} />
      </button>
    </Node>
  );
};
