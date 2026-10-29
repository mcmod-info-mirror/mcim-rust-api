redis-tool-install:
    sudo apt-get install lsb-release curl gpg
    curl -fsSL https://packages.redis.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/redis-archive-keyring.gpg
    sudo chmod 644 /usr/share/keyrings/redis-archive-keyring.gpg
    echo "deb [signed-by=/usr/share/keyrings/redis-archive-keyring.gpg] https://packages.redis.io/deb $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/redis.list
    sudo apt-get update
    sudo apt install redis-tools # for redis-cli

# 用 data/seed.sql（表结构 + 测试数据）重建 DATABASE_URL 指向的数据库（会清空 public schema）
import-data:
    PGOPTIONS='-c client_min_messages=warning' psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c 'DROP SCHEMA public CASCADE' -c 'CREATE SCHEMA public'
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -o /dev/null -f data/seed.sql

# 将 DATABASE_URL 中的当前数据导出为 data/seed.sql
dump-seed:
    pg_dump --no-owner --no-privileges "$DATABASE_URL" > data/seed.sql

# 将 MONGODB_URI / MONGODB_DATABASE 中的全部数据迁移到 DATABASE_URL 指向的 PostgreSQL
migrate-from-mongodb:
    cargo run --release -p migrate-mongo -- --truncate


build:
    cargo build --release

ci-test:
    cargo test --verbose