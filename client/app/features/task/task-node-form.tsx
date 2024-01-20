import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { schemas } from "~/api/schema";
import { z } from "zod";
import { useCreateTaskNode } from "./use-create-task-node";
import { Node } from "reactflow";
import { TaskNodeData } from "./task-node";

const createTaskNodeSchema = schemas.CreateTaskNode.pick({ task: true });
type CreateTaskNode = z.infer<typeof createTaskNodeSchema>;

type Props = { onAddNode: (node: Node<TaskNodeData>) => void };
export const TaskNodeForm = ({ onAddNode }: Props) => {
  const createMutation = useCreateTaskNode();
  const {
    register,
    handleSubmit: createHandleSubmit,
    clearErrors,
    formState: { errors },
  } = useForm<CreateTaskNode>({
    defaultValues: { task: { title: "" } },
    resolver: zodResolver(createTaskNodeSchema),
  });

  const handleSubmit = createHandleSubmit(({ task }) => {
    createMutation.mutate(
      { task, id: crypto.randomUUID(), x: 0, y: 0 },
      {
        onSuccess: ({ node_info, task }) => {
          onAddNode({
            id: node_info.id,
            data: { title: task.title, taskId: task.id },
            position: { x: node_info.x, y: node_info.y },
          });
        },
      }
    );
  });

  const { onBlur, ...taskTitleRegister } = register("task.title");
  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    onBlur(e);
    clearErrors();
  };

  return (
    <form onSubmit={handleSubmit}>
      {errors.task?.title?.message && (
        <div className="bg-neutral-900 text-red-400 w-min px-3 py-2 rounded whitespace-nowrap mx-auto mb-2 text-sm">
          {errors.task.title.message}
        </div>
      )}

      <div className="w-[500px] h-[45px] bg-neutral-900 rounded-full flex items-center p-3 transition-shadow focus-within:ring-2 ring-offset-2 ring-neutral-700">
        <input
          autoComplete="off"
          className=" bg-neutral-900 text-neutral-100 focus-visible:outline-none rounded w-full h-full p-1"
          {...taskTitleRegister}
          onBlur={handleBlur}
          disabled={createMutation.isPending}
        />
      </div>
    </form>
  );
};
