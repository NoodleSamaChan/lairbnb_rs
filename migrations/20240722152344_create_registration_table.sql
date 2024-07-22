-- Create Registration Table
CREATE TABLE users(
   id uuid NOT NULL,
   PRIMARY KEY (id),
   account_name text UNIQUE NOT NULL,
   account_password text NOT NULL
);