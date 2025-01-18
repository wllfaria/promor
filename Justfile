default:
    just -l

migrate:
    just migrate_rev
    just migrate_run

migrate_run:
    sqlx migrate run

migrate_rev:
    sqlx migrate revert
