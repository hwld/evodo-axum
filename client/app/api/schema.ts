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
const ConnectSubtask = z.object({
  parent_task_id: z.string(),
  subtask_id: z.string(),
});
const DisconnectSubtask = z.object({
  parent_task_id: z.string(),
  subtask_id: z.string(),
});
const ReconnectSubtask = z.object({
  new_parent_task_id: z.string(),
  new_subtask_id: z.string(),
  old_parent_task_id: z.string(),
  old_subtask_id: z.string(),
});
const UpdateTaskNodeInfo = z.object({ x: z.number(), y: z.number() });
const TaskNodeInfo = z.object({
  id: z.string(),
  task_id: z.string(),
  user_id: z.string(),
  x: z.number(),
  y: z.number(),
});
const TaskStatus = z.enum(["Todo", "Done"]);
const Task = z.object({
  created_at: z.string(),
  id: z.string(),
  status: TaskStatus,
  subtask_ids: z.array(z.string()),
  title: z.string(),
  updated_at: z.string(),
  user_id: z.string(),
});
const TaskNode = z.object({ node_info: TaskNodeInfo, task: Task });
const CreateTask = z.object({ title: z.string().min(1).max(100) });
const CreateTaskNode = z.object({
  task: CreateTask,
  x: z.number(),
  y: z.number(),
});
const UpdateTask = z.object({ status: TaskStatus, title: z.string() });
const DeleteTaskResponse = z.object({ task_id: z.string() });

export const schemas = {
  User,
  Session,
  SessionResponse,
  CreateUser,
  SignupSessionResponse,
  ConnectSubtask,
  DisconnectSubtask,
  ReconnectSubtask,
  UpdateTaskNodeInfo,
  TaskNodeInfo,
  TaskStatus,
  Task,
  TaskNode,
  CreateTask,
  CreateTaskNode,
  UpdateTask,
  DeleteTaskResponse,
};

const endpoints = makeApi([
  {
    method: "post",
    path: "/auth/cancel-signup",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "get",
    path: "/auth/login",
    requestFormat: "json",
    parameters: [
      {
        name: "after_login_redirect",
        type: "Query",
        schema: z.string().nullish(),
      },
    ],
    response: z.void(),
  },
  {
    method: "get",
    path: "/auth/login-callback",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "post",
    path: "/auth/logout",
    requestFormat: "json",
    response: z.void(),
  },
  {
    method: "get",
    path: "/auth/session",
    requestFormat: "json",
    response: SessionResponse,
  },
  {
    method: "post",
    path: "/auth/signup",
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
    path: "/auth/signup-session",
    requestFormat: "json",
    response: z.object({ session_exists: z.boolean() }),
  },
  {
    method: "post",
    path: "/subtask/connect",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: ConnectSubtask,
      },
    ],
    response: z.void(),
  },
  {
    method: "delete",
    path: "/subtask/disconnect",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: DisconnectSubtask,
      },
    ],
    response: z.void(),
  },
  {
    method: "put",
    path: "/subtask/reconnect",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: ReconnectSubtask,
      },
    ],
    response: z.void(),
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
    response: z.object({ task_id: z.string() }),
  },
]);

export const _api = new Zodios(endpoints);

export function createApiClient(baseUrl: string, options?: ZodiosOptions) {
  return new Zodios(baseUrl, endpoints, options);
}
