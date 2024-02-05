CREATE TABLE `users` (
    `id` text PRIMARY KEY NOT NULL,
    `name` text NOT NULL,
    `profile` text NOT NULL
);

CREATE TABLE `tasks` (
    `id` text PRIMARY KEY NOT NULL,
    `status` text DEFAULT 'Todo' NOT NULL,
    `title` text NOT NULL,
    `user_id` text NOT NULL,
    `created_at` text DEFAULT (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime')) NOT NULL,
    `updated_at` text DEFAULT (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime')) NOT NULL,

    FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade,
    CHECK (`status` = 'Todo' OR `status` = 'Done')
);

CREATE TRIGGER `trigger_tasks_updated_at` AFTER UPDATE ON `tasks`
BEGIN
    UPDATE `tasks` SET `updated_at` = strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime') WHERE rowid == NEW.rowid;
END;

CREATE TABLE `subtask_connections` (
    `subtask_id` text NOT NULL,
    `parent_task_id` text NOT NULL,
    `user_id` text NOT NULL,

    FOREIGN KEY (`subtask_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`parent_task_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade,
    PRIMARY KEY (`subtask_id`, `parent_task_id`)
);

CREATE TABLE `task_node_info` (
    `task_id` text PRIMARY KEY NOT NULL,
    `x` real NOT NULL,
    `y` real NOT NULL,
    `user_id` text NOT NULL,
    
    FOREIGN KEY (`task_id`) REFERENCES `tasks`(`id`) ON UPDATE no 
    action ON DELETE cascade,
    FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade
);