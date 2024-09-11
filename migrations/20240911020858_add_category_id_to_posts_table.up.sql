-- Add up migration script here
ALTER TABLE `posts` ADD `category_id` BIGINT UNSIGNED NOT NULL;
