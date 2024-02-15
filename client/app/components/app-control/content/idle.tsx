import {
  LockKeyholeIcon,
  MaximizeIcon,
  MinusIcon,
  PlusIcon,
  SearchIcon,
  SettingsIcon,
  UnlockKeyholeIcon,
} from "lucide-react";
import { AppControlItem } from "../app-control-item";
import { AppControlMode } from "../app-control";
import { motion } from "framer-motion";
import {
  ReactFlowState,
  useReactFlow,
  useStore,
  useStoreApi,
} from "@xyflow/react";

type Props = { onChangeMode: (mode: AppControlMode) => void };

const selector = (s: ReactFlowState) => ({
  isInteractive:
    s.nodesDraggable ||
    s.nodesConnectable ||
    s.elementsSelectable ||
    s.edgesUpdatable,
  minZoomReached: s.transform[2] <= s.minZoom,
  maxZoomReached: s.transform[2] >= s.maxZoom,
});

export const IdleContent: React.FC<Props> = ({ onChangeMode }) => {
  const flow = useReactFlow();
  const { maxZoomReached, minZoomReached, isInteractive } = useStore(selector);
  const store = useStoreApi();

  const handleChangeToSearchMode = () => {
    onChangeMode("search");
  };

  const handleZoomIn = () => {
    flow.zoomIn();
  };

  const handleZoomOut = () => {
    flow.zoomOut();
  };

  const handleFitView = () => {
    flow.fitView();
  };

  const handleToggleInteractivity = () => {
    store.setState({
      nodesDraggable: !isInteractive,
      nodesConnectable: !isInteractive,
      elementsSelectable: !isInteractive,
      edgesUpdatable: !isInteractive,
    });
  };

  const handleChangeToSettingsMode = () => {
    onChangeMode("settings");
  };

  return (
    <motion.div layout className="flex p-2">
      <AppControlItem
        position="left"
        tooltip="検索"
        icon={SearchIcon}
        onClick={handleChangeToSearchMode}
      />

      <AppControlItem
        tooltip="ズームイン"
        icon={PlusIcon}
        disabled={maxZoomReached}
        onClick={handleZoomIn}
      />
      <AppControlItem
        tooltip="ズームアウト"
        disabled={minZoomReached}
        icon={MinusIcon}
        onClick={handleZoomOut}
      />
      <AppControlItem
        tooltip="すべてのノードを画面に収める"
        icon={MaximizeIcon}
        onClick={handleFitView}
      />
      <AppControlItem
        tooltip={isInteractive ? "ノードを固定する" : "ノードの固定を解除する"}
        icon={isInteractive ? UnlockKeyholeIcon : LockKeyholeIcon}
        onClick={handleToggleInteractivity}
      />
      <AppControlItem
        tooltip="設定"
        position="right"
        icon={SettingsIcon}
        onClick={handleChangeToSettingsMode}
      />
    </motion.div>
  );
};
