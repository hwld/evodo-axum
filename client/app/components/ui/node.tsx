import { ComponentProps, ReactNode } from "react";
import { Card } from "./card";
import { NodeGrip } from "./node-grip";
import { cn } from "~/lib/utils";

type Props = {
  children?: ReactNode;
  className?: string;
  style?: ComponentProps<"div">["style"];
  size?: "md" | "sm";
};
export const Node: React.FC<Props> = ({
  children,
  className,
  style,
  size = "md",
}) => {
  return (
    <Card
      className={cn(
        "flex items-center justify-between",
        size === "md" && "p-3 gap-2",
        size === "sm" && "p-2 gap-1",
        className
      )}
      style={style}
    >
      <NodeGrip size={size === "sm" ? 18 : undefined} />
      {children}
      <NodeGrip size={size === "sm" ? 18 : undefined} />
    </Card>
  );
};
