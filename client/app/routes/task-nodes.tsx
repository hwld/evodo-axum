import { LoaderFunctionArgs, json } from "@remix-run/node";
import { Edge, Node, OnEdgesChange, Panel, applyEdgeChanges } from "reactflow";
import { TaskNodeForm } from "~/features/task-node/task-node-form";
import { Outlet, useLoaderData } from "@remix-run/react";
import { TaskNode, TaskNodeData } from "~/features/task-node/task-node";
import { useCallback, useEffect, useState } from "react";
import { useUpdateTaskNode } from "~/features/task-node/use-update-task-node";
import { requireUserSession } from "~/session.server";
import { AppControl } from "~/components/app-control/app-control";
import { SessionProvider } from "~/features/auth/use-session";
import { NodeView } from "~/components/node-view";
import { useConnectTaskNode } from "~/features/task-node/use-connect-task-node/use-connect-task-node";
import { buildTaskNodeEdges, buildTaskNodes } from "~/features/task-node/util";
import { useUpdateTaskNodeEdge } from "~/features/task-node/use-update-task-node-edge/use-update-task-node-edge";
import { serverFetch } from "~/api/index.server";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await requireUserSession(request);
  const taskNodes = await serverFetch.get("/task-nodes", {
    headers: { cookie: request.headers.get("cookie") },
  });

  return json({ session, taskNodes });
};

const nodeTypes = { task: TaskNode } as const;

export default function TaskNodesPage() {
  const { session, taskNodes } = useLoaderData<typeof loader>();

  const [nodes, setNodes] = useState<Node<TaskNodeData>[]>(
    buildTaskNodes(taskNodes)
  );
  const [edges, setEdges] = useState<Edge[]>(buildTaskNodeEdges(taskNodes));

  const { handleConnect } = useConnectTaskNode({ setEdges });
  const { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd } =
    useUpdateTaskNodeEdge({ setEdges });

  const { handleNodesChange } = useUpdateTaskNode({ setNodes });

  const handleEdgesChange: OnEdgesChange = useCallback((changes) => {
    setEdges((old) => applyEdgeChanges(changes, old));
  }, []);

  useEffect(() => {
    setNodes(buildTaskNodes(taskNodes));
    setEdges(buildTaskNodeEdges(taskNodes));
  }, [taskNodes]);

  return (
    <SessionProvider session={session}>
      <div className="h-dvh">
        <NodeView
          nodeTypes={nodeTypes}
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onEdgeUpdateStart={handleEdgeUpdateStart}
          onEdgeUpdate={handleEdgeUpdate}
          onEdgeUpdateEnd={handleEdgeUpdateEnd}
          onConnect={handleConnect}
        >
          <Panel position="top-center">
            <AppControl />
          </Panel>
          <Panel position="bottom-center">
            <TaskNodeForm />
          </Panel>
        </NodeView>
        <Outlet />
      </div>
    </SessionProvider>
  );
}
