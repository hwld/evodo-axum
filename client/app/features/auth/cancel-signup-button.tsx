import { useNavigate } from "@remix-run/react";
import { useMutation } from "@tanstack/react-query";
import { api } from "~/api/index.client";
import { Button } from "~/components/ui/button";

export const CancelSignupButton: React.FC = () => {
  const navigate = useNavigate();
  const cancelMutation = useMutation({
    mutationFn: () => {
      return api.post("/auth/cancel-signup", undefined);
    },
    onError: (err) => {
      console.error(err);
      window.alert("登録をキャンセルできませんでした。");
    },
    onSuccess: () => {
      navigate("/", { replace: true });
    },
  });

  const handleCancel = async () => {
    cancelMutation.mutate();
  };

  return (
    <Button
      type="button"
      variant="link"
      size="sm"
      onClick={handleCancel}
      disabled={cancelMutation.isPaused}
    >
      登録をやめる
    </Button>
  );
};
