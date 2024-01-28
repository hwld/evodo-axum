import { createFetch } from ".";

export const serverFetch = createFetch(process.env.BACKEND_URL || "");
