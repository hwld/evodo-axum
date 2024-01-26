import { makeApi, Zodios, type ZodiosOptions } from "@zodios/core";
import { z } from "zod";

const User = z.object({
  id: z.string(),
  name: z.string(),
  profile: z.string(),
});
const Session = z.object({ user: User });
const SessionResponse = z.object({ session: Session.nullable() }).partial();
const CreateUser = z.object({
  name: z.string().min(1).max(100),
  profile: z.string().max(500),
});
const SignupSessionResponse = z.object({ session_exists: z.boolean() });
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
const CreateTask = z.object({ title: z.string().min(1).max(100) });
const CreateTaskNode = z.object({
  task: CreateTask,
  x: z.number(),
  y: z.number(),
});
const UpdateTask = z.object({ status: TaskStatus, title: z.string() });

export const schemas = {
  User,
  Session,
  SessionResponse,
  CreateUser,
  SignupSessionResponse,
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
    method: "post",
    path: "/cancel-signup",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "get",
    path: "/login",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "get",
    path: "/login-callback",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "post",
    path: "/logout",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "get",
    path: "/session",
    requestFormat: "json",
    response: SessionResponse,
  },
  {
    method: "post",
    path: "/signup",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: CreateUser,
      },
    ],
    response: User,
  },
  {
    method: "get",
    path: "/signup-session",
    requestFormat: "json",
    response: z.object({ session_exists: z.boolean() }),
  },
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
        schema: z.object({ title: z.string().min(1).max(100) }),
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
    ],
    response: Task,
  },
  {
    method: "delete",
    path: "/tasks/:id",
    requestFormat: "json",
    response: Task,
  },
]);

export const _api = new Zodios(endpoints);

export function createApiClient(baseUrl: string, options?: ZodiosOptions) {
  return new Zodios(baseUrl, endpoints, options);
}
