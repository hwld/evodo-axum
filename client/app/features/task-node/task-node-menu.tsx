import clsx from "clsx";
import { PanelRightOpenIcon, XIcon } from "lucide-react";
import { ComponentProps, ReactNode, useState } from "react";
import { DeleteTaskDialog } from "../task/delete-task-dialog";
import { useDeleteTask } from "./use-delete-task-node";
import {
  NavLink,
  NavLinkProps,
  useMatches,
  useNavigate,
} from "@remix-run/react";

type Props = { taskId: string; className?: string };
export const TaskNodeMenu: React.FC<Props> = ({ taskId, className }) => {
  const navigate = useNavigate();
  const matches = useMatches();

  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const handleTriggerDelete = () => {
    setIsDeleteDialogOpen(true);
  };

  const deleteMutation = useDeleteTask();
  const handleDelete = () => {
    deleteMutation.mutate(
      { taskId },
      {
        onSuccess: () => {
          if (matches.some(({ id }) => id === "routes/task-nodes.$node-id")) {
            navigate("/task-nodes", { replace: true });
          }
        },
      }
    );
  };

  return (
    <div
      className={clsx(
        "flex gap-1  p-[2px] bg-primary/90 text-primary-foreground rounded transition-opacity z-",
        className
      )}
    >
      <TaskNodeMenuLink to={taskId}>
        <PanelRightOpenIcon size={18} />
      </TaskNodeMenuLink>

      <TaskNodeMenuButton
        onClick={handleTriggerDelete}
        disabled={deleteMutation.isPending}
      >
        <XIcon size={20} />
      </TaskNodeMenuButton>

      <DeleteTaskDialog
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        onDelete={handleDelete}
      />
    </div>
  );
};

const menuItemClass =
  "rounded size-6 transition-[background-color,opacity] hover:bg-primary-foreground/20 flex justify-center items-center ";

type ButtonProps = ComponentProps<"button">;
const TaskNodeMenuButton: React.FC<ButtonProps> = ({
  children,
  className,
  ...props
}) => {
  return (
    <button {...props} className={clsx(menuItemClass, className)}>
      {children}
    </button>
  );
};

type LinkProps = NavLinkProps & { children: ReactNode };
const TaskNodeMenuLink: React.FC<LinkProps> = ({
  children,
  className,
  ...props
}) => {
  return (
    <NavLink {...props} className={clsx(menuItemClass, className)}>
      {children}
    </NavLink>
  );
};
