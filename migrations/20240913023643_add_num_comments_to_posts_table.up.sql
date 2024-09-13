-- Add up migration script here
ALTER TABLE `posts` ADD `num_comments` INT UNSIGNED NOT NULL;
