import { LoaderFunctionArgs, json } from "@remix-run/node";
import { useLoaderData, useNavigate } from "@remix-run/react";
import { motion, useAnimate } from "framer-motion";
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
  const navigate = useNavigate();

  const [scope, animate] = useAnimate();
  const handleClose = async () => {
    // 画面遷移したときにexitアニメーションを実行する方法がわからないので
    // closeボタンが押されたときだけアニメーションを実行する
    await animate(scope.current, { x: 128, opacity: 0 });
    navigate("/task-nodes");
  };

  return (
    <motion.div
      ref={scope}
      className="h-dvh w-[500px] top-0 right-0 fixed p-2"
      initial={{ opacity: 0, x: 128 }}
      animate={{ opacity: 1, x: 0 }}
    >
      <Card className="h-full w-full overflow-hidden">
        <div className="overflow-auto p-6 h-full">
          <Button
            size="icon"
            variant="ghost"
            className="absolute right-5 top-5"
            onClick={handleClose}
          >
            <XIcon />
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
            <TaskDescriptionForm defaultTask={task} key={task.id} />
          </VerticalDatailRow>
        </div>
      </Card>
    </motion.div>
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
