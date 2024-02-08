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
const ConnectBlockTask = z.object({
  blocked_task_id: z.string(),
  blocking_task_id: z.string(),
});
const DisconnectBlockTask = z.object({
  blocked_task_id: z.string(),
  blocking_task_id: z.string(),
});
const ReconnectBlockTask = z.object({
  new_blocked_task_id: z.string(),
  new_blocking_task_id: z.string(),
  old_blocked_task_id: z.string(),
  old_blocking_task_id: z.string(),
});
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
  task_id: z.string(),
  user_id: z.string(),
  x: z.number(),
  y: z.number(),
});
const TaskStatus = z.enum(["Todo", "Done"]);
const Task = z.object({
  blocked_task_ids: z.array(z.string()),
  created_at: z.string(),
  description: z.string(),
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
const UpdateTask = z.object({
  description: z.string().max(2000),
  title: z.string().min(1).max(100),
});
const DeleteTaskResponse = z.object({ task_id: z.string() });
const UpdateTaskStatus = z.object({ status: TaskStatus });

export const schemas = {
  User,
  Session,
  SessionResponse,
  CreateUser,
  SignupSessionResponse,
  ConnectBlockTask,
  DisconnectBlockTask,
  ReconnectBlockTask,
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
  UpdateTaskStatus,
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
    path: "/block-task/connect",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: ConnectBlockTask,
      },
    ],
    response: z.void(),
  },
  {
    method: "delete",
    path: "/block-task/disconnect",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: DisconnectBlockTask,
      },
    ],
    response: z.void(),
  },
  {
    method: "put",
    path: "/block-task/reconnect",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: ReconnectBlockTask,
      },
    ],
    response: z.void(),
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
    path: "/task-nodes/:id",
    requestFormat: "json",
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
    method: "get",
    path: "/tasks/:id",
    requestFormat: "json",
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
  {
    method: "put",
    path: "/tasks/:id/update-status",
    requestFormat: "json",
    parameters: [
      {
        name: "body",
        type: "Body",
        schema: UpdateTaskStatus,
      },
    ],
    response: Task,
  },
]);

export const _api = new Zodios(endpoints);

export function createApiClient(baseUrl: string, options?: ZodiosOptions) {
  return new Zodios(baseUrl, endpoints, options);
}
