-- Your SQL goes here
CREATE TABLE `clients`(
	`id` INT8 NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`description` TEXT NOT NULL,
	`location` TEXT NOT NULL,
	`revoked` BOOL NOT NULL,
	`auth_key` TEXT NOT NULL,
	`api_key` TEXT NOT NULL,
	`created_on` TEXT NOT NULL,
	`accessed_on` TEXT NOT NULL
);

