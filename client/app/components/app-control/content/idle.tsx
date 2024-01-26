import { PanelRightOpenIcon, SearchIcon, SettingsIcon } from "lucide-react";
import { AppControlItem } from "../app-control-item";
import { AppControlMode } from "../app-control";

type Props = { onChangeMode: (mode: AppControlMode) => void };

export const IdleContent: React.FC<Props> = ({ onChangeMode }) => {
  return (
    <div className="flex w-full">
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
    </div>
  );
};
