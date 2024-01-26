import { z } from "zod";
import { schemas } from "~/api/schema";

export type Session = z.infer<typeof schemas.Session>;
