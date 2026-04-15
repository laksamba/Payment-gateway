-- Add password_hash to merchants table for password login
ALTER TABLE merchants ADD COLUMN password_hash TEXT;