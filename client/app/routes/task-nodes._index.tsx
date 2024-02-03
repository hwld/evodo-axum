import { LoaderFunctionArgs, json } from "@remix-run/node";
import {
  Edge,
  Node,
  OnEdgesChange,
  Panel,
  applyEdgeChanges,
  useEdgesState,
} from "reactflow";
import { TaskNodeForm } from "~/features/task-node/task-node-form";
import { useLoaderData } from "@remix-run/react";
import { TaskNode, TaskNodeData } from "~/features/task-node/task-node";
import { useCallback, useState } from "react";
import { useUpdateTaskNode } from "~/features/task-node/use-update-task-node";
import { requireUserSession } from "~/session.server";
import { AppControl } from "~/components/app-control/app-control";
import { SessionProvider } from "~/features/auth/use-session";
import { serverFetch } from "~/api/index.server";
import { NodeView } from "~/components/node-view";
import { useConnectTaskNode } from "~/features/task-node/use-connect-task-node";
import { generateSubtaskEdge } from "~/features/task-node/util";
import { useUpdateTaskNodeEdge } from "~/features/task-node/use-update-task-node-edge";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const session = await requireUserSession(request);
  const taskNodes = await serverFetch.get("/task-nodes", {
    headers: { cookie: request.headers.get("cookie") },
  });

  return json({ taskNodes, session });
};

const nodeTypes = { task: TaskNode } as const;

export default function TaskNodesPage() {
  const { taskNodes, session } = useLoaderData<typeof loader>();

  const [nodes, setNodes] = useState<Node<TaskNodeData>[]>(
    taskNodes.map(({ task, node_info, ancestor_task_ids }) => {
      return {
        type: "task",
        id: task.id,
        data: {
          title: task.title,
          taskId: task.id,
          status: task.status,
          ancestorNodeIds: ancestor_task_ids,
        },
        position: { x: node_info.x, y: node_info.y },
      };
    })
  );

  const [edges, setEdges] = useEdgesState(
    taskNodes
      .map(({ task }): Edge[] => {
        return task.subtask_ids.map((subtaskId): Edge => {
          return generateSubtaskEdge({
            parentTaskId: task.id,
            subtaskId: subtaskId,
          });
        });
      })
      .flat()
  );

  const { handleConnect } = useConnectTaskNode({ setEdges });
  const { handleEdgeUpdateStart, handleEdgeUpdate, handleEdgeUpdateEnd } =
    useUpdateTaskNodeEdge({ setEdges });

  const handleAddTaskNode = useCallback((node: Node<TaskNodeData>) => {
    setNodes((nodes) => [...nodes, { ...node, type: "task" }]);
  }, []);
  const { handleNodesChange } = useUpdateTaskNode({ setNodes });

  const handleEdgesChange: OnEdgesChange = useCallback(
    (changes) => {
      setEdges((old) => applyEdgeChanges(changes, old));
    },
    [setEdges]
  );

  return (
    <SessionProvider session={session}>
      <div className="h-dvh">
        <NodeView
          nodeTypes={nodeTypes}
          nodes={nodes}
          edges={edges}
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
            <TaskNodeForm onAddNode={handleAddTaskNode} />
          </Panel>
        </NodeView>
      </div>
    </SessionProvider>
  );
}
