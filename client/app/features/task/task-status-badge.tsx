import clsx from "clsx";
import { z } from "zod";
import { schemas } from "~/api/schema";

type Props = { status: z.infer<typeof schemas.Task>["status"] };

export const TaskStatusBadge: React.FC<Props> = ({ status }) => {
  const labelMap = { Todo: "未完了", Done: "完了" };
  const badgeClass = {
    Todo: "border-red-500 text-red-500 bg-red-50",
    Done: "border-green-500 text-green-500 bg-green-50",
  };
  return (
    <div
      className={clsx(
        "border w-[70px] py-1 rounded-full text-sm  font-bold text flex justify-center items-center",
        badgeClass[status]
      )}
    >
      {labelMap[status]}
    </div>
  );
};
