import { LinksFunction } from "@remix-run/node";
// eslint-disable-next-line import/no-named-as-default
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Node,
  useNodesState,
} from "reactflow";
import reactFlowStyles from "reactflow/dist/style.css";
import { AppDescriptionNode } from "~/components/app-description-node";
import { AppTitleNode } from "~/components/app-title-node";
import { NoopNode } from "~/components/noop-node";
import { LoginButtonNode } from "~/features/auth/login-button-node";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: reactFlowStyles },
];

const nodeTypes = {
  noop: () => <NoopNode />,
  logo: () => <AppTitleNode />,
  loginButton: () => <LoginButtonNode />,
  description: () => <AppDescriptionNode />,
} as const;

const initialNodes: Node[] = [
  { type: "noop", data: {}, id: "d1", position: { x: 0, y: -380 } },
  { type: "noop", data: {}, id: "d2", position: { x: 0, y: 380 } },
  { type: "logo", data: {}, id: "1", position: { x: 0, y: -200 } },
  { type: "description", data: {}, id: "3", position: { x: 0, y: 0 } },
  { type: "loginButton", data: {}, id: "2", position: { x: 0, y: 200 } },
];

export default function Login() {
  const [nodes, _, onNodesChange] = useNodesState(initialNodes);

  return (
    <div className="h-[100dvh]">
      <ReactFlow
        nodeOrigin={[0.5, 0.5]}
        nodeTypes={nodeTypes}
        nodes={nodes}
        onNodesChange={onNodesChange}
        fitView
        deleteKeyCode={null}
      >
        <Background />
        <MiniMap />
        <Controls />
      </ReactFlow>
    </div>
  );
}
