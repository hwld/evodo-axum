import { LoaderFunctionArgs, json } from "@remix-run/node";
import { Outlet, useLoaderData } from "@remix-run/react";
import { requireUserSession } from "~/session.server";
import { SessionProvider } from "~/features/auth/use-session";
import { serverFetch } from "~/api/index.server";
import { TaskNodeViewProvider } from "~/features/task-node/task-node-view-provider";
import { TaskNodeView } from "~/features/task-node/task-node-view";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await requireUserSession(request);
  const taskNodeData = await serverFetch.get("/task-nodes", {
    headers: { cookie: request.headers.get("cookie") },
  });

  return json({ session, taskNodeData });
};

export default function TaskNodesPage() {
  const { session, taskNodeData } = useLoaderData<typeof loader>();

  return (
    <div className="h-dvh">
      <SessionProvider session={session}>
        <TaskNodeViewProvider taskNodeData={taskNodeData}>
          <TaskNodeView />
          <Outlet />
        </TaskNodeViewProvider>
      </SessionProvider>
    </div>
  );
}
