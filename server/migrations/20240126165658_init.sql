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

CREATE TABLE subtask_connections (
    `subtask_id` text NOT NULL,
    `parent_task_id` text NOT NULL,
    `user_id` text NOT NULL,

    FOREIGN KEY (`subtask_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`parent_task_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade,
    PRIMARY KEY (`subtask_id`, `parent_task_id`)
);

CREATE TABLE `task_node_info` (
    -- フロントエンド側で一意なIDを生成したいので、task_idではなく独自のidを主キーにする
    `id` text PRIMARY KEY NOT NULL,
    `task_id` text UNIQUE NOT NULL,
    `x` real NOT NULL,
    `y` real NOT NULL,
    `user_id` text NOT NULL,
    
    FOREIGN KEY (`task_id`) REFERENCES `tasks`(`id`) ON UPDATE no 
    action ON DELETE cascade,
    FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade
);