import { makeApi, Zodios, type ZodiosOptions } from "@zodios/core";
import { z } from "zod";

const TaskStatus = z.enum(["Todo", "Done"]);
const Task = z.object({
  created_at: z.string(),
  id: z.string(),
  status: TaskStatus,
  title: z.string(),
  updated_at: z.string(),
});
const CreateTask = z.object({ title: z.string() });
const UpdateTask = z.object({ status: TaskStatus, title: z.string() });

export const schemas = {
  TaskStatus,
  Task,
  CreateTask,
  UpdateTask,
};

const endpoints = makeApi([
  {
    method: "get",
    path: "/tasks",
    requestFormat: "json",
    response: z.array(Task),
  },
  {
    method: "post",
    path: "/tasks",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: z.object({ title: z.string() }),
      },
    ],
    response: z.void(),
  },
  {
    method: "put",
    path: "/tasks/:id",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: UpdateTask,
      },
      {
        name: "id",
        type: "Path",
        schema: z.string(),
      },
    ],
    response: Task,
  },
  {
    method: "delete",
    path: "/tasks/:id",
    requestFormat: "json",
    parameters: [
      {
        name: "id",
        type: "Path",
        schema: z.string(),
      },
    ],
    response: Task,
  },
]);

export const _api = new Zodios(endpoints);

export function createApiClient(baseUrl: string, options?: ZodiosOptions) {
  return new Zodios(baseUrl, endpoints, options);
}
