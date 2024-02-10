import { OnEdgesChange, Panel, applyEdgeChanges } from "@xyflow/react";
import { TaskNodeForm } from "~/features/task-node/task-node-form";
import { useCallback } from "react";
import { useUpdateTaskNode } from "~/features/task-node/use-update-task-node";
import { AppControl } from "~/components/app-control/app-control";
import { NodeView } from "~/components/node-view";
import { useConnectTaskNode } from "~/features/task-node/use-connect-task-node/use-connect-task-node";
import { nodeTypes } from "~/features/task-node/util";
import { useUpdateTaskNodeEdge } from "~/features/task-node/use-update-task-node-edge/use-update-task-node-edge";
import {
  useTaskNodeView,
  useTaskNodeViewAction,
} from "./task-node-view-provider";

export const TaskNodeView: React.FC = () => {
  const { taskNodes, taskNodeEdges } = useTaskNodeView();
  const { setTaskNodeEdges } = useTaskNodeViewAction();

  const { handleConnect } = useConnectTaskNode();

  const { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd } =
    useUpdateTaskNodeEdge();

  const { handleNodesChange } = useUpdateTaskNode();

  const handleEdgesChange: OnEdgesChange = useCallback(
    (changes) => {
      setTaskNodeEdges((old) => applyEdgeChanges(changes, old));
    },
    [setTaskNodeEdges]
  );

  return (
    <NodeView
      nodeTypes={nodeTypes}
      nodes={taskNodes}
      edges={taskNodeEdges}
      onNodesChange={handleNodesChange}
      onEdgesChange={handleEdgesChange}
      onEdgeUpdateStart={handleEdgeUpdateStart}
      onEdgeUpdate={handleEdgeUpdate}
      onEdgeUpdateEnd={handleEdgeUpdateEnd}
      onConnect={handleConnect}
    >
      <Panel position="top-center">
        <AppControl />
      </Panel>
      <Panel position="bottom-center">
        <TaskNodeForm />
      </Panel>
    </NodeView>
  );
};
