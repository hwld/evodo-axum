import { CheckCircle2Icon, CircleDashedIcon, XIcon } from "lucide-react";
import { Handle, NodeProps, Position, useReactFlow } from "reactflow";
import { useDeleteTask } from "./use-delete-task-node";
import { useUpdateTaskStatus } from "../task/use-update-task-status";
import { Node } from "~/components/ui/node";
import { Checkbox, CheckboxIndicator } from "@radix-ui/react-checkbox";
import { useId } from "react";
import { cn } from "~/lib/utils";
import { Task } from "../task";
import { buildTaskNodeEdges, buildTaskNodes, subtaskHandle } from "./util";
import { api } from "~/api/index.client";
import { toast } from "sonner";

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

  const updateMutation = useUpdateTaskStatus();
  const handleUpdateStatus = () => {
    updateMutation.mutate(
      {
        taskId: data.taskId,
        status: data.status === "Todo" ? "Done" : "Todo",
      },
      {
        onSuccess: async () => {
          try {
            const taskNodes = await api.get("/task-nodes");
            flow.setNodes(buildTaskNodes(taskNodes));
            flow.setEdges(buildTaskNodeEdges(taskNodes));
          } catch (e) {
            console.error(e);
            toast.error("タスクの読み込みに失敗しました。");
          }
        },
      }
    );
  };

  return (
    <Node
      className={cn(
        "group max-w-[450px] break-all relative",
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
        className="rounded p-[2px] absolute top-1 right-1 text-neutral-500 group-hover:opacity-100 opacity-0 transition-[background-color,opacity] bg-primary text-primary-foreground hover:bg-primary/80"
        onClick={handleDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={20} />
      </button>
      <Handle
        type="target"
        position={Position.Left}
        className="!-left-5 !size-4 !rounded-sm !bg-transparent !border !border-neutral-300 shadow"
      />
      <Handle
        type="source"
        id={subtaskHandle}
        position={Position.Right}
        className="!-right-5 !top-0 !translate-y-0 !size-4 !rounded-sm !bg-transparent !border !border-neutral-300 shadow"
      />
      <Handle
        type="source"
        id="block"
        position={Position.Right}
        className="!-right-5 !bottom-0 !top-auto !translate-y-0 !size-4 !rounded-sm !bg-transparent !border !border-neutral-300 shadow"
      />
    </Node>
  );
};
