import type { CheatSheetData } from './index'

export const dockerCommands: CheatSheetData = {
  id: 'docker',
  name: 'Docker 命令',
  description: '容器运行、镜像管理、Dockerfile、Compose 等高频 Docker 命令速查',
  columns: [
    { key: 'command', label: '命令', width: '280px', copyable: true },
    { key: 'category', label: '分类', width: '100px' },
    { key: 'params', label: '常用参数', width: '220px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // === 容器运行 ===
    { command: 'docker run [选项] 镜像 [命令]', category: '运行', params: '-d 后台; --name 名称; -p 宿主:容器; -P 自动端口映射; -e KEY=VAL 环境变量; --rm 退出后删; -v 挂载; --network; --restart', description: '创建并运行容器', examples: ['docker run -d --name nginx -p 80:80 nginx:alpine', 'docker run --rm -it alpine sh', 'docker run -d -p 3306:3306 -e MYSQL_ROOT_PASSWORD=secret --name mysql mysql:8', 'docker run -v $(pwd):/app -w /app node:18 npm test'] },
    { command: 'docker run --restart 策略', category: '运行', params: 'no 不重启(默认); always 总是; unless-stopped 除非手动停; on-failure[:N] 失败重启N次', description: '重启策略', examples: ['docker run -d --restart=always --name api api:latest', 'docker run -d --restart=on-failure:3 --name worker worker:v2'] },
    { command: 'docker ps [选项]', category: '运行', params: '无参 运行中; -a 全部; -q 仅ID; --format "table {{.Names}} {{.Status}}"; -n N 最近N个; -f 过滤', description: '查看容器列表', examples: ['docker ps -a --format "table {{.Names}}\\t{{.Status}}\\t{{.Ports}}"', 'docker ps -a -f "status=exited" -q | head', 'docker ps -n 5'] },
    { command: 'docker start / stop / restart / pause 容器', category: '运行', params: 'start: -a 附加输出; stop: -t 超时秒; restart: -t; pause/unpause 暂停/恢复', description: '容器生命周期控制', examples: ['docker stop -t 10 nginx', 'docker restart --time 30 api', 'docker pause worker && docker unpause worker'] },
    { command: 'docker kill [选项] 容器', category: '运行', params: '-s 信号(默认SIGKILL); -e 信号; 比 stop 强制', description: '强制终止（发信号）', examples: ['docker kill -s SIGTERM api', 'docker kill $(docker ps -q)'] },
    { command: 'docker rm [选项] 容器', category: '运行', params: '-f 强制(运行中也删); -v 同时删匿名卷; -f 过滤全部已停', description: '删除容器', examples: ['docker rm -f nginx', 'docker rm $(docker ps -aq)', 'docker system prune -f'] },
    { command: 'docker exec [选项] 容器 命令', category: '运行', params: '-it 交互TTY; -d 后台; -u 用户; -e KEY=VAL; -w 工作目录; --privileged 特权', description: '在运行容器内执行命令', examples: ['docker exec -it nginx sh', 'docker exec -it mysql mysql -uroot -p', 'docker exec -u 0 api cat /etc/passwd'] },
    { command: 'docker logs [选项] 容器', category: '运行', params: '-f 持续跟踪; -t 时间戳; -n N 最后N行; --since "时间"; --until; --tail N 最后N行; --details 详细', description: '查看容器日志', examples: ['docker logs -f --tail 200 nginx', 'docker logs --since "2h" api', 'docker logs -f -t --since 1h --until 30m ago worker'] },
    { command: 'docker inspect [选项] 容器/镜像', category: '运行', params: '-f "模板" Go模板输出; -s 统计; -e 检查; --format', description: '查看详细元数据（JSON/结构化）', examples: ['docker inspect nginx', 'docker inspect -f "{{.State.Status}}\\t{{.Config.Image}}" nginx', 'docker inspect -s mysql'] },
    { command: 'docker top 容器', category: '运行', params: '无参数; ps [options]', description: '容器内进程列表', examples: ['docker top nginx'] },
    { command: 'docker cp [选项] 源 目标', category: '运行', params: '容器:路径 与 宿主机路径互拷; -a 保留权限; -L 跟随软链接', description: '容器与宿主机间复制文件', examples: ['docker cp nginx:/etc/nginx/nginx.conf ./nginx.conf', 'docker cp ./src app:/app/src', 'docker cp app:/var/log ./docker_logs/'] },
    { command: 'docker diff 容器', category: '运行', params: '无参数; A新增 D删 M改', description: '查看容器文件系统变化', examples: ['docker diff nginx'] },
    // === 镜像 ===
    { command: 'docker images / image ls [选项]', category: '镜像', params: '-a 全部含中间层; -q 仅ID; -d 悬空镜像; --format; -f filter', description: '列出本地镜像', examples: ['docker images -a --format "table {{.Repository}}\\t{{.Tag}}\\t{{.Size}}"', 'docker images -f "dangling=true" -q'] },
    { command: 'docker pull 镜像[:tag|@digest]', category: '镜像', params: '-a 拉取所有tag; --platform linux/amd64; --quiet', description: '从仓库拉取镜像', examples: ['docker pull nginx:alpine', 'docker pull node:20-bookworm-slim', 'docker pull --platform linux/amd64 alpine:3'] },
    { command: 'docker push 镜像', category: '镜像', params: '-a 推送所有tag; --digest', description: '推送镜像到仓库', examples: ['docker push myapp:v1.0', 'docker tag myapp:v1.0 myregistry.com/myapp:v1.0 && docker push myregistry.com/myapp:v1.0'] },
    { command: 'docker build [选项] PATH | URL | -', category: '镜像', params: '-t 标签; -f Dockerfile; --progress=plain 进度; --no-cache; --build-arg KEY=VAL; --target 多阶段目标; --platform linux/amd64,linux/arm64; -q 安静; --network host', description: '构建镜像', examples: ['docker build -t myapp:v1.0 .', 'docker build -t myapp:dev -f Dockerfile.dev .', 'docker build --build-arg NODE_ENV=production -t myapp .', 'docker build --platform linux/amd64 -t myapp .'] },
    { command: 'docker tag 源镜像[:tag] 目标[:tag]', category: '镜像', params: '无参数; 等于创建别名', description: '给镜像打标签', examples: ['docker tag myapp:latest myapp:v1.0.0', 'docker tag ubuntu:22.04 registry.example.com/base/ubuntu:22.04'] },
    { command: 'docker rmi [选项] 镜像', category: '镜像', params: '-f 强制; -p 同时删被推标签镜像; -f 悬空', description: '删除镜像', examples: ['docker rmi nginx:old', 'docker rmi $(docker images -q -f dangling=true)'] },
    { command: 'docker save / load [选项]', category: '镜像', params: 'save -o 文件; load -i 文件; save 多个镜像用空格', description: '导出/导入镜像（离线传输）', examples: ['docker save -o myapp.tar myapp:v1.0', 'docker load -i myapp.tar', 'docker save nginx:alpine redis:7 -o images.tar'] },
    // === 清理 ===
    { command: 'docker system prune [选项]', category: '清理', params: '-a 清理所有非正在使用; -f 强制(不确认); --volumes 含卷; --filter; -d 仅悬空镜像', description: '一键清理（释放磁盘）', examples: ['docker system prune -af', 'docker system prune -af --volumes', 'docker system df  # 先看占用'] },
    { command: 'docker image / container / network prune', category: '清理', params: 'image prune -a -f; container prune -f; network prune -f; volume prune', description: '分别清理镜像/容器/网络/卷', examples: ['docker image prune -af', 'docker container prune -f', 'docker volume prune'] },
    // === 数据卷 ===
    { command: 'docker volume [子命令]', category: '卷', params: 'create/ls/rm/inspect/prune', description: '命名卷管理', examples: ['docker volume create mysql_data', 'docker volume ls', 'docker volume rm mysql_data'] },
    { command: 'docker run -v 用法', category: '卷', params: '命名卷: -v name:/path; 绑定: -v /host:/container:ro; tmpfs: --tmpfs /path', description: '挂载数据', examples: ['docker run -d -v mysql_data:/var/lib/mysql mysql', 'docker run -d -v /data/mysql:/var/lib/mysql:ro mysql', 'docker run -d --tmpfs /tmp:rw,noexec,nosuid,size=100m app'] },
    // === 网络 ===
    { command: 'docker network [子命令]', category: '网络', params: 'create/ls/rm/inspect/connect/disconnect; create -d bridge|overlay|none|host; --subnet --gateway', description: 'Docker 网络管理', examples: ['docker network create --driver bridge --subnet 172.20.0.0/16 mynet', 'docker network connect mynet nginx', 'docker network ls'] },
    { command: 'docker run --network 模式', category: '网络', params: 'bridge(默认); host 用宿主机网络; none 无; container:ID 共享; 自定义网络名', description: '指定容器网络模式', examples: ['docker run -d --network host redis', 'docker run -d --network appnet api', 'docker run -d --network none offline_job'] },
    // === Dockerfile ===
    { command: 'FROM 基础镜像[:tag]', category: 'Dockerfile', params: '--platform; 多阶段 FROM AS alias; scratch 空镜像', description: '指定基础镜像（必须第一条）', examples: ['FROM node:20-alpine AS build', 'FROM nginx:alpine', 'FROM scratch'] },
    { command: 'COPY / ADD 源 目标', category: 'Dockerfile', params: 'ADD 自动解压tar; COPY --from=构建阶段; COPY --chown=user:group; 通配符', description: '复制文件（ADD 还能自动解压）', examples: ['COPY package.json ./', 'COPY --from=build /app/dist /usr/share/nginx/html', 'ADD app.tar.gz /tmp/'] },
    { command: 'RUN 命令 && 命令', category: 'Dockerfile', params: '建议 \\ 换行 && 连接; apt-get 版本锁定; 清理缓存在同一条 RUN 里', description: '执行命令创建新层（每一条 RUN 是一层）', examples: ['RUN apt-get update && apt-get install -y --no-install-recommends curl=7.* && rm -rf /var/lib/apt/lists/*', 'RUN npm install && npm cache clean --force', 'RUN adduser -D -h /app app && chown -R app:app /app'] },
    { command: 'WORKDIR / APP / EXPOSE / ENV / ARG', category: 'Dockerfile', params: 'WORKDIR 自动创建; EXPOSE 仅文档声明; ARG 构建时变量(--build-arg); ENV 运行时变量', description: '工作目录/入口/端口/环境变量', examples: ['WORKDIR /app', 'EXPOSE 8080', 'ENV NODE_ENV=production', 'ARG VERSION=1.0', 'CMD ["node", "server.js"]'] },
    { command: 'ENTRYPOINT vs CMD', category: 'Dockerfile', params: 'ENTRYPOINT ["exec"] 不可覆盖(除非--entrypoint); CMD 可被 docker run 覆盖; 组合: ENTRYPOINT固定 + CMD默认参数', description: '容器启动命令', examples: ['ENTRYPOINT ["node"]', 'CMD ["server.js"]', '-- 效果: 默认启动 node server.js，可 docker run myapp index.js 覆盖'] },
    { command: 'USER / HEALTHCHECK / LABEL / STOPSIGNAL', category: 'Dockerfile', params: 'USER 非root运行; HEALTHCHECK 健康检查; LABEL 元数据; STOPSIGNAL 停止信号', description: '安全/元数据/健康检查', examples: ['USER app', 'HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -f http://localhost:8080/health || exit 1', 'LABEL org.opencontainers.image.version="1.0.0"'] },
    // === Compose ===
    { command: 'docker compose up [选项]', category: 'Compose', params: '-d 后台; --build 先构建; -f 指定文件; --scale service=N; --remove-orphans; -t 超时; --force-recreate', description: '启动服务（创建/启动/重建）', examples: ['docker compose up -d', 'docker compose up -d --build', 'docker compose -f docker-compose.dev.yml up', 'docker compose up -d --scale worker=3'] },
    { command: 'docker compose down [选项]', category: 'Compose', params: '-v 删除卷; --rmi all/local 删除镜像; --volumes; -t 超时', description: '停止并删除服务/网络/容器', examples: ['docker compose down', 'docker compose down -v', 'docker compose down --rmi all'] },
    { command: 'docker compose ps / logs / exec / pull / build', category: 'Compose', params: 'ps -a; logs -f service; exec -it service sh; pull service; build service --no-cache', description: '常用子命令', examples: ['docker compose ps', 'docker compose logs -f --tail=100 api', 'docker compose exec mysql mysql -uroot -p', 'docker compose build --no-cache api'] },
    { command: 'docker compose stop / restart / start / kill', category: 'Compose', params: 'stop [service]; restart service; start service; kill -s SIGTERM service', description: '服务生命周期', examples: ['docker compose stop api', 'docker compose restart -t 10 api', 'docker compose kill -s SIGKILL worker'] },
    { command: 'docker compose config', category: 'Compose', params: '解析/合并配置; 验证yaml正确性', description: '查看合并后的配置', examples: ['docker compose config', 'docker compose -f base.yml -f prod.yml config'] },
    // === 仓库/其他 ===
    { command: 'docker login [选项] [registry]', category: '仓库', params: '-u 用户名; -p 密码; 无参默认 Docker Hub', description: '登录远程仓库', examples: ['docker login', 'docker login -u myuser myregistry.com', 'docker logout myregistry.com'] },
    { command: 'docker search [选项] 关键词', category: '仓库', params: '--limit N; --no-trunc; --stars N; --automated', description: '搜索镜像（Docker Hub）', examples: ['docker search --stars 100 nginx', 'docker search --limit 10 redis'] },
    { command: 'docker info / version', category: '其他', params: 'info 全量信息; version 客户端/服务端版本; --format Go模板', description: 'Docker 环境信息', examples: ['docker info', 'docker version', 'docker info --format "{{.ServerVersion}}\\n{{.OperatingSystem}}"'] },
  ],
}
