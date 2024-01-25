import { createApiClient } from "./schema";

// TODO
export const api = createApiClient(`http://localhost:8787`, {
  axiosConfig: { withCredentials: true },
});
