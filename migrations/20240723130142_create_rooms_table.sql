-- create rooms table
CREATE TABLE rooms (
    id uuid references users(id),
    PRIMARY KEY (id),
    title text NOT NULL, 
    image text NOT NULL, 
    description text NOT NULL, 
    lon DOUBLE PRECISION NOT NULL, 
    lat DOUBLE PRECISION NOT NULL
);