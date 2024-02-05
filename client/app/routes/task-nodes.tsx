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
import { useConnectTaskNode } from "~/features/task-node/use-connect-task-node";
import { buildTaskNodeEdges, buildTaskNodes } from "~/features/task-node/util";
import { useUpdateTaskNodeEdge } from "~/features/task-node/use-update-task-node-edge";
import { toast } from "sonner";
import { api } from "~/api/index.client";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await requireUserSession(request);

  return json({ session });
};

const nodeTypes = { task: TaskNode } as const;

export default function TaskNodesPage() {
  const { session } = useLoaderData<typeof loader>();

  const [nodes, setNodes] = useState<Node<TaskNodeData>[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);

  const { handleConnect } = useConnectTaskNode({ setEdges });
  const { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd } =
    useUpdateTaskNodeEdge({ setEdges });

  const handleAddTaskNode = useCallback((node: Node<TaskNodeData>) => {
    setNodes((nodes) => [...nodes, { ...node, type: "task" }]);
  }, []);
  const { handleNodesChange } = useUpdateTaskNode({ setNodes });

  const handleEdgesChange: OnEdgesChange = useCallback((changes) => {
    setEdges((old) => applyEdgeChanges(changes, old));
  }, []);

  useEffect(() => {
    const fetchNodes = async () => {
      try {
        const nodes = await api.get("/task-nodes");
        setNodes(buildTaskNodes(nodes));
        setEdges(buildTaskNodeEdges(nodes));
      } catch (e) {
        console.error(e);
        toast.error("タスクを読み込めませんでした。");
      }
    };

    fetchNodes();
  }, []);

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
            <TaskNodeForm onAddNode={handleAddTaskNode} />
          </Panel>
        </NodeView>
        <Outlet />
      </div>
    </SessionProvider>
  );
}
