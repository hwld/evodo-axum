import { Button } from "~/components/ui/button";
import { Node } from "~/components/ui/node";

export const LoginButtonNode = () => {
  return (
    <Node className="w-[400px]">
      <Button asChild>
        <a href={`${window.ENV.BACKEND_URL}/auth/login`}>
          Googleでログインする
        </a>
      </Button>
    </Node>
  );
};
