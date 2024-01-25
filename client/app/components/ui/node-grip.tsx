import { GripVerticalIcon } from "lucide-react";

type Props = { size?: number };
export const NodeGrip: React.FC<Props> = ({ size }) => {
  return (
    <div className="shrink-0">
      <GripVerticalIcon className="text-muted-foreground" size={size} />
    </div>
  );
};
