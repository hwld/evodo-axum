import {
  BlocksIcon,
  CheckIcon,
  Grid2X2Icon,
  LayoutGridIcon,
  XIcon,
} from "lucide-react";
import { Handle, NodeProps, Position, useReactFlow } from "reactflow";
import { useUpdateTaskStatus } from "../task/use-update-task-status";
import { Checkbox, CheckboxIndicator } from "@radix-ui/react-checkbox";
import { useId, useState } from "react";
import { cn } from "~/lib/utils";
import { Task } from "../task";
import { buildTaskNodeEdges, buildTaskNodes, subtaskHandle } from "./util";
import { api } from "~/api/index.client";
import { toast } from "sonner";
import { Card } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import clsx from "clsx";
import { DeleteTaskDialog } from "./delete-task-dialog";
import { UpdateTaskDialog } from "./update-task-dialog";
import { useDeleteTask } from "./use-delete-task-node";

export type TaskNodeData = {
  title: string;
  taskId: string;
  status: Task["status"];
  type: "normal" | "main" | "sub";
};

type Props = NodeProps<TaskNodeData>;
export const TaskNode: React.FC<Props> = ({ data }) => {
  const checkboxId = useId();
  const flow = useReactFlow<TaskNodeData>();
  const isChecked = data.status === "Done";

  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const handleTriggerDelete = () => {
    setIsDeleteDialogOpen(true);
  };

  const deleteMutation = useDeleteTask();
  const handleDelete = () => {
    deleteMutation.mutate(
      { taskId: data.taskId },
      {
        onSuccess: async () => {
          try {
            const nodes = await api.get("/task-nodes");
            flow.setNodes(buildTaskNodes(nodes));
            flow.setEdges(buildTaskNodeEdges(nodes));
          } catch (e) {
            console.error(e);
            toast.error("タスクを読み込めませんでした。");
          }
        },
      }
    );
  };

  const [isUpdateDialogOpen, setIsUpdateDialogOpen] = useState(false);
  const updateMutation = useUpdateTaskStatus();
  const handleTriggerUpdateStatus = () => {
    if (data.type === "main") {
      setIsUpdateDialogOpen(true);
      return;
    }

    handleUpdateStatus();
  };

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
    <Card
      className={cn(
        "group min-w-[250px] max-w-[450px] break-all relative flex flex-col gap-1 p-2 transition-colors",
        isChecked && "border-green-500"
      )}
    >
      <div className="flex gap-1 text-muted-foreground items-center">
        {
          {
            sub: (
              <>
                <BlocksIcon size={16} />
                <p className="text-xs">サブタスク</p>
              </>
            ),
            main: (
              <>
                <LayoutGridIcon size={16} />
                <p className="text-xs">メインタスク</p>
              </>
            ),
            normal: (
              <>
                <Grid2X2Icon size={16} />
                <p className="text-xs">タスク</p>
              </>
            ),
          }[data.type]
        }
      </div>
      <Separator
        className={cn("transition-colors", isChecked && "bg-green-500")}
      />
      <div
        className={clsx("flex items-center p-1 transition-colors grow rounded")}
      >
        <Checkbox
          checked={isChecked}
          onCheckedChange={handleTriggerUpdateStatus}
          id={checkboxId}
          className={clsx(
            "shrink-0 size-[20px] border-2 rounded flex items-center justify-center data-[state=checked]:border-green-500 data-[state=checked]:bg-green-50 text-green-500 transition-colors relative hover:bg-green-50 hover:data-[state=checked]:text-green-400 hover:data-[state=checked]:border-green-400"
          )}
          disabled={updateMutation.isPending}
        >
          <CheckboxIndicator>
            <CheckIcon size={13} strokeWidth={3} />
          </CheckboxIndicator>
        </Checkbox>
        <label className={clsx("pl-1 cursor-pointer")} htmlFor={checkboxId}>
          {data.title}
        </label>
      </div>

      <DeleteTaskDialog
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        onDelete={handleDelete}
      />

      <UpdateTaskDialog
        open={isUpdateDialogOpen}
        onOpenChange={setIsUpdateDialogOpen}
        onUpdate={handleUpdateStatus}
      />

      <button
        className="rounded p-[2px] absolute top-1 right-1 text-neutral-500 group-hover:opacity-100 opacity-0 transition-[background-color,opacity] bg-primary text-primary-foreground hover:bg-primary/80"
        onClick={handleTriggerDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={20} />
      </button>
      <Handle
        type="target"
        position={Position.Left}
        className="!-left-[10px] !size-5 !rounded-full !bg-primary-foreground !border !border-neutral-300 shadow"
      />
      <Handle
        type="source"
        id={subtaskHandle}
        position={Position.Right}
        className="!-right-[10px] !size-5 !rounded-full !bg-primary-foreground !border !border-neutral-300 shadow"
      />
    </Card>
  );
};
