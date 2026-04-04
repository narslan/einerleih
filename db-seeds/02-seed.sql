-- Seed data for users
INSERT INTO users (id, username, email, created_by, created_at, modified_by, modified_at) VALUES
  ('00000000-0000-0000-0000-000000000001', 'user01', 'user01@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000002', 'user02', 'user02@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000003', 'user03', 'user03@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000004', 'user04', 'user04@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000005', 'user05', 'user05@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000006', 'user06', 'user06@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000007', 'user07', 'user07@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000008', 'user08', 'user08@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000009', 'user09', 'user09@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000010', 'user10', 'user10@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000011', 'user11', 'user11@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000012', 'user12', 'user12@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000013', 'user13', 'user13@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000014', 'user14', 'user14@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000015', 'user15', 'user15@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000016', 'user16', 'user16@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000017', 'user17', 'user17@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000018', 'user18', 'user18@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000019', 'user19', 'user19@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000020', 'user20', 'user20@example.com', NULL, NOW(), NULL, NOW()),
  ('00000000-0000-0000-0000-000000000021', 'apitest01', 'apitest01@example.com', NULL, NOW(), NULL, NOW());

INSERT INTO user_auth
(user_id, password_hash, created_at, modified_at)
VALUES('00000000-0000-0000-0000-000000000021', '$argon2id$v=19$m=19456,t=2,p=1$XBFwBY52C9SpzkxON1OTLg$djDqZQvzxFKc9HOCWyZfKy+RlFTs0BJFSkcw/Tos14c', NOW(), NOW());
