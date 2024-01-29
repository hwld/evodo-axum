import { LoaderFunctionArgs, json } from "@remix-run/node";
import {
  Node,
  OnNodesChange,
  Panel,
  applyNodeChanges,
  useEdgesState,
} from "reactflow";
import { TaskNodeForm } from "~/features/task-node/task-node-form";
import { useLoaderData } from "@remix-run/react";
import { TaskNode, TaskNodeData } from "~/features/task-node/task-node";
import { useState } from "react";
import { useUpdateTaskNode } from "~/features/task-node/use-update-task-node";
import { requireUserSession } from "~/session.server";
import { AppControl } from "~/components/app-control/app-control";
import { SessionProvider } from "~/features/auth/use-session";
import { serverFetch } from "~/api/index.server";
import { NodeView } from "~/components/node-view";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await requireUserSession(request);
  const taskNodes = await serverFetch.get("/task-nodes", {
    headers: { cookie: request.headers.get("cookie") },
  });

  return json({ taskNodes, session });
};

const nodeTypes = { task: TaskNode } as const;

export default function TaskNodesPage() {
  const updatePosition = useUpdateTaskNode();

  const { taskNodes, session } = useLoaderData<typeof loader>();
  const [nodes, setNodes] = useState<Node<TaskNodeData>[]>(
    taskNodes.map(({ task, node_info }) => {
      return {
        type: "task",
        id: node_info.id,
        data: { title: task.title, taskId: task.id, status: task.status },
        position: { x: node_info.x, y: node_info.y },
      };
    })
  );
  const [edges, _setEdges, onEdgesChange] = useEdgesState([]);

  const handleAddTaskNode = (node: Node<TaskNodeData>) => {
    setNodes((nodes) => [...nodes, { ...node, type: "task" }]);
  };

  const handleNodesChange: OnNodesChange = (changes) => {
    updatePosition(changes);
    setNodes((nodes) => applyNodeChanges(changes, nodes));
  };

  return (
    <SessionProvider session={session}>
      <div className="h-dvh">
        <NodeView
          nodeTypes={nodeTypes}
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          onEdgesChange={onEdgesChange}
        >
          <Panel position="top-center">
            <AppControl />
          </Panel>
          <Panel position="bottom-center">
            <TaskNodeForm onAddNode={handleAddTaskNode} />
          </Panel>
        </NodeView>
      </div>
    </SessionProvider>
  );
}
