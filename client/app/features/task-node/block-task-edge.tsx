import { BaseEdge, EdgeProps, getBezierPath, useStore } from "@xyflow/react";
import { cn } from "~/lib/utils";
import { useCallback } from "react";

export const BlockTaskEdge: React.FC<EdgeProps> = ({
  id,
  selected,
  source,
  ...props
}) => {
  const [edgePath] = getBezierPath({
    ...props,
  });

  const blockingNode = useStore(
    useCallback(
      (store) => {
        return store.nodeLookup.get(source);
      },
      [source]
    )
  );

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        className={cn(
          "!stroke-[1.5px] ",
          blockingNode?.data.status === "Done"
            ? "!stroke-green-500"
            : "!stroke-red-400",
          selected && "!stroke-[3px]"
        )}
        style={{ strokeDasharray: 5 }}
      />
    </>
  );
};
