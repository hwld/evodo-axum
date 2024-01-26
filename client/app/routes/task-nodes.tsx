import { LinksFunction, LoaderFunctionArgs, json } from "@remix-run/node";
// eslint-disable-next-line import/no-named-as-default
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Node,
  OnNodesChange,
  Panel,
  applyNodeChanges,
  useEdgesState,
} from "reactflow";
import { api } from "~/api";
import reactFlowStyles from "reactflow/dist/style.css";
import { TaskNodeForm } from "~/features/task-node/task-node-form";
import { useLoaderData } from "@remix-run/react";
import { TaskNode, TaskNodeData } from "~/features/task-node/task-node";
import { useState } from "react";
import { useUpdateTaskNode } from "~/features/task-node/use-update-task-node";
import { AppLogo } from "~/components/app-logo";
import { requireUserSession } from "~/session.server";
import { AppControl } from "~/components/app-control/app-control";
import { SessionProvider } from "~/features/auth/use-session";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: reactFlowStyles },
];

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await requireUserSession(request);
  const taskNodes = await api.get("/task-nodes", {});
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
      <div className="h-[100dvh]">
        <ReactFlow
          nodeOrigin={[0.5, 0.5]}
          nodeTypes={nodeTypes}
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          onEdgesChange={onEdgesChange}
          deleteKeyCode={null}
          disableKeyboardA11y={true}
          fitView
          panActivationKeyCode="none"
          defaultViewport={{ x: 0, y: 0, zoom: 0.5 }}
        >
          <Panel
            position="top-left"
            className="bg-transparent flex items-center gap-1 text-muted-foreground justify-center"
          >
            <AppLogo size={18} />
            <div className="mb-[1px] text-sm">evodo</div>
          </Panel>

          <Panel position="top-center">
            <AppControl />
          </Panel>

          <Panel position="bottom-center">
            <TaskNodeForm onAddNode={handleAddTaskNode} />
          </Panel>

          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>
    </SessionProvider>
  );
}
