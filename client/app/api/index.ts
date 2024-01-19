import { createApiClient } from "./schema";

export const api = createApiClient(`http://localhost:8787`, {
  axiosConfig: { withCredentials: true },
});
