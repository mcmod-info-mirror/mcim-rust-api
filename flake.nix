{
  description = "mcim-rust-api development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # PostgreSQL 开发约定：
        # - 所有数据库文件（数据目录、socket、日志）只放在 /tmp/mcim-rust-api-pg 下，
        #   不写入项目目录或 $HOME，避免污染本机存储；重启/清理 /tmp 后数据即丢失。
        # - 只监听 unix socket（listen_addresses=''），不占用本机 5432 端口。
        # - 使用 pg-start / pg-stop / pg-reset 管理，首次 pg-start 自动 initdb。
        pgRoot = "/tmp/mcim-rust-api-pg";

        pgStart = pkgs.writeShellScriptBin "pg-start" ''
          set -euo pipefail
          mkdir -p "$PGHOST"
          if [ ! -f "$PGDATA/PG_VERSION" ]; then
            initdb -D "$PGDATA" -U "$PGUSER" --auth=trust --encoding=UTF8 >/dev/null
          fi
          pg_ctl -D "$PGDATA" -l "${pgRoot}/postgres.log" \
            -o "-k $PGHOST -c listen_addresses='''" -w start
          psql -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$PGDATABASE'" | grep -q 1 \
            || createdb "$PGDATABASE"
        '';

        pgStop = pkgs.writeShellScriptBin "pg-stop" ''
          pg_ctl -D "$PGDATA" -m fast stop
        '';

        # 停止并删除 /tmp 下全部 PostgreSQL 文件
        pgReset = pkgs.writeShellScriptBin "pg-reset" ''
          pg_ctl -D "$PGDATA" -m fast stop 2>/dev/null || true
          rm -rf "${pgRoot}"
        '';

        # Redis 开发约定（同 PostgreSQL）：
        # - 数据（RDB/AOF）、socket、pid、日志全部放在 /tmp/mcim-rust-api-redis 下，不污染本机存储。
        # - 只监听 unix socket（port 0），不占用本机 6379 端口；REDIS_URL 指向该 socket。
        # - 使用 redis-start / redis-stop / redis-reset 管理。
        redisRoot = "/tmp/mcim-rust-api-redis";

        redisStart = pkgs.writeShellScriptBin "redis-start" ''
          set -euo pipefail
          mkdir -p "${redisRoot}"
          redis-server --daemonize yes \
            --port 0 \
            --unixsocket "${redisRoot}/redis.sock" --unixsocketperm 700 \
            --dir "${redisRoot}" \
            --pidfile "${redisRoot}/redis.pid" \
            --logfile "${redisRoot}/redis.log"
          for _ in $(seq 50); do
            [ -S "${redisRoot}/redis.sock" ] && exit 0
            sleep 0.1
          done
          echo "redis failed to start, see ${redisRoot}/redis.log" >&2
          exit 1
        '';

        redisStop = pkgs.writeShellScriptBin "redis-stop" ''
          redis-cli -s "${redisRoot}/redis.sock" shutdown nosave
        '';

        # 停止并删除 /tmp 下全部 Redis 文件
        redisReset = pkgs.writeShellScriptBin "redis-reset" ''
          redis-cli -s "${redisRoot}/redis.sock" shutdown nosave 2>/dev/null || true
          rm -rf "${redisRoot}"
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain (edition 2024)
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer

            # Native deps: reqwest default-tls -> openssl
            pkg-config
            openssl

            # Project tooling
            git
            just

            # Databases / clients
            postgresql
            pgStart
            pgStop
            pgReset
            redis
            redisStart
            redisStop
            redisReset
            mongodb-tools
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          RUST_BACKTRACE = "1";

          # PostgreSQL：数据与 socket 全部在 /tmp（见上方约定）
          PGDATA = "${pgRoot}/data";
          PGHOST = "${pgRoot}/run";
          PGUSER = "postgres";
          PGDATABASE = "mcim_backend";
          DATABASE_URL = "postgresql://postgres@/mcim_backend?host=${pgRoot}/run";
          # psql 命令历史默认写 ~/.psql_history，改到 /tmp
          PSQL_HISTORY = "${pgRoot}/psql_history";

          # Redis：数据与 socket 全部在 /tmp（见上方约定）
          REDIS_URL = "redis+unix://${redisRoot}/redis.sock";
          # redis-cli 命令历史默认写 ~/.rediscli_history，改到 /tmp
          REDISCLI_HISTFILE = "${redisRoot}/rediscli_history";
        };
      }
    );
}
