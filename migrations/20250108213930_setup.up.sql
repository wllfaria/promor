CREATE TABLE IF NOT EXISTS store_handler_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL
);

INSERT INTO store_handler_types (name) VALUES ('scraper'), ('api');

CREATE TABLE IF NOT EXISTS store_apis (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL
);

INSERT INTO store_apis (name) VALUES ('amazon'), ('mercado_livre');

CREATE TABLE IF NOT EXISTS store_handlers (
    id SERIAL PRIMARY KEY,
    type_id INT REFERENCES store_handler_types(id) ON DELETE CASCADE NOT NULL,
    api_id INT REFERENCES store_apis(id) ON DELETE SET NULL,
    parser VARCHAR(100)
);

INSERT INTO store_handlers (type_id, api_id, parser) VALUES (1, NULL, 'kabum_search');

CREATE TYPE HANDLER AS ENUM ('api', 'scraper');

CREATE TABLE IF NOT EXISTS stores (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    url TEXT NOT NULL,
    handler HANDLER NOT NULL,
    api_id INT REFERENCES store_apis(id) ON DELETE CASCADE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
