CREATE TABLE entries (
  id SERIAL PRIMARY KEY,
  namespace VARCHAR(64) NOT NULL,
  content TEXT NOT NULL
);
