import { Node } from "./ui/node";

export const AppDescriptionNode: React.FC = () => {
  return (
    <Node className="h-[300px] w-[300px]">
      <div className="text-muted-foreground p-1 text-sm">
        <p>
          evodo-axumは、 RustのAxumを試すために作ったevodoプロジェクトです。
          <br />
          <br />
          React
          Flowを使って、タスク同士の関係を視覚的に表現できれば面白いなぁと思っています。
        </p>
      </div>
    </Node>
  );
};
