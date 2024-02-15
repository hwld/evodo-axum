import { PanelRightOpenIcon, SearchIcon, SettingsIcon } from "lucide-react";
import { AppControlItem } from "../app-control-item";
import { AppControlMode } from "../app-control";
import { motion } from "framer-motion";

type Props = { onChangeMode: (mode: AppControlMode) => void };

export const IdleContent: React.FC<Props> = ({ onChangeMode }) => {
  return (
    <motion.div layout className="flex p-2">
      <AppControlItem
        position="left"
        icon={SearchIcon}
        onClick={() => onChangeMode("search")}
      />
      <AppControlItem
        position="center"
        icon={SettingsIcon}
        onClick={() => onChangeMode("settings")}
      />
      <AppControlItem
        position="right"
        icon={PanelRightOpenIcon}
        onClick={() => {}}
      />
    </motion.div>
  );
};
