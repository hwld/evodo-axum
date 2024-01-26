import { z } from "zod";
import { schemas } from "~/api/schema";

export type Task = z.infer<typeof schemas.Task>;
