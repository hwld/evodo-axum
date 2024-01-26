import { AlignJustifyIcon } from "lucide-react";
import { Node } from "./ui/node";

type Props = { className?: string };

/**
 * なにもしない装飾用のNode
 */
export const NoopNode: React.FC<Props> = ({ className }) => {
  return (
    <Node size="sm" className={className}>
      <AlignJustifyIcon className="text-muted-foreground" size={18} />
    </Node>
  );
};
