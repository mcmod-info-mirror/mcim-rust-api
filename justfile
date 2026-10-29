redis-tool-install:
    sudo apt-get install lsb-release curl gpg
    curl -fsSL https://packages.redis.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/redis-archive-keyring.gpg
    sudo chmod 644 /usr/share/keyrings/redis-archive-keyring.gpg
    echo "deb [signed-by=/usr/share/keyrings/redis-archive-keyring.gpg] https://packages.redis.io/deb $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/redis.list
    sudo apt-get update
    sudo apt install redis-tools # for redis-cli

# 将 data/ 下的测试数据导入 DATABASE_URL 指向的 PostgreSQL
import-data:
    cargo run -p migrate-mongo -- --from-json data --truncate

# 将 MONGODB_URI / MONGODB_DATABASE 中的全部数据迁移到 DATABASE_URL 指向的 PostgreSQL
migrate-from-mongodb:
    cargo run --release -p migrate-mongo -- --truncate


build:
    cargo build --release

ci-test:
    cargo test --verbose