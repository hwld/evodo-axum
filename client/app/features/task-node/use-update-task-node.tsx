import { useMutation } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { NodeChange, applyNodeChanges } from "@xyflow/react";
import { api } from "~/api/index.client";
import { toast } from "sonner";
import { useTaskNodeViewAction } from "./task-node-view-provider";

export const useUpdateTaskNode = () => {
  const { setTaskNodes } = useTaskNodeViewAction();

  const mutation = useMutation({
    mutationFn: ({ id, x, y }: { id: string; x: number; y: number }) => {
      return api.put("/task-node-info/:id", { x: x, y: y }, { params: { id } });
    },
    onError: (err) => {
      console.error(err);
      toast.error("ノードの位置を更新することができませんでした。");
    },
  });

  const timerMapRef = useRef(new Map<string, number>());
  const debounceCall = useCallback((key: string, callback: () => void) => {
    const timer = timerMapRef.current.get(key);
    if (timer) {
      window.clearTimeout(timer);
    }

    const newTimer = window.setTimeout(callback, 200);
    timerMapRef.current.set(key, newTimer);
  }, []);

  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      changes.forEach((change) => {
        if (change.type !== "position" || !change.position) {
          return;
        }

        const position = change.position;
        debounceCall(change.id, () => {
          mutation.mutate({
            id: change.id,
            x: position.x,
            y: position.y,
          });
        });
      });

      setTaskNodes((nodes) => applyNodeChanges(changes, nodes));
    },
    [debounceCall, mutation, setTaskNodes]
  );

  return { handleNodesChange };
};
