-- Track whether current presence was user-manual (dnd/away/offline) or connection-driven.
ALTER TABLE users
ADD COLUMN IF NOT EXISTS presence_manual BOOLEAN NOT NULL DEFAULT false;
