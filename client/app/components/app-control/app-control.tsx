import { useState } from "react";
import { XIcon } from "lucide-react";
import { useClickOutside } from "@mantine/hooks";
import { motion } from "framer-motion";
import { cn } from "~/lib/utils";
import { IdleContent } from "./content/idle";
import { SearchContent } from "./content/search";
import { SettingsContent } from "./content/settings";

export type AppControlMode = "idle" | "search" | "settings";

export const AppControl: React.FC = () => {
  const [mode, setMode] = useState<AppControlMode>("idle");
  const ref = useClickOutside(() => {
    if (mode === "settings") {
      setMode("idle");
    }
  }, ["mouseup", "click", "touchend"]);

  const modeWidth = {
    idle: 150,
    search: 500,
    settings: 200,
  };

  const controlModeClass = {
    idle: "p-2",
    search: "p-2",
    settings: "rounded-xl",
  };

  const content = {
    idle: <IdleContent onChangeMode={setMode} />,
    search: <SearchContent />,
    settings: <SettingsContent width={modeWidth["settings"]} />,
  };

  return (
    <div className="flex gap-1 items-center relative">
      <motion.div
        ref={ref}
        className={cn(
          "bg-primary text-primary-foreground rounded-full shadow flex items-center justify-center",
          controlModeClass[mode]
        )}
        initial={{ width: modeWidth.idle }}
        animate={{ width: modeWidth[mode] }}
      >
        {content[mode]}
      </motion.div>
      <button
        onClick={() => setMode("idle")}
        className={cn(
          "flex bg-primary rounded-full absolute left-[calc(100%+5px)] w-[35px] h-[35px] top-1 text-primary-foreground justify-center items-center transition-all hover:bg-primary/85",
          mode === "idle" ? "opacity-0" : "opacity-100"
        )}
      >
        <XIcon />
      </button>
    </div>
  );
};
