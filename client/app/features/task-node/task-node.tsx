import {
  BlocksIcon,
  CheckIcon,
  Grid2X2Icon,
  LayoutGridIcon,
  ShieldHalfIcon,
  SplitIcon,
} from "lucide-react";
import { Handle, NodeProps, Position, useReactFlow } from "reactflow";
import { useUpdateTaskStatus } from "../task/use-update-task-status";
import { Checkbox, CheckboxIndicator } from "@radix-ui/react-checkbox";
import { useId, useState } from "react";
import { cn } from "~/lib/utils";
import { Task } from "../task";
import {
  blockTaskHandle,
  buildTaskNodeEdges,
  buildTaskNodes,
  subtaskHandle,
} from "./util";
import { Card } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import clsx from "clsx";
import { UpdateTaskDialog } from "./update-task-dialog";
import { TaskNodeMenu } from "./task-node-menu";
import { useRevalidator } from "@remix-run/react";
import { toast } from "sonner";
import { api } from "~/api/index.client";

export type TaskNodeData = {
  title: string;
  taskId: string;
  status: Task["status"];
  type: "normal" | "main" | "sub";
};

type Props = NodeProps<TaskNodeData>;
export const TaskNode: React.FC<Props> = ({ data }) => {
  const revalidator = useRevalidator();
  const checkboxId = useId();
  const flow = useReactFlow<TaskNodeData>();
  const isChecked = data.status === "Done";

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

          revalidator.revalidate();
        },
      }
    );
  };

  return (
    <Card
      className={cn(
        "group justify-items-start gap-[2px] content-start min-w-[250px] max-w-[450px] break-all relative grid grid-cols-[1fr_5px_25px] grid-rows-[20px_5px_20px_1fr] p-2 transition-colors",
        isChecked && "border-green-500"
      )}
    >
      <div className="col-start-1 row-start-1 flex gap-1 text-muted-foreground items-center h-5 relative w-full">
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
        <TaskNodeMenu
          taskId={data.taskId}
          className="opacity-0 group-hover:opacity-100 absolute -top-[6px] right-0"
        />
      </div>
      <Separator
        className={cn(
          "col-start-1 row-start-2 transition-colors self-center",
          isChecked && "bg-green-500"
        )}
      />
      <div
        className={clsx(
          "col-start-1 row-start-3 row-span-2 flex items-center p-1 transition-colors grow rounded"
        )}
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

      <Separator
        orientation="vertical"
        className={cn(
          "row-start-1 row-span-4 col-start-2 justify-self-center",
          isChecked && "bg-green-500"
        )}
      />

      <div className="row-start-1 col-start-3 col-span-1 text-muted-foreground w-full h-full flex justify-center items-center relative">
        <SplitIcon size={15} />
        <Handle
          type="source"
          id={subtaskHandle}
          position={Position.Right}
          className="!-right-[0px] !left-full !top-[50%] !size-[20px] !rounded-full !bg-primary-foreground !border !border-neutral-300 shadow"
        />
      </div>
      <Separator
        className={cn(
          "row-start-2 col-start-3 self-center",
          isChecked && "bg-green-500"
        )}
      />
      <div className="row-start-3 col-start-3 text-muted-foreground w-full h-full flex justify-center relative">
        <ShieldHalfIcon size={15} />
        <Handle
          type="source"
          id={blockTaskHandle}
          position={Position.Right}
          className="!-right-[0px] !left-full !top-[50%] !size-[20px] !rounded-full !bg-primary-foreground !border !border-neutral-300 shadow"
        />
      </div>

      <UpdateTaskDialog
        open={isUpdateDialogOpen}
        onOpenChange={setIsUpdateDialogOpen}
        onUpdate={handleUpdateStatus}
      />
      <Handle
        type="target"
        position={Position.Left}
        className="!-left-[10px] !size-5 !rounded-full !bg-primary-foreground !border !border-neutral-300 shadow"
      />
    </Card>
  );
};
