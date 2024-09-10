-- Add up migration script here
CREATE TABLE IF NOT EXISTS `categories` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(100) NOT NULL,
    `parent_id` BIGINT UNSIGNED DEFAULT 0,
    `description` VARCHAR(255) DEFAULT NULL,
    PRIMARY KEY (`id`)
);
