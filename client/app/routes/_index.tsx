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

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: reactFlowStyles },
];

export const loader = async () => {
  const taskNodes = await api.get("/task-nodes", {});
  return json({ taskNodes });
};

export default function Index() {
  const { taskNodes } = useLoaderData<typeof loader>();
  const [nodes, setNodes, onNodesChange] = useNodesState(
    taskNodes.map(({ task, node_info }) => {
      return {
        id: node_info.id,
        data: { label: task.title },
        position: { x: node_info.x, y: node_info.y },
      };
    })
  );
  const [edges, _setEdges, onEdgesChange] = useEdgesState([]);

  const handleAddNode = (node: Node) => {
    setNodes((nodes) => [...nodes, node]);
  };

  return (
    <div className="h-[100dvh]">
      <ReactFlow
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
          <TaskNodeForm onAddNode={handleAddNode} />
        </Panel>
      </ReactFlow>
    </div>
  );
}
