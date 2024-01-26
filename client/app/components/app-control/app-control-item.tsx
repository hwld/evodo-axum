import { LucideIcon } from "lucide-react";
import { cn } from "~/lib/utils";

type Props = {
  icon: LucideIcon;
  position: "left" | "center" | "right";
  onClick: () => void;
};

export const AppControlItem: React.FC<Props> = ({
  icon: Icon,
  position,
  onClick,
}) => {
  const positionClass = {
    left: "rounded-l-full",
    center: "rounded-sm",
    right: "rounded-r-full",
  };

  return (
    <button
      onClick={onClick}
      className={cn(
        "grow flex justify-center hover:bg-primary-foreground/10 h-full transition-colors py-1 px-3 cursor-pointer",
        positionClass[position]
      )}
    >
      <Icon size={20} />
    </button>
  );
};
