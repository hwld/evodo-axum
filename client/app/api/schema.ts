import { makeApi, Zodios, type ZodiosOptions } from "@zodios/core";
import { z } from "zod";

const UpdateTaskNodeInfo = z.object({ x: z.number(), y: z.number() });
const TaskNodeInfo = z.object({
  id: z.string(),
  task_id: z.string(),
  x: z.number(),
  y: z.number(),
});
const TaskStatus = z.enum(["Todo", "Done"]);
const Task = z.object({
  created_at: z.string(),
  id: z.string(),
  status: TaskStatus,
  title: z.string(),
  updated_at: z.string(),
});
const TaskNode = z.object({ node_info: TaskNodeInfo, task: Task });
const CreateTask = z.object({ title: z.string().min(1) });
const CreateTaskNode = z.object({
  task: CreateTask,
  x: z.number(),
  y: z.number(),
});
const UpdateTask = z.object({ status: TaskStatus, title: z.string() });

export const schemas = {
  UpdateTaskNodeInfo,
  TaskNodeInfo,
  TaskStatus,
  Task,
  TaskNode,
  CreateTask,
  CreateTaskNode,
  UpdateTask,
};

const endpoints = makeApi([
  {
    method: "put",
    path: "/task-node-info/:id",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: UpdateTaskNodeInfo,
      },
      {
        name: "id",
        type: "Path",
        schema: z.string(),
      },
    ],
    response: TaskNodeInfo,
  },
  {
    method: "get",
    path: "/task-nodes",
    requestFormat: "json",
    response: z.array(TaskNode),
  },
  {
    method: "post",
    path: "/task-nodes",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: CreateTaskNode,
      },
    ],
    response: TaskNode,
  },
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
        schema: z.object({ title: z.string().min(1) }),
      },
    ],
    response: Task,
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
