CREATE TABLE IF NOT EXISTS timestamps_table (
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS base_table (
    active BOOLEAN NOT NULL DEFAULT TRUE
) INHERITS (timestamps_table);

CREATE TABLE IF NOT EXISTS stores (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    url TEXT NOT NULL
) INHERITS (base_table);

CREATE TABLE IF NOT EXISTS pages (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    url TEXT NOT NULL,
    store_id INT REFERENCES stores(id) ON DELETE CASCADE NOT NULL,
    handler TEXT NOT NULL,
    page_kind TEXT NOT NULL,
    ean TEXT,
    gtin TEXT
) INHERITS (base_table);

CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    brand VARCHAR(100) NOT NULL,
    image TEXT,
    ean TEXT,
    gtin TEXT
) INHERITS (base_table);

CREATE INDEX ean_idx ON products (ean);
CREATE INDEX gtin_idx ON products (gtin);

CREATE TABLE IF NOT EXISTS product_prices (
    id SERIAL PRIMARY KEY,
    product_id INT REFERENCES products(id) NOT NULL,
    store_id INT REFERENCES stores(id) NOT NULL,
    price DECIMAL(19, 4) NOT NULL
) INHERITS (base_table);
