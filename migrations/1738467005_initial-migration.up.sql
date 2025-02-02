-- Write your up sql migration here
CREATE TABLE IF NOT EXISTS "users" (
    "id" integer,
    "username" text NOT NULL,
    "password" text NOT NULL,
    "display_name" text DEFAULT NULL,
    "email" text DEFAULT NULL,
    --
    "totp_secret" text DEFAULT NULL,
    --
    "requires_password_reset" integer NOT NULL DEFAULT 0,
    "requires_second_factor" integer NOT NULL DEFAULT 0,
    "email_verified_at" datetime DEFAULT NULL,
    --
    "created_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
) RANDOM ROWID;
