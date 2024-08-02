-- create rooms table
CREATE TABLE rooms (
    room_id uuid,
    PRIMARY KEY (room_id),
    title text NOT NULL, 
    image text NOT NULL, 
    description text NOT NULL, 
    lon DOUBLE PRECISION NOT NULL, 
    lat DOUBLE PRECISION NOT NULL,
    account_id uuid
);