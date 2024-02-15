import { useState } from "react";
import { XIcon } from "lucide-react";
import { useClickOutside } from "@mantine/hooks";
import { motion } from "framer-motion";
import { cn } from "~/lib/utils";
import { IdleContent } from "./content/idle";
import { SearchContent } from "./content/search";
import { SettingsContent } from "./content/settings";

export type AppControlMode = "idle" | "search" | "settings";

// framer-motionでradiuxを補正するためにstyleで指定する
// https://www.framer.com/motion/layout-animations/##scale-correction
export const controlRadiux = 25;
export const AppControl: React.FC = () => {
  const [mode, setMode] = useState<AppControlMode>("idle");
  const ref = useClickOutside(() => {
    if (mode === "settings") {
      setMode("idle");
    }
  }, ["mouseup", "click", "touchend"]);

  const content = {
    idle: <IdleContent onChangeMode={setMode} />,
    search: <SearchContent />,
    settings: <SettingsContent />,
  };

  return (
    <div className="flex gap-1 items-center relative">
      <motion.div
        layout
        ref={ref}
        className={cn(
          "bg-primary text-primary-foreground shadow flex items-center justify-center overflow-hidden"
        )}
        style={{ borderRadius: controlRadiux }}
        transition={{ type: "spring", damping: 27, stiffness: 300 }}
      >
        {content[mode]}
      </motion.div>
      <button
        onClick={() => setMode("idle")}
        className={cn(
          "flex bg-primary rounded-full absolute left-[calc(100%+5px)] size-[36px] top-1 text-primary-foreground justify-center items-center transition-all hover:bg-primary/85",
          mode === "idle" ? "opacity-0" : "opacity-100"
        )}
      >
        <XIcon />
      </button>
    </div>
  );
};
