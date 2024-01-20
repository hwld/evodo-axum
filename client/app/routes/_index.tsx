import { LinksFunction, json } from "@remix-run/node";
// eslint-disable-next-line import/no-named-as-default
import ReactFlow, {
  Background,
  Controls,
  Node,
  Panel,
  useEdgesState,
  useNodesState,
} from "reactflow";
import { api } from "~/api";
import reactFlowStyles from "reactflow/dist/style.css";
import { TaskNodeForm } from "~/features/task/task-node-form";
import { useLoaderData } from "@remix-run/react";
import { TaskNode, TaskNodeData } from "~/features/task/task-node";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: reactFlowStyles },
];

export const loader = async () => {
  const taskNodes = await api.get("/task-nodes", {});
  return json({ taskNodes });
};

const nodeTypes = { task: TaskNode } as const;

export default function Index() {
  const { taskNodes } = useLoaderData<typeof loader>();
  const [nodes, setNodes, onNodesChange] = useNodesState<TaskNodeData>(
    taskNodes.map(({ task, node_info }) => {
      return {
        type: "task",
        id: node_info.id,
        data: { title: task.title, taskId: task.id },
        position: { x: node_info.x, y: node_info.y },
      };
    })
  );
  const [edges, _setEdges, onEdgesChange] = useEdgesState([]);

  const handleAddTaskNode = (node: Node<TaskNodeData>) => {
    setNodes((nodes) => [...nodes, { ...node, type: "task" }]);
  };

  return (
    <div className="h-[100dvh]">
      <ReactFlow
        nodeTypes={nodeTypes}
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        deleteKeyCode={null}
        fitView
      >
        <Background />
        <Controls />
        <Panel position="bottom-center">
          <TaskNodeForm onAddNode={handleAddTaskNode} />
        </Panel>
      </ReactFlow>
    </div>
  );
}
