import { Node } from "./ui/node";

export const AppDescriptionNode: React.FC = () => {
  return (
    <Node className="h-[300px] w-[400px]">
      <div className="text-muted-foreground p-1 text-sm">
        <p>
          evodo-axumは、 RustのAxumとReact Flowを試すために作った
          <a
            href="https://github.com/hwld/evodo"
            className="underline underline-offset-2 text-accent-foreground mx-1"
          >
            evodoプロジェクト
          </a>
          です。
          <br />
          <br />
          <br />
          ノードエディタでタスクを管理することで、分解されたタスクやブロックしている・されているタスクをわかりやすく管理できるかもしれないと思い、React
          Flowを使用しています。
        </p>
      </div>
    </Node>
  );
};
