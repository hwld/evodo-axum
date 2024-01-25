import { ComponentProps, ReactNode } from "react";
import { Card } from "./card";
import { NodeGrip } from "./node-grip";
import { cn } from "~/lib/utils";

type Props = {
  children?: ReactNode;
  className?: string;
  style?: ComponentProps<"div">["style"];
};
export const Node: React.FC<Props> = ({ children, className, style }) => {
  return (
    <Card
      className={cn("flex items-center p-3 gap-2 justify-between", className)}
      style={style}
    >
      <NodeGrip />
      {children}
      <NodeGrip />
    </Card>
  );
};
