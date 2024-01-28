import { createFetch } from ".";

export const api = createFetch(window.ENV.BACKEND_URL || "");
