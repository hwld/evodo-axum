import { Node, useNodesState } from "reactflow";
import { AppDescriptionNode } from "~/components/app-description-node";
import { AppTitleNode } from "~/components/app-title-node";
import { NodeView } from "~/components/node-view";
import { NoopNode } from "~/components/noop-node";
import { LoginButtonNode } from "~/features/auth/login-button-node";

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
    <div className="h-dvh">
      <NodeView
        nodeTypes={nodeTypes}
        nodes={nodes}
        onNodesChange={onNodesChange}
      />
    </div>
  );
}
