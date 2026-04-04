-- ===============================================
-- 00-drop-tables.sql
-- Drops the application schema in dependency order.
-- ===============================================

DROP TABLE IF EXISTS uploaded_files;
DROP TABLE IF EXISTS event_calendar;
DROP TABLE IF EXISTS user_auth;
DROP TABLE IF EXISTS article;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS towns;
DROP TABLE IF EXISTS users;
