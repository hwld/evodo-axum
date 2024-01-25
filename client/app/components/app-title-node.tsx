import { AppLogo } from "./app-logo";
import { Node } from "./ui/node";

export const AppTitleNode = () => {
  return (
    <Node className="w-[300px]">
      <div className="flex items-center gap-2">
        <AppLogo size={40} />
        <p className="text-2xl font-bold">evodo-axum</p>
      </div>
    </Node>
  );
};
