import { LoaderFunctionArgs, json } from "@remix-run/node";
import { Link, useLoaderData } from "@remix-run/react";
import {
  CircleIcon,
  Clock4Icon,
  HistoryIcon,
  LucideIcon,
  TextIcon,
  XIcon,
} from "lucide-react";
import { ReactNode } from "react";
import { serverFetch } from "~/api/index.server";
import { Button } from "~/components/ui/button";
import { Card } from "~/components/ui/card";
import { TaskDescriptionForm } from "~/features/task/task-description-form";
import { TaskStatusBadge } from "~/features/task/task-status-badge";

export const loader = async ({ request, params }: LoaderFunctionArgs) => {
  const id = params["node-id"];
  if (id === undefined) {
    throw new Error("node-idが存在しません");
  }

  const task = await serverFetch.get("/tasks/:id", {
    params: { id },
    headers: { cookie: request.headers.get("cookie") },
  });
  return json({ task });
};

export default function TaskNodeDetail() {
  const { task } = useLoaderData<typeof loader>();

  return (
    <div className="h-dvh w-[500px] top-0 right-0 fixed p-2 animate-in slide-in-from-right-32">
      <Card className="h-full w-full p-6 flex flex-col">
        <Button
          size="icon"
          variant="ghost"
          className="absolute right-5 top-5"
          asChild
        >
          <Link to="/task-nodes">
            <XIcon />
          </Link>
        </Button>
        <div className="scape-y-1">
          <div className="text-sm text-muted-foreground">タスクの詳細</div>
          <div className="text-2xl font-bold">{task.title}</div>
          <div className="text-xs text-muted-foreground">ID: {task.id}</div>
        </div>

        <VerticalDatailRow icon={CircleIcon} title="状態">
          <div className="ml-2">
            <TaskStatusBadge status={task.status} />
          </div>
        </VerticalDatailRow>

        <div className="mt-5 space-y-1">
          <HorizontalDetailRow
            icon={Clock4Icon}
            title="作成日"
            label={task.created_at}
          />
          <HorizontalDetailRow
            icon={HistoryIcon}
            title="更新日"
            label={task.updated_at}
          />
        </div>

        <VerticalDatailRow icon={TextIcon} title="説明">
          <TaskDescriptionForm defaultTask={task} />
        </VerticalDatailRow>
      </Card>
    </div>
  );
}

type HorizontalDetailRowProps = {
  icon: LucideIcon;
  label: string;
  title: string;
};
const HorizontalDetailRow: React.FC<HorizontalDetailRowProps> = ({
  icon: Icon,
  title,
  label,
}) => {
  return (
    <div className="flex items-center gap-1 w-full">
      <div className="w-[80px] flex gap-1 items-center text-muted-foreground text-sm">
        <Icon size={17} />
        {title}
      </div>
      <div className="">{label}</div>
    </div>
  );
};

type VerticalDetailRowProps = {
  icon: LucideIcon;
  title: string;
  children: ReactNode;
};
const VerticalDatailRow: React.FC<VerticalDetailRowProps> = ({
  icon: Icon,
  title,
  children,
}) => {
  return (
    <div className="space-y-2 mt-5">
      <div className="flex gap-1 text-muted-foreground items-center text-sm">
        <Icon size={17} />
        {title}
      </div>
      {children}
    </div>
  );
};
