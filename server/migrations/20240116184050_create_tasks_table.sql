-- Add migration script here
CREATE TABLE `tasks` (
    `id` text PRIMARY KEY NOT NULL,
    `status` text DEFAULT 'Todo' NOT NULL,
    `title` text NOT NULL,
    `created_at` text DEFAULT (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime')) NOT NULL,
    `updated_at` text DEFAULT (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime')) NOT NULL,
    CHECK (`status` = 'Todo' OR `status` = 'Done')
)