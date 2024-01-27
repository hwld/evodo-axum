import { Link } from "@remix-run/react";
import { Button } from "~/components/ui/button";
import { Node } from "~/components/ui/node";

export const LoginButtonNode = () => {
  return (
    <Node className="w-[400px]">
      <Button asChild>
        <Link to="http://localhost:8787/auth/login">Googleでログインする</Link>
      </Button>
    </Node>
  );
};
