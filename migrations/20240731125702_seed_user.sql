-- Add migration script here
INSERT INTO users (id, account_name, account_password, account_email)
VALUES (
    'ddf8994f-d522-4659-8d02-c1d479057be6',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$VeuJXGwjapv4Ce2YMUHx1Q$76EMyGRxKuRKaEFSIQTtjImCl4bJzXDPIVfxg7TXy30',
    'test@test.com'
);