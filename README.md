# mcim-rust-api

![mcim-rust-api](https://socialify.git.ci/mcmod-info-mirror/mcim-rust-api/image?description=1&font=Inter&issues=1&language=1&name=1&owner=1&pattern=Overlapping%20Hexagons&pulls=1&stargazers=1&theme=Auto)

**MCIM API rewritten in Rust**

为各平台的 Mod 的 API 缓存加速

**这是一项公益服务，请不要攻击我们，你的[赞助](#赞助)将帮助我们维持服务器运营**  [![Stripe](https://img.shields.io/badge/Stripe-5469d4?style=for-the-badge&logo=stripe&logoColor=ffffff)](https://www.mcimirror.top/sponsor) 

已缓存 **绝大多数** 的 Modrinth 和 Curseforge 上的 Minecraft Mod 信息。

> [!Warning]
> 由于多种原因，OpenMCIM 已暂停运行，~~现在不再利用节点分发而是常规 CDN 分发~~，MCIM API 不受影响。  
> ~~文件下载已现在重定向到 [Pysio](https://github.com/pysio2007) 提供的 Cloudflare 镜像源上，现已投入运行~~  
> 文件下载暂时不提供

API 支持 [Curseforge](https://curseforge.com/) 和 [Modrinth](https://modrinth.com/)

启动器接入相关见 <https://www.mcimirror.top>，此处不再赘述

## 如何部署

首先你需要在 MongoDB 向 `mcim_backend` 导入 [MCIM Data](https://github.com/mcmod-info-mirror/data) 的数据，或者运行 [mcim-rust-sync](https://github.com/mcmod-info-mirror/mcim-rust-sync) 自行抓取。你可以直接调用它来完成 Modrinth 的抓取，但你很需要依靠用户请求来收集 Curseforge 的 Mod，暂无方式可以抓取 Curseforge 的全部 Mod。

然后用 Docker 部署 <https://hub.docker.com/r/z0z0r4/mcim-rust-api>，或者你可以自行构建，将环境变量填在 `.env` 直接运行。

### 🐳 使用 `docker run`

```bash
docker run -d \
  --name mcim-rust-api \
  --restart always \
  --network host \
  -e RUST_LOG=INFO \
  -e MONGODB_URI="mongodb://user:password@localhost" \
  -e REDIS_URL="redis://localhost" \
  -e CURSEFORGE_API_URL="https://api.curseforge.com" \
  -e MODRINTH_API_URL="https://api.modrinth.com" \
  -e CURSEFORGE_API_KEY="CURSEFORGE_API_KEY" \
  -e CURSEFORGE_FILE_CDN_URL="https://edge.forgecdn.net" \
  -e MODRINTH_FILE_CDN_URL="https://cdn.modrinth.com" \
  z0z0r4/mcim-rust-api:latest
```

---

### 📦 使用 `docker-compose`

`docker-compose.yml` 文件：

```yaml
services:
  mcim-rust-api:
    container_name: mcim-rust-api
    image: z0z0r4/mcim-rust-api:latest
    restart: always
    network_mode: host
    environment:
      RUST_LOG: INFO
      PORT: 8080
      MONGODB_URI: "mongodb://user:password@localhost"
      REDIS_URL: "redis://localhost"
      CURSEFORGE_API_URL: "https://api.curseforge.com"
      MODRINTH_API_URL: "https://api.modrinth.com"
      CURSEFORGE_API_KEY: "CURSEFORGE_API_KEY"
      CURSEFORGE_FILE_CDN_URL: "https://edge.forgecdn.net"
      MODRINTH_FILE_CDN_URL: "https://cdn.modrinth.com"
```

## 🛠 环境变量说明

| 变量名                       | 说明                      |
| ------------------------- | ----------------------- |
| `MONGODB_URI`             | MongoDB 数据库连接字符串        |
| `REDIS_URL`               | Redis 连接地址              |
| `CURSEFORGE_API_URL`      | CurseForge API 根地址      |
| `CURSEFORGE_API_KEY`      | CurseForge API Key      |
| `CURSEFORGE_FILE_CDN_URL` | CurseForge 文件 CDN 地址    |
| `MODRINTH_FILE_CDN_URL`   | Modrinth 文件 CDN 地址      |

> 🔒 请将 `MONGODB_URI`、`REDIS_URL` 与 `CURSEFORGE_API_KEY` 替换为你自己的配置。
