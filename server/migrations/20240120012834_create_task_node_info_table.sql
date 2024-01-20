CREATE TABLE `task_node_info` (
    -- フロントエンド側で一意なIDを生成したいので、task_idではなく独自のidを主キーにする
    `id` text PRIMARY KEY NOT NULL,
    'task_id' text UNIQUE NOT NULL,
    `x` real NOT NULL,
    `y` real NOT NULL,
    FOREIGN KEY (`task_id`) REFERENCES `tasks`(`id`) ON UPDATE no action ON DELETE cascade
)
