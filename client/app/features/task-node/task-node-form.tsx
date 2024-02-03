import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { schemas } from "~/api/schema";
import { z } from "zod";
import { useCreateTaskNode } from "./use-create-task-node";
import { Node, useViewport } from "reactflow";
import { TaskNodeData } from "./task-node";
import { useEffect, useRef } from "react";
import { useMergedRef } from "@mantine/hooks";
import { CommandIcon } from "lucide-react";

const createTaskNodeSchema = schemas.CreateTaskNode.pick({ task: true });
type CreateTaskNode = z.infer<typeof createTaskNodeSchema>;

type Props = { onAddNode: (node: Node<TaskNodeData>) => void };
export const TaskNodeForm = ({ onAddNode }: Props) => {
  const viewport = useViewport();

  const createMutation = useCreateTaskNode();
  const {
    register,
    handleSubmit: createHandleSubmit,
    reset,
    clearErrors,
    formState: { errors },
  } = useForm<CreateTaskNode>({
    defaultValues: { task: { title: "" } },
    resolver: zodResolver(createTaskNodeSchema),
  });

  const handleSubmit = createHandleSubmit(({ task }) => {
    const { x, y, zoom } = viewport;
    createMutation.mutate(
      {
        task,
        id: crypto.randomUUID(),
        x: (-x + window.innerWidth / 2) / zoom,
        y: (-y + window.innerHeight / 2) / zoom,
      },
      {
        onSuccess: ({ node_info, task }) => {
          onAddNode({
            id: task.id,
            data: {
              title: task.title,
              taskId: task.id,
              status: task.status,
            },
            position: { x: node_info.x, y: node_info.y },
          });
          reset();
        },
      }
    );
  });
  const { ref, onBlur, ...taskTitleRegister } = register("task.title");
  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    onBlur(e);
    clearErrors();
  };

  const inputElRef = useRef<HTMLInputElement>(null);
  const inputRef = useMergedRef(ref, inputElRef);
  useEffect(() => {
    const focusInput = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        inputElRef.current?.focus();
      }
    };

    window.addEventListener("keydown", focusInput);
    return () => {
      window.removeEventListener("keydown", focusInput);
    };
  }, []);

  return (
    <form onSubmit={handleSubmit}>
      {errors.task?.title?.message && (
        <div className="bg-neutral-900 text-red-400 w-min px-3 py-2 rounded whitespace-nowrap mx-auto mb-2 text-sm">
          {errors.task.title.message}
        </div>
      )}

      <div className="w-[500px] h-[45px] bg-neutral-900 rounded-full flex items-center p-3 transition-shadow focus-within:ring-2 ring-offset-2 ring-neutral-700 shadow-xl">
        <input
          autoComplete="off"
          placeholder="タスクを入力してください..."
          className=" bg-neutral-900 text-neutral-100 focus-visible:outline-none rounded w-full h-full p-1 placeholder:text-neutral-300"
          {...taskTitleRegister}
          ref={inputRef}
          onBlur={handleBlur}
          disabled={createMutation.isPending}
        />
        <div className="flex items-center gap-[2px] text-sm border rounded-full px-2 py-1 text-neutral-300 border-neutral-300 font-bold">
          <CommandIcon size={16} />
          <p>K</p>
        </div>
      </div>
    </form>
  );
};
