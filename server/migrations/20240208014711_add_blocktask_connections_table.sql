CREATE TABLE `blocking_tasks` (
    -- ブロックしているタスク
    blocking_task_id text NOT NULL,
    -- ブロックされているタスク
    blocked_task_id text NOT NULL, 
    user_id text NOT NULL,

    FOREIGN KEY (`blocking_task_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`blocked_task_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade,
    PRIMARY KEY (`blocking_task_id`, `blocked_task_id`)
)