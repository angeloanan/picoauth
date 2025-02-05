
--
-- Sqlite SQL Schema dump automatic generated by geni
--

CREATE TABLE schema_migrations (id VARCHAR(255) PRIMARY KEY);
CREATE TABLE "users" (
    "id" integer,
    "username" text NOT NULL,
    "password" text NOT NULL,
    "display_name" text DEFAULT NULL,
    "email" text DEFAULT NULL,
    --
    "totp_secret" text DEFAULT NULL,
    "totp_active_at" datetime DEFAULT NULL,
    --
    "requires_password_reset" integer NOT NULL DEFAULT 0,
    "requires_second_factor" integer NOT NULL DEFAULT 0,
    "email_verified_at" datetime DEFAULT NULL,
    --
    "created_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
) RANDOM ROWID;
CREATE TABLE "revoked_refresh_jwt" (
    "token" text NOT NULL,
    "revoked_at" datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (token)
);
CREATE TABLE "forgot_password_token" (
    "token" text NOT NULL,
    "user_id" integer NOT NULL,
    "expires_at" datetime NOT NULL,
    PRIMARY KEY (token),
    FOREIGN KEY (user_id) REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE
);