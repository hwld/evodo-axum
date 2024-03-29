import { AppLogo } from "./app-logo";
import { Node } from "./ui/node";

export const AppTitleNode = () => {
  return (
    <Node className="w-[400px]">
      <div className="flex items-center gap-2">
        <AppLogo size={45} />
        <p className="text-2xl font-bold">evodo-axum</p>
      </div>
    </Node>
  );
};
