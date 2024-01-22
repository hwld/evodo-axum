import { LinksFunction, json } from "@remix-run/node";
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

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: reactFlowStyles },
];

export const loader = async () => {
  const taskNodes = await api.get("/task-nodes", {});
  return json({ taskNodes });
};

const nodeTypes = { task: TaskNode } as const;

export default function Index() {
  const updatePosition = useUpdateTaskNode();

  const { taskNodes } = useLoaderData<typeof loader>();
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
    <div className="h-[100dvh]">
      <ReactFlow
        nodeOrigin={[0.5, 0.5]}
        nodeTypes={nodeTypes}
        nodes={nodes}
        edges={edges}
        onNodesChange={handleNodesChange}
        onEdgesChange={onEdgesChange}
        deleteKeyCode={null}
        fitView
      >
        <Background />
        <Controls />
        <MiniMap />
        <Panel position="bottom-center">
          <TaskNodeForm onAddNode={handleAddTaskNode} />
        </Panel>
      </ReactFlow>
    </div>
  );
}
