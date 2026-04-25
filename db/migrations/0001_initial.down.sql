-- ===============================================
-- 0001_initial.down.sql
-- Roll back the initial schema in dependency order
-- ===============================================

DROP TABLE IF EXISTS uploaded_files;
DROP TABLE IF EXISTS event_calendar;
DROP TABLE IF EXISTS email_verification_token;
DROP TABLE IF EXISTS notification_outbox;
DROP TABLE IF EXISTS mailbox_entry;
DROP TABLE IF EXISTS booking;
DROP TABLE IF EXISTS user_auth;
DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS article_tags;
DROP TABLE IF EXISTS article;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS towns;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS users;
