import { z } from "zod";
import { schemas } from "~/api/schema";

export type TaskNode = z.infer<typeof schemas.TaskNode>;
