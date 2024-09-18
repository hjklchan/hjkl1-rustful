-- Add up migration script here
ALTER TABLE `posts` ADD `num_views` INT UNSIGNED DEFAULT 0;
