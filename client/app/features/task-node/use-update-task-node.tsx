import { useMutation } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { NodeChange } from "reactflow";
import { api } from "~/api/index.client";

export const useUpdateTaskNode = () => {
  const mutation = useMutation({
    mutationFn: ({ id, x, y }: { id: string; x: number; y: number }) => {
      return api.put("/task-node-info/:id", { x: x, y: y }, { params: { id } });
    },
    onError: (err) => {
      console.error(err);
      window.alert("ノードの位置を更新することができませんでした");
    },
  });

  const nodeTimerMapRef = useRef(new Map<string, number>());
  const updatePosition = useCallback(
    (changes: NodeChange[]) => {
      changes.forEach((change) => {
        if (change.type !== "position" || !change.position) {
          return;
        }

        const timer = nodeTimerMapRef.current.get(change.id);
        if (timer) {
          window.clearTimeout(timer);
        }

        const newTimer = window.setTimeout(() => {
          if (!change.position) {
            return;
          }
          mutation.mutate({
            id: change.id,
            x: change.position.x,
            y: change.position.y,
          });
        }, 200);

        nodeTimerMapRef.current.set(change.id, newTimer);
      });
    },
    [mutation]
  );

  return updatePosition;
};
