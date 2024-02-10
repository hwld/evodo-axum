import { Edge, Node, ReactFlowProvider } from "@xyflow/react";
import { TaskNodeViewData } from "~/features/task-node/task-node";
import {
  ReactNode,
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  TaskNodeData,
  buildTaskNodeEdges,
  buildTaskNodes,
} from "~/features/task-node/util";

// useReactFlowというhookもあるのだが、その中のsetNodesやsetEdgesは非同期で実行されるため、
// ちらつきが生じてしまう。
// これを解決するために、NodeとEdgeと直接それらを変更する関数を公開する。

type TaskNodeViewContext = {
  taskNodes: Node<TaskNodeViewData>[];
  taskNodeEdges: Edge[];
};
type TsakNodeViewActionContext = {
  setTaskNodes: React.Dispatch<React.SetStateAction<Node<TaskNodeViewData>[]>>;
  setTaskNodeEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
};
const TaskNodeViewContext = createContext<TaskNodeViewContext | undefined>(
  undefined
);
const TaskNodeViewActionContext = createContext<
  TsakNodeViewActionContext | undefined
>(undefined);

export const TaskNodeViewProvider: React.FC<{
  taskNodeData: TaskNodeData[];
  children: ReactNode;
}> = ({ taskNodeData, children }) => {
  const [taskNodes, setTaskNodes] = useState<Node<TaskNodeViewData>[]>(
    buildTaskNodes(taskNodeData)
  );
  const [taskNodeEdges, setTaskNodeEdges] = useState<Edge[]>(
    buildTaskNodeEdges(taskNodeData)
  );

  const data: TaskNodeViewContext = useMemo(() => {
    return { taskNodes, taskNodeEdges };
  }, [taskNodeEdges, taskNodes]);

  const action: TsakNodeViewActionContext = useMemo(() => {
    return { setTaskNodes, setTaskNodeEdges };
  }, []);

  useEffect(() => {
    setTaskNodes(buildTaskNodes(taskNodeData));
    setTaskNodeEdges(buildTaskNodeEdges(taskNodeData));
  }, [setTaskNodeEdges, setTaskNodes, taskNodeData]);

  return (
    <ReactFlowProvider>
      <TaskNodeViewContext.Provider value={data}>
        <TaskNodeViewActionContext.Provider value={action}>
          {children}
        </TaskNodeViewActionContext.Provider>
      </TaskNodeViewContext.Provider>
    </ReactFlowProvider>
  );
};

export const useTaskNodeView = (): TaskNodeViewContext => {
  const context = useContext(TaskNodeViewContext);
  if (!context) {
    throw new Error("TaskNodeViewProviderが必要です");
  }
  return context;
};

export const useTaskNodeViewAction = (): TsakNodeViewActionContext => {
  const context = useContext(TaskNodeViewActionContext);
  if (!context) {
    throw new Error("TaskNodeViewProviderが必要です");
  }
  return context;
};
