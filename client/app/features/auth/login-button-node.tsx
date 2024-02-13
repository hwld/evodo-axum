import { Button } from "~/components/ui/button";
import { Node } from "~/components/ui/node";

export const LoginButtonNode = () => {
  const handleLogin = () => {
    // SSR環境ではwindowを参照できないのでaタグではなくbuttonで遷移させる
    window.location.href = `${window.ENV.BACKEND_URL}/auth/login`;
  };

  return (
    <Node className="w-[400px]">
      <Button onClick={handleLogin}>Googleでログインする</Button>
    </Node>
  );
};
