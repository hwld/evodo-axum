import { createApiClient } from "./schema";

export const createFetch = (url: string) =>
  createApiClient(url, {
    axiosConfig: { withCredentials: true },
  });
