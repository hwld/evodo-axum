import { LinksFunction } from "@remix-run/node";
import { Link } from "@remix-run/react";
import { AlertTriangleIcon } from "lucide-react";
// eslint-disable-next-line import/no-named-as-default
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Node,
  useNodesState,
} from "reactflow";
import reactFlowStyles from "reactflow/dist/style.css";
import { Button } from "~/components/ui/button";
import { Node as NodeComponent } from "~/components/ui/node";
import { NoopNode } from "~/components/ui/noop-node";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: reactFlowStyles },
];

const nodeTypes = {
  noop: () => <NoopNode />,
  alert: () => {
    return (
      <NodeComponent className="w-[300px] h-[300px] text-destructive">
        <AlertTriangleIcon size={200} />
      </NodeComponent>
    );
  },
  toLoginPageLink: function ToLoginPageLink() {
    return (
      <NodeComponent className="w-[400px]">
        <Button asChild size="sm">
          <Link to="/login" replace>
            ログインページへ戻る
          </Link>
        </Button>
      </NodeComponent>
    );
  },
  description: () => {
    return (
      <NodeComponent className="w-[400px] h-[200px]">
        <div className="flex flex-col items-center gap-2">
          <div className="flex gap-2 items-center">
            <AlertTriangleIcon />
            <div className="font-bold text-xl">Error</div>
            <AlertTriangleIcon />
          </div>
          <div className="text-sm text-muted-foreground">
            ログイン処理でエラーが発生しました。
            <br />
            もう一度ログインを試してみてください。
          </div>
        </div>
      </NodeComponent>
    );
  },
} as const;

const initialNodes: Node[] = [
  { type: "noop", data: {}, id: "d1", position: { x: 0, y: -266 } },
  { type: "noop", data: {}, id: "d2", position: { x: 0, y: 350 } },
  { type: "description", data: {}, id: "3", position: { x: 0, y: 0 } },
  { type: "toLoginPageLink", data: {}, id: "2", position: { x: 0, y: 146 } },
];

export default function AuthErrorPage() {
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
