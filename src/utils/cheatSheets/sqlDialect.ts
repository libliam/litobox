import type { CheatSheetData } from './index'

export const sqlDialectCheatsheet: CheatSheetData = {
  id: 'sqlDialect',
  name: 'PostgreSQL / MySQL 特有语法',
  description: '通用 SQL 已有一份，这里专列 PG / MySQL 各自独有的高频语法（对照）',
  columns: [
    { key: 'pg', label: 'PostgreSQL', width: '260px', copyable: true },
    { key: 'mysql', label: 'MySQL 8+', width: '260px', copyable: true },
    { key: 'description', label: '说明 / 差异' },
  ],
  rows: [
    // === 类型系统 ===
    { pg: 'BOOLEAN / SMALLSERIAL / SERIAL / BIGSERIAL / UUID / JSONB / ARRAY[1,2,3]', mysql: 'BOOL/TINYINT(1) / TINYINT / INT AUTO_INCREMENT / BIGINT AUTO_INCREMENT / CHAR(36) / JSON / JSON', description: '类型差异：PG 的 SERIAL 是自动递增+序列，MySQL 用 AUTO_INCREMENT；PG 原生 UUID 类型；PG 有数组类型，MySQL 没有；PG JSONB 支持 GIN 索引' },
    { pg: 'NUMERIC / DECIMAL / REAL / DOUBLE PRECISION / MONEY', mysql: 'DECIMAL / FLOAT / DOUBLE', description: '数值类型：PG 有 MONEY 带货币符号；PG NUMERIC 精度最高' },
    { pg: 'TIMESTAMP / TIMESTAMPTZ / DATE / TIME / INTERVAL', mysql: 'DATETIME / TIMESTAMP / DATE / TIME / YEAR', description: '时间类型：PG 有 INTERVAL 间隔类型 + TIMESTAMPTZ 时区感知；MySQL TIMESTAMP 会自动转换为 UTC 存储' },
    // === 自增ID ===
    { pg: 'CREATE TABLE t (id SERIAL PRIMARY KEY) / GENERATED AS IDENTITY', mysql: 'CREATE TABLE t (id INT AUTO_INCREMENT PRIMARY KEY)', description: '自增：PG 8.4+ 推荐 GENERATED ALWAYS AS IDENTITY；MySQL AUTO_INCREMENT 会根据指定步长自动递增' },
    { pg: 'INSERT INTO t ... RETURNING id', mysql: 'SELECT LAST_INSERT_ID()', description: '获取刚插入 ID：PG 支持 RETURNING 子句（一次多行全返回）；MySQL 用 LAST_INSERT_ID()' },
    // === 字符串 ===
    { pg: '|| 拼接 / || \' \' || / CONCAT()', mysql: 'CONCAT(a, " ", b) / CONCAT_WS(" ", a, b)', description: '字符串拼接：PG 用 || 更 SQL 标准；MySQL 也有 CONCAT' },
    { pg: 'LIKE / ILIKE 忽略大小写 / % 通配', mysql: 'LIKE 默认不区分大小写(受collation影响) / RLIKE 正则', description: '大小写：MySQL 默认 collation 一般_ci 所以 LIKE 不区分；PG 用 ILIKE 才忽略' },
    { pg: 'SUBSTRING / LEFT / RIGHT', mysql: 'SUBSTRING / LEFT / RIGHT', description: '基本一致；PG 还有 string_splice / REPLACE' },
    // === JSON ===
    { pg: 'data->>\'key\' 取文本 / data->\'key\' 取JSON / data @> \'{"a":1}\'', mysql: 'JSON_EXTRACT(data, \'$.key\') / data->"$.key" 简写 / JSON_CONTAINS', description: 'JSON 支持：PG JSONB 更完整（支持索引、JSON 操作符 @> <@ ? |）；MySQL JSON 功能略弱' },
    // === 日期 ===
    { pg: 'CURRENT_TIMESTAMP / NOW() / EXTRACT(EPOCH FROM ts) / DATE_TRUNC(\'day\', ts)', mysql: 'NOW() / UNIX_TIMESTAMP() / FROM_UNIXTIME() / DATE_FORMAT() / DATE_SUB(NOW(), INTERVAL 7 DAY)', description: '日期函数：PG 有 DATE_TRUNC 截断粒度；MySQL 有 INTERVAL + DATE_ADD/DATE_SUB' },
    { pg: 'INTERVAL \'1 day\' / \'2 hours\' / \'30 minutes\'', mysql: 'INTERVAL 1 DAY / INTERVAL 2 HOUR', description: '间隔写法：PG 字符串+单位；MySQL 数值+大写单位' },
    // === 唯一约束 ===
    { pg: 'UNIQUE(col) / UNIQUE(a,b)', mysql: 'UNIQUE KEY 索引名(col) / UNIQUE(col)', description: '基本一致；PG 8+ 支持部分唯一索引' },
    // === 窗口函数 ===
    { pg: 'ROW_NUMBER() / RANK() / LAG() / LEAD() / DENSE_RANK()', mysql: '同 PG（MySQL 8.0+ 也支持窗口函数）', description: '窗口函数：PG 8.4+ 引入；MySQL 5.7 不支持，8.0+ 支持得比较全' },
    // === 分页 ===
    { pg: 'LIMIT 10 OFFSET 20', mysql: 'LIMIT 20,10 或 LIMIT 10 OFFSET 20', description: 'PG LIMIT/OFFSET 是 SQL 标准；MySQL 支持两者；OFFSET 大了都慢（应该用游标）' },
    // === 递归CTE ===
    { pg: 'WITH RECURSIVE cte AS (SELECT ... UNION ALL SELECT ... FROM cte)', mysql: 'MySQL 8.0+ 同样支持 WITH RECURSIVE', description: '递归 CTE：PG 和 MySQL 8.0+ 都支持；MySQL 5.7 不支持' },
    // === 索引 ===
    { pg: 'CREATE INDEX CONCURRENTLY / CREATE INDEX USING GIN(data->\'key\') / CREATE INDEX ON t ((LOWER(col)))', mysql: 'CREATE INDEX / FULLTEXT / SPATIAL / 前缀索引 INDEX(col(10))', description: 'PG 支持在线建索引（CONCURRENTLY 不锁表）、表达式索引、GIN 倒排索引；MySQL 支持全文/空间索引 + 前缀索引' },
    { pg: 'DROP INDEX CONCURRENTLY', mysql: 'DROP INDEX idx ON t', description: 'PG 删除也能在线；MySQL 会锁表' },
    // === 事务隔离级别 ===
    { pg: 'SET TRANSACTION ISOLATION LEVEL READ COMMITTED / REPEATABLE READ / SERIALIZABLE', mysql: 'SET TRANSACTION ISOLATION LEVEL ... / InnoDB 默认 REPEATABLE READ', description: '隔离级别：PG 默认 READ COMMITTED；MySQL InnoDB 默认 REPEATABLE READ（幻读用 Next-Key Lock 解决）' },
    // === 锁 ===
    { pg: 'SELECT ... FOR UPDATE [OF table] / SKIP LOCKED / NOWAIT', mysql: 'SELECT ... FOR UPDATE / FOR SHARE (8.0) / LOCK IN SHARE MODE (旧)', description: 'PG 锁语法更完整：SKIP LOCKED 跳过被锁行、FOR UPDATE OF 指定表；MySQL 用 LOCK IN SHARE MODE 代替 FOR SHARE' },
    { pg: 'LOCK TABLE table IN MODE SHARE/EXCLUSIVE', mysql: 'LOCK TABLE t READ / WRITE', description: '表锁：PG 模式名更多；MySQL 简单 READ/WRITE' },
    // === 序列 ===
    { pg: 'CREATE SEQUENCE seq START 1 INCREMENT 1 / NEXTVAL(\'seq\')', mysql: '无原生序列（用 AUTO_INCREMENT 模拟）', description: '序列：PG 原生支持，多列可共享同一个序列；MySQL 靠 AUTO_INCREMENT 或额外表模拟' },
    // === 扩展 ===
    { pg: 'CREATE EXTENSION pgcrypto; CREATE EXTENSION postgis; CREATE EXTENSION uuid-ossp', mysql: 'PLUGIN / 插件机制不同', description: 'PG 扩展系统非常强大；MySQL 插件机制有限' },
    // === 其他 ===
    { pg: 'COPY t FROM \'/file.csv\' / TO \'/out.csv\' WITH (FORMAT CSV, HEADER)', mysql: 'LOAD DATA INFILE \'/file.csv\' INTO TABLE t FIELDS TERMINATED BY \',\' IGNORE 1 LINES', description: 'CSV 批量导入导出：PG COPY 功能完整（服务器文件）；MySQL LOAD DATA INFILE 需要权限/secure_file_priv' },
    { pg: 'INSERT ... ON CONFLICT DO NOTHING/UPDATE', mysql: 'INSERT ... ON DUPLICATE KEY UPDATE', description: 'Upsert：PG 8.4+ 用标准 ON CONFLICT；MySQL 用 ON DUPLICATE KEY UPDATE 同义但非标准' },
    { pg: 'RETURNING / RETURNING * / RETURNING col1, col2', mysql: 'SELECT LAST_INSERT_ID()', description: 'RETURNING 子句：PG 的 INSERT/UPDATE/DELETE 都能 RETURNING；MySQL 只有 LAST_INSERT_ID() 有限替代' },
    { pg: 'CAST(x AS type) / ::type 简写', mysql: 'CAST(x AS type) / CONVERT(x, type)', description: '类型转换：PG 支持 :: 简写 `col::int`；MySQL 只有 CAST/CONVERT' },
    { pg: 'GREATEST(a,b,c) / LEAST(a,b,c)', mysql: '同（MySQL 5.7+）', description: '比较多个值取最大/最小：两者都支持' },
    { pg: 'UNSIGNED 无（PG 不支持 UNSIGNED 语法）', mysql: 'INT UNSIGNED / BIGINT UNSIGNED', description: 'PG 不支持 UNSIGNED，只能用 CHECK 约束或 domain 模拟' },
  ],
}
