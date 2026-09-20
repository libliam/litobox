import type { CheatSheetData } from './index'

export const redisCommands: CheatSheetData = {
  id: 'redis',
  name: 'Redis 常用命令',
  description: 'Redis 数据结构/高可用/监控/Lua/客户端常用命令速查，每条命令可点击展开看示例',
  columns: [
    { key: 'command', label: '命令', width: '220px', copyable: true },
    { key: 'category', label: '分类', width: '110px' },
    { key: 'params', label: '常用参数', width: '240px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // === 通用/Key 管理 ===
    { command: 'KEYS pattern / SCAN 0 MATCH pattern', category: '通用', params: 'SCAN COUNT N; TYPE key; EXISTS key; DEL key; RENAME; EXPIRE; PEXPIRE 毫秒; TTL/PTTL; PERSIST', description: 'Key 的查找/删除/过期', examples: ['KEYS "user:*"  # 生产慎用，会阻塞', 'SCAN 0 MATCH "user:*" COUNT 100', 'TTL session:abc  # 剩余秒数，-1永久 -2不存在', 'DEL cache:key'] },
    { command: 'TYPE / EXISTS / DBSIZE / INFO', category: '通用', params: 'TYPE key; EXISTS key1 key2...; DBSIZE; INFO [section]; INFO memory/clients/stats/server', description: '查看类型/存在/DB状态', examples: ['TYPE user:1  # string/list/hash/set/zset/stream', 'EXISTS user:1 user:2', 'INFO memory  # 内存使用', 'INFO commandstats  # 命令耗时统计'] },
    { command: 'MIGRATE / DUMP / RESTORE', category: '通用', params: 'MIGRATE host port key destdb timeout [COPY] [REPLACE] [KEYS ...]; DUMP key; RESTORE key ttl blob', description: '跨实例迁移 Key', examples: ['MIGRATE 10.0.0.2 6379 user:1 0 5000 COPY REPLACE', 'DUMP user:1 | redis-cli -h dest RESTORE user:1 0'] },
    // === String ===
    { command: 'SET key value [EX seconds] [PX ms] [NX|XX]', category: 'String', params: 'EX 过期秒; PX 过期毫秒; NX 仅不存在时(setnx); XX 仅存在时', description: '设置字符串值（Redis 2.6.12+ 推荐用这个替代 setnx/setex）', examples: ['SET session:abc "data" EX 3600', 'SET lock:task random NX PX 30000  # 分布式锁', 'SET counter 0 NX'] },
    { command: 'GET / MGET / MSET / GETSET / GETDEL', category: 'String', params: 'MGET k1 k2; MSET k1 v1 k2 v2; GETSET 原子替换返回旧值; GETDEL 原子获取并删', description: '获取/批量操作', examples: ['MGET user:1 user:2 user:3', 'MSET user:1:a "张三" user:1:b 18', 'GETSET token:abc newval'] },
    { command: 'INCR / DECR / INCRBY / DECRBY / APPEND', category: 'String', params: 'INCR key +1; INCRBY key N; DECR 负数; APPEND key val', description: '原子计数/追加', examples: ['INCR article:1:views', 'INCRBY counter 10', 'APPEND log "line\\n"'] },
    { command: 'SETRANGE / GETRANGE / STRLEN', category: 'String', params: 'SETRANGE key offset val; GETRANGE key start end; STRLEN key', description: '子串操作', examples: ['SETRANGE token 10 "XXXX"', 'GETRANGE name 0 3'] },
    // === Hash ===
    { command: 'HSET / HGET / HMSET / HMGET', category: 'Hash', params: 'HSET key field val [field val...]; HMSET 同HSET; HGET key field; HMGET key f1 f2', description: 'Hash 字段读写', examples: ['HSET user:1 name "张三" age 18 city "北京"', 'HMGET user:1 name age city'] },
    { command: 'HGETALL / HKEYS / HVALS / HLEN', category: 'Hash', params: 'HGETALL 所有字段值; HKEYS 所有字段名; HVALS 所有值; HLEN 字段数', description: 'Hash 全量遍历', examples: ['HGETALL user:1', 'HKEYS user:1 | head -5'] },
    { command: 'HINCRBY / HDEL / HEXISTS', category: 'Hash', params: 'HINCRBY key field delta; HDEL key field [field...]; HEXISTS key field', description: 'Hash 修改/计数', examples: ['HINCRBY user:1 balance 100', 'HDEL user:1 city', 'HEXISTS user:1 email'] },
    // === List ===
    { command: 'LPUSH / RPUSH / LPOP / RPOP', category: 'List', params: 'LPUSH 左推入; RPUSH 右推入; LPOP 左弹出; RPOP 右弹出; LPUSHX 仅存在时推入', description: 'List 双端操作', examples: ['LPUSH queue task1 task2 task3', 'RPUSH logs "entry1" "entry2"', 'LPOP queue'] },
    { command: 'LRANGE / LLEN / LINDEX / LSET / LREM', category: 'List', params: 'LRANGE key 0 -1 全部; LLEN 长度; LINDEX key 索引(负从末尾); LSET 索引值; LREM key count val', description: 'List 访问/修改', examples: ['LRANGE queue 0 -1', 'LLEN queue', 'LREM queue 0 "old_task"'] },
    { command: 'BLPOP / BRPOP / BRPOPLPUSH', category: 'List', params: 'BLPOP key [key...] timeout 阻塞弹出; BRPOPLPUSH src dst timeout', description: '阻塞队列模式（消费者）', examples: ['BLPOP queue 30', 'BRPOPLPUSH queue processing 60'] },
    // === Set ===
    { command: 'SADD / SREM / SMEMBERS / SISMEMBER', category: 'Set', params: 'SADD key val [val...]; SREM; SMEMBERS 所有成员; SISMEMBER key val 1/0', description: 'Set 添加/删除/查询', examples: ['SADD tags redis golang docker', 'SREM tags docker', 'SISMEMBER tags redis'] },
    { command: 'SINTER / SUNION / SDIFF', category: 'Set', params: 'KEYS 1 2... 多集合; SINTERSTORE 结果存新key', description: '集合运算（交/并/差）', examples: ['SINTER user:1:tags user:2:tags', 'SUNION tag:frontend tag:backend', 'SDIFF all_tags selected_tags'] },
    { command: 'SCARD / SMISMEMBER / SORT', category: 'Set', params: 'SCARD 成员数; SMISMEMBER key m1 m2; SORT key [ALPHA] [LIMIT N M]', description: 'Set 其他操作', examples: ['SCARD user:1:tags', 'SORT myset ALPHA'] },
    // === Sorted Set ===
    { command: 'ZADD / ZSCORE / ZINCRBY', category: 'ZSet', params: 'ZADD key score member; ZADD NX XX CH GT LT; ZSCORE key member; ZINCRBY key delta member', description: '有序集合添加/计数', examples: ['ZADD rank 100 "张三" 95 "李四" 92 "王五"', 'ZADD rank NX 88 "赵六"', 'ZINCRBY rank 5 "张三"'] },
    { command: 'ZRANGE / ZREVRANGE / ZRANGEBYSCORE', category: 'ZSet', params: 'ZRANGE key 0 -1 WITHSCORES; ZREVRANGE 反向; ZRANGEBYSCORE key min max [LIMIT offset N]; ZRANGE key BYSCORE BYLEX REV', description: '按分数/排名取成员', examples: ['ZRANGE rank 0 -1 WITHSCORES', 'ZREVRANGE rank 0 9 WITHSCORES  # Top 10', 'ZRANGEBYSCORE rank 90 100 WITHSCORES'] },
    { command: 'ZRANK / ZREVRANK / ZCARD / ZCOUNT', category: 'ZSet', params: 'ZRANK key member 排名(从0); ZREVRANK 反向排名; ZCARD 成员数; ZCOUNT key min max', description: '排名/数量查询', examples: ['ZRANK rank "张三"', 'ZCOUNT rank 90 100'] },
    // === Stream (Redis 5.0+) ===
    { command: 'XADD key field value [field value...]', category: 'Stream', params: 'XADD key ID field val (ID=*自动生成); XMAXLEN ~ N 近似截断; TRIM', description: '向 Stream 添加消息', examples: ['XADD logs * level "error" msg "DB timeout" user "app"', 'XADD logs MAXLEN ~ 10000 * msg "test"'] },
    { command: 'XREAD / XREADGROUP', category: 'Stream', params: 'XREAD [COUNT N] [BLOCK ms] STREAMS key [key...] ID [ID...]; XREADGROUP GROUP g consumer STREAMS key ID', description: '读取消息（消费者组模式）', examples: ['XREAD COUNT 10 BLOCK 3000 STREAMS logs $', 'XREADGROUP GROUP g1 consumer1 COUNT 5 STREAMS logs >'] },
    { command: 'XGROUP / XACK / XPENDING', category: 'Stream', params: 'XGROUP CREATE key groupname ID; XACK key groupname ID [ID...]; XPENDING key groupname', description: '消费者组管理', examples: ['XGROUP CREATE logs g1 $ MKSTREAM', 'XACK logs g1 1699999999-0', 'XPENDING logs g1'] },
    // === HyperLogLog / Bitmap / GEO ===
    { command: 'PFADD / PFCOUNT / PFMERGE', category: '其他', params: 'PFADD key elems; PFCOUNT key [key...]; PFMERGE dest src1 src2', description: '基数估算（UV 统计，12KB/亿）', examples: ['PFADD uv:day1 user:1 user:2 user:3', 'PFCOUNT uv:day1 uv:day2  # 近似合并后UV'] },
    { command: 'SETBIT / GETBIT / BITCOUNT / BITOP', category: '其他', params: 'SETBIT key offset val; BITCOUNT key [start end]; BITOP AND dest src1 src2; BITFIELD 复杂操作', description: 'Bitmap（签到/在线状态）', examples: ['SETBIT sign:user1 20240101 1  # 2024-01-01 签到', 'BITCOUNT sign:user1  # 总共签到天数', 'BITOP AND active:week day1 day2 day3 day4 day5 day6 day7'] },
    { command: 'GEOADD / GEODIST / GEORADIUS / GEORADIUSBYMEMBER', category: '其他', params: 'GEOADD key lng lat member; GEODIST key m1 m2 [unit]; GEORADIUS key lng lat radius unit [WITHDIST] [ASC|DESC] [COUNT N]', description: '地理位置（Redis 3.2+）', examples: ['GEOADD shops 116.3974 39.9093 "天安门" 121.4737 31.2304 "外滩"', 'GEODIST shops "天安门" "外滩" km', 'GEORADIUS shops 116.3974 39.9093 5 km WITHDIST'] },
    // === 事务 ===
    { command: 'MULTI ... EXEC / DISCARD', category: '事务', params: 'MULTI 开启; 命令入队; EXEC 执行; DISCARD 放弃; 不保证原子性(需配合 WATCH)', description: 'Redis 事务（命令队列，单线程保证连续执行）', examples: ['MULTI\nINCR counter\nINCRBY counter 10\nEXEC', 'MULTI\nSET a 1\nDISDCARD'] },
    { command: 'WATCH / UNWATCH', category: '事务', params: 'WATCH key [key...]; 配合 MULTI EXEC 做乐观锁; EXEC 时若 key 被改则返回 null', description: 'CAS 乐观锁', examples: ['WATCH balance\nMULTI\nDECRBY balance 100\nEXEC  # 如果 balance 在这之间被改过，返回 [null]'] },
    { command: 'Lua 脚本 EVAL / SCRIPT', category: '事务', params: 'EVAL script numkeys key [key...] arg [arg...]; SCRIPT LOAD/EXISTS/FLUSH; redis.call("CMD") 执行命令', description: 'Lua 脚本保证原子性（最靠谱方案）', examples: ["EVAL \"return redis.call('INCR', KEYS[1])\" 1 counter", "EVALSHA sha1 1 counter  # 预加载后调用\nSCRIPT LOAD \"return redis.call('GET', KEYS[1])\""] },
    // === 高可用 ===
    { command: 'REPLICATIONOF / SLAVEOF / REPLICAOF', category: '高可用', params: 'REPLICAOF host port 设为主节点; REPLICAOF NO ONE 取消(变独立); INFO REPLICATION', description: '主从复制', examples: ['REPLICAOF 10.0.0.1 6379', 'INFO REPLICATION'] },
    { command: 'CLUSTER [子命令]', category: '高可用', params: 'MEET / NODES / INFO / SLOTS / ADDSLOTS / DELSLOTS / FORGET / FAILOVER', description: 'Redis Cluster 分片集群', examples: ['CLUSTER MEET 10.0.0.2 6379', 'CLUSTER NODES', 'CLUSTER SLOTS', 'CLUSTER FAILOVER'] },
    { command: 'SENTINEL [子命令]', category: '高可用', params: 'master / slaves / sentinels / get-master-addr-by-name / failover', description: '哨兵模式（故障转移）', examples: ['SENTINEL get-master-addr-by-name mymaster', 'SENTINEL failover mymaster', 'SENTINEL master mymaster'] },
    { command: 'PERSISTENCE: SAVE / BGSAVE / BGREWRITEAOF', category: '持久化', params: 'SAVE 同步快照; BGSAVE 后台 RDB; BGREWRITEAOF AOF重写; CONFIG SET appendonly yes; CONFIG SET save "900 1 300 10 60 10000"', description: 'RDB / AOF 持久化', examples: ['BGSAVE  # 触发RDB快照', 'BGREWRITEAOF  # 触发AOF重写', 'CONFIG SET appendonly yes  # 开启AOF'] },
    // === 发布订阅 ===
    { command: 'PUBLISH / SUBSCRIBE / PSUBSCRIBE / UNSUBSCRIBE', category: 'Pub/Sub', params: 'PUBLISH channel msg; SUBSCRIBE ch1 ch2; PSUBSCRIBE "news:*"; 客户端断连消息丢失（非持久）', description: '发布订阅（轻量，消息即发即弃）', examples: ['PUBLISH news "新内容发布"', 'SUBSCRIBE news alerts', 'PSUBSCRIBE "chat:*"'] },
    // === 监控/运维 ===
    { command: 'INFO / CONFIG / COMMAND / MEMORY', category: '运维', params: 'INFO [section]; CONFIG GET/SET key val; MEMORY USAGE key; MEMORY STATS; COMMAND COUNT; COMMAND INFO cmd', description: '配置/监控/调试', examples: ['INFO memory', 'CONFIG GET maxmemory', 'CONFIG SET maxmemory 256mb', 'MEMORY USAGE user:1', 'CONFIG SET maxmemory-policy allkeys-lru'] },
    { command: 'CLIENT [子命令]', category: '运维', params: 'LIST / KILL / GETNAME / SETNAME / PAUSE; CONFIG SET timeout 300', description: '客户端管理', examples: ['CLIENT LIST', 'CLIENT KILL addr 10.0.0.5:*', 'CLIENT SETNAME myclient', 'CLIENT PAUSE 30000 WRITE'] },
    { command: 'BGSAVE / LASTSAVE / DBSIZE', category: '运维', params: 'LASTSAVE 上次保存时间戳; DBSIZE 当前key数; FLUSHDB / FLUSHALL; SHUTDOWN [NOSAVE|SAVE]', description: '数据库操作', examples: ['LASTSAVE', 'DBSIZE', 'FLUSHDB ASYNC', 'FLUSHALL ASYNC', 'SHUTDOWN SAVE'] },
    { command: 'DEBUG OBJECT / DEBUG SLEEP', category: '运维', params: 'DEBUG OBJECT key; DEBUG SLEEP seconds; DEBUG RELOAD', description: '调试/性能测试', examples: ['DEBUG OBJECT user:1', 'DEBUG SLEEP 2'] },
  ],
}
