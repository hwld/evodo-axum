CREATE TABLE subtasks (
    `subtask_id` text NOT NULL,
    `parent_task_id` text NOT NULL,

    FOREIGN KEY (`subtask_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    FOREIGN KEY (`parent_task_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade,
    PRIMARY KEY (`subtask_id`, `parent_task_id`)
);