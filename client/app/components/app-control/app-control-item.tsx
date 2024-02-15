import { LucideIcon } from "lucide-react";
import { cn } from "~/lib/utils";
import { Tooltip, TooltipProvider, TooltipTrigger } from "../ui/tooltip";
import { TooltipContent, TooltipPortal } from "@radix-ui/react-tooltip";

type Props = {
  icon: LucideIcon;
  position?: "left" | "center" | "right";
  onClick: () => void;
  disabled?: boolean;
  tooltip: string;
};

export const AppControlItem: React.FC<Props> = ({
  icon: Icon,
  position = "center",
  onClick,
  disabled,
  tooltip,
}) => {
  const positionClass = {
    left: "rounded-l-full pr-2 pl-3",
    center: "rounded-sm px-2",
    right: "rounded-r-full pl-2 pr-3",
  };

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger>
          <button
            onClick={onClick}
            className={cn(
              "grow flex justify-center hover:bg-primary-foreground/10 h-full transition-colors py-1 cursor-pointer",
              positionClass[position],
              "disabled:cursor-not-allowed disabled:text-neutral-500 disabled:hover:bg-transparent"
            )}
            disabled={disabled}
          >
            <Icon size={20} />
          </button>
        </TooltipTrigger>
        <TooltipPortal>
          <TooltipContent
            sideOffset={10}
            className="text-neutral-100 bg-neutral-900 px-2 py-1 rounded text-xs animate-in slide-in-from-top-1 data-[state=closed]:animate-out data-[state=closed]:slide-out-to-top-2 data-[state=closed]:fade-out-20"
          >
            <p>{tooltip}</p>
          </TooltipContent>
        </TooltipPortal>
      </Tooltip>
    </TooltipProvider>
  );
};
