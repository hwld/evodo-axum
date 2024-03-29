import {
  ReactFlow,
  Background,
  MiniMap,
  Panel,
  ReactFlowProps,
} from "@xyflow/react";
import { AppLogo } from "./app-logo";

type Props = ReactFlowProps;
export const NodeView: React.FC<Props> = ({ children, ...props }) => {
  return (
    <ReactFlow
      nodeOrigin={[0.5, 0.5]}
      deleteKeyCode={null}
      disableKeyboardA11y={true}
      fitView
      panActivationKeyCode="none"
      defaultViewport={{ x: 0, y: 0, zoom: 0.5 }}
      proOptions={{ hideAttribution: true }}
      {...props}
    >
      {children}

      <Panel
        position="top-left"
        className="bg-transparent flex items-center gap-1 text-muted-foreground justify-center"
      >
        <AppLogo size={18} />
        <div className="mb-[1px] text-sm">evodo</div>
      </Panel>

      <Background />
      <MiniMap position="bottom-left" />
    </ReactFlow>
  );
};
