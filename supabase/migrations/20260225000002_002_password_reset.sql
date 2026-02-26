-- Migration: 002_password_reset
-- Description: Add password reset token columns to users table

ALTER TABLE users 
ADD COLUMN reset_token VARCHAR(255),
ADD COLUMN reset_token_expires_at TIMESTAMPTZ;

CREATE INDEX idx_users_reset_token ON users(reset_token);
