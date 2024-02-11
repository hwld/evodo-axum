import { BaseEdge, EdgeProps, getBezierPath, useStore } from "@xyflow/react";
import { useCallback } from "react";
import { cn } from "~/lib/utils";

export const SubtaskEdge: React.FC<EdgeProps> = ({
  id,
  selected,
  target,
  ...props
}) => {
  const subTaskNode = useStore(
    useCallback(
      (store) => {
        return store.nodeLookup.get(target);
      },
      [target]
    )
  );
  const [edgePath] = getBezierPath({
    ...props,
  });

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        className={cn(
          "!stroke-[1.5px]",
          subTaskNode?.data.status === "Done"
            ? "!stroke-green-500"
            : "!stroke-neutral-300",
          selected && "!stroke-[3px]"
        )}
      />
    </>
  );
};
