-- Add migration script here
ALTER TABLE rooms ADD COLUMN account_id uuid NOT NULL;