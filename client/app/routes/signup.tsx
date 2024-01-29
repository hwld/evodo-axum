import { LoaderFunctionArgs, json } from "@remix-run/node";
import { Node, useNodesState } from "reactflow";
import { SignupFormNode } from "~/features/auth/signup-form-node";
import { Node as NodeComponent } from "~/components/ui/node";
import { UserPlusIcon } from "lucide-react";
import { requireSignupSession } from "~/session.server";
import { NoopNode } from "~/components/noop-node";
import { AppLogo } from "~/components/app-logo";
import { NodeView } from "~/components/node-view";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  await requireSignupSession(request);
  return json({});
};

const nodeTypes = {
  noop: () => <NoopNode />,
  logo: () => (
    <NodeComponent className="w-[400px] h-[530px]">
      <div className="flex flex-col items-center gap-3">
        <AppLogo size={150} />
        <div className="text-3xl font-bold">evodo-axum</div>
      </div>
    </NodeComponent>
  ),
  title: () => (
    <NodeComponent className="w-[400px]">
      <div className="space-y-2">
        <div className="flex items-center gap-1">
          <UserPlusIcon />
          <h1 className="font-bold">ユーザーを登録する</h1>
        </div>
        <div className="text-xs text-muted-foreground">
          ユーザー名とプロフィールを入力して、ユーザーを登録することができます。
        </div>
      </div>
    </NodeComponent>
  ),
  form: () => <SignupFormNode />,
} as const;

const initialNodes: Node[] = [
  { type: "noop", data: {}, id: "d1", position: { x: 211, y: -441 } },
  { type: "noop", data: {}, id: "d2", position: { x: 211, y: 296 } },
  { type: "logo", data: {}, id: "l1", position: { x: 420, y: -52 } },
  { type: "title", data: {}, id: "t1", position: { x: 0, y: -270 } },
  { type: "form", data: {}, id: "f1", position: { x: 0, y: 0 } },
];

export default function SignupPage() {
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
