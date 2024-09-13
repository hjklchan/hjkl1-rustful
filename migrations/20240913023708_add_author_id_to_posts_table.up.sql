-- Add up migration script here
ALTER TABLE `posts` ADD `author_id` BIGINT UNSIGNED NOT NULL;
