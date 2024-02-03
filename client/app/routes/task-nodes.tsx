import { Outlet } from "@remix-run/react";
import { ReactFlowProvider } from "reactflow";

export default function TaskNodesPage() {
  return (
    <ReactFlowProvider>
      <Outlet />
    </ReactFlowProvider>
  );
}
