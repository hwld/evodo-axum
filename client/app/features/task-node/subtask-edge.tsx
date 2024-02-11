import { BaseEdge, EdgeProps, getBezierPath, useStore } from "@xyflow/react";
import { useCallback } from "react";
import { cn } from "~/lib/utils";

export const SubtaskEdge: React.FC<EdgeProps> = ({
  id,
  selected,
  source,
  target,
  ...props
}) => {
  const mainTaskNode = useStore(
    useCallback(
      (store) => {
        return store.nodeLookup.get(source);
      },
      [source]
    )
  );
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
          "!stroke-[1.5px] !stroke-neutral-300",
          {
            "!stroke-green-500": subTaskNode?.data?.status === "Done",
            "!stroke-red-500": mainTaskNode?.data?.isBlocked,
          },
          selected && "!stroke-[3px]"
        )}
      />
    </>
  );
};
