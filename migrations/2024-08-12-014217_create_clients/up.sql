-- Your SQL goes here
CREATE TABLE "clients"(
	"id" BIGSERIAL NOT NULL PRIMARY KEY,
	"name" TEXT NOT NULL,
	"status" TEXT NOT NULL,
	"description" TEXT,
	"location" TEXT,
	"revoked" BOOL NOT NULL,
	"auth_key" TEXT NOT NULL,
	"api_key" TEXT,
	"created_on" TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"accessed_on" TIMESTAMP
);

