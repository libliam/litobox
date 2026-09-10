/**
 * 速查表通用类型定义
 * 新增速查表只需创建一个实现 CheatSheetData 的对象并在 index.ts 中注册
 */

export interface CheatSheetColumn {
  /** 字段 key，对应 row 中的属性名 */
  key: string
  /** 表头显示名 */
  label: string
  /** 列宽，如 '100px' */
  width?: string
  /** 是否可点击复制 */
  copyable?: boolean
}

export interface CheatSheetData {
  /** 唯一标识 */
  id: string
  /** Tab 显示名 */
  name: string
  /** 描述（可选） */
  description?: string
  /** 列定义 */
  columns: CheatSheetColumn[]
  /** 数据行 */
  rows: Record<string, string>[]
}

// HTTP 状态码
const httpStatusCodes: CheatSheetData = {
  id: 'httpStatus',
  name: 'HTTP 状态码',
  description: 'HTTP 响应状态码速查，点击状态码可复制',
  columns: [
    { key: 'code', label: '状态码', width: '100px', copyable: true },
    { key: 'name', label: '名称', width: '160px' },
    { key: 'category', label: '分类', width: '100px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // 1xx 信息
    { code: '100', name: 'Continue', category: '信息', description: '服务器已收到请求头，客户端应继续发送请求体' },
    { code: '101', name: 'Switching Protocols', category: '信息', description: '服务器同意切换协议（如 WebSocket 升级）' },
    { code: '102', name: 'Processing', category: '信息', description: '服务器已接受请求，正在处理，暂无响应' },
    { code: '103', name: 'Early Hints', category: '信息', description: '用于在最终响应前预加载资源' },
    // 2xx 成功
    { code: '200', name: 'OK', category: '成功', description: '请求成功' },
    { code: '201', name: 'Created', category: '成功', description: '资源创建成功' },
    { code: '202', name: 'Accepted', category: '成功', description: '请求已接受，正在处理' },
    { code: '203', name: 'Non-Authoritative Information', category: '成功', description: '返回信息来自第三方副本' },
    { code: '204', name: 'No Content', category: '成功', description: '请求成功，无返回内容' },
    { code: '205', name: 'Reset Content', category: '成功', description: '请求成功，客户端应重置视图' },
    { code: '206', name: 'Partial Content', category: '成功', description: '部分内容返回（Range 请求）' },
    { code: '207', name: 'Multi-Status', category: '成功', description: '多状态响应（WebDAV）' },
    { code: '208', name: 'Already Reported', category: '成功', description: '已在之前的响应中报告过' },
    { code: '226', name: 'IM Used', category: '成功', description: '服务器已满足对资源的请求' },
    // 3xx 重定向
    { code: '300', name: 'Multiple Choices', category: '重定向', description: '存在多个可选资源' },
    { code: '301', name: 'Moved Permanently', category: '重定向', description: '资源已永久移动到新 URL' },
    { code: '302', name: 'Found', category: '重定向', description: '资源临时移动到新 URL' },
    { code: '303', name: 'See Other', category: '重定向', description: '应使用 GET 方法请求新 URL' },
    { code: '304', name: 'Not Modified', category: '重定向', description: '资源未修改，可使用缓存' },
    { code: '305', name: 'Use Proxy', category: '重定向', description: '需通过代理访问' },
    { code: '307', name: 'Temporary Redirect', category: '重定向', description: '临时重定向，保持请求方法' },
    { code: '308', name: 'Permanent Redirect', category: '重定向', description: '永久重定向，保持请求方法' },
    // 4xx 客户端错误
    { code: '400', name: 'Bad Request', category: '客户端错误', description: '请求参数有误' },
    { code: '401', name: 'Unauthorized', category: '客户端错误', description: '未认证，需登录' },
    { code: '402', name: 'Payment Required', category: '客户端错误', description: '需付费（预留）' },
    { code: '403', name: 'Forbidden', category: '客户端错误', description: '已认证但无权限访问' },
    { code: '404', name: 'Not Found', category: '客户端错误', description: '资源不存在' },
    { code: '405', name: 'Method Not Allowed', category: '客户端错误', description: '请求方法不被允许' },
    { code: '406', name: 'Not Acceptable', category: '客户端错误', description: '无法生成客户端可接受的响应' },
    { code: '407', name: 'Proxy Authentication Required', category: '客户端错误', description: '需代理认证' },
    { code: '408', name: 'Request Timeout', category: '客户端错误', description: '请求超时' },
    { code: '409', name: 'Conflict', category: '客户端错误', description: '请求与当前资源状态冲突' },
    { code: '410', name: 'Gone', category: '客户端错误', description: '资源已永久删除' },
    { code: '411', name: 'Length Required', category: '客户端错误', description: '缺少 Content-Length 头' },
    { code: '412', name: 'Precondition Failed', category: '客户端错误', description: '前置条件校验失败' },
    { code: '413', name: 'Payload Too Large', category: '客户端错误', description: '请求体过大' },
    { code: '414', name: 'URI Too Long', category: '客户端错误', description: '请求 URI 过长' },
    { code: '415', name: 'Unsupported Media Type', category: '客户端错误', description: '不支持的媒体类型' },
    { code: '416', name: 'Range Not Satisfiable', category: '客户端错误', description: 'Range 范围无效' },
    { code: '417', name: 'Expectation Failed', category: '客户端错误', description: 'Expect 头要求无法满足' },
    { code: '418', name: "I'm a Teapot", category: '客户端错误', description: '服务器拒绝煮咖啡（愚人节玩笑）' },
    { code: '421', name: 'Misdirected Request', category: '客户端错误', description: '请求被定向到无法产生响应的服务器' },
    { code: '422', name: 'Unprocessable Entity', category: '客户端错误', description: '语义错误，无法处理' },
    { code: '423', name: 'Locked', category: '客户端错误', description: '资源被锁定（WebDAV）' },
    { code: '424', name: 'Failed Dependency', category: '客户端错误', description: '依赖的请求失败' },
    { code: '425', name: 'Too Early', category: '客户端错误', description: '请求发送过早' },
    { code: '426', name: 'Upgrade Required', category: '客户端错误', description: '需升级协议' },
    { code: '428', name: 'Precondition Required', category: '客户端错误', description: '需前置条件头' },
    { code: '429', name: 'Too Many Requests', category: '客户端错误', description: '请求过于频繁（限流）' },
    { code: '431', name: 'Request Header Fields Too Large', category: '客户端错误', description: '请求头字段过大' },
    { code: '451', name: 'Unavailable For Legal Reasons', category: '客户端错误', description: '因法律原因不可用' },
    // 5xx 服务器错误
    { code: '500', name: 'Internal Server Error', category: '服务器错误', description: '服务器内部错误' },
    { code: '501', name: 'Not Implemented', category: '服务器错误', description: '服务器不支持该功能' },
    { code: '502', name: 'Bad Gateway', category: '服务器错误', description: '网关/代理收到无效响应' },
    { code: '503', name: 'Service Unavailable', category: '服务器错误', description: '服务暂时不可用' },
    { code: '504', name: 'Gateway Timeout', category: '服务器错误', description: '网关/代理超时' },
    { code: '505', name: 'HTTP Version Not Supported', category: '服务器错误', description: '不支持该 HTTP 版本' },
    { code: '506', name: 'Variant Also Negotiates', category: '服务器错误', description: '内容协商配置错误' },
    { code: '507', name: 'Insufficient Storage', category: '服务器错误', description: '存储空间不足（WebDAV）' },
    { code: '508', name: 'Loop Detected', category: '服务器错误', description: '检测到循环' },
    { code: '510', name: 'Not Extended', category: '服务器错误', description: '需要进一步扩展请求' },
    { code: '511', name: 'Network Authentication Required', category: '服务器错误', description: '需网络认证' },
  ],
}

// MIME 类型
const mimeTypes: CheatSheetData = {
  id: 'mime',
  name: 'MIME 类型',
  description: '常见文件扩展名与 MIME 类型对照，点击扩展名或 MIME 可复制',
  columns: [
    { key: 'ext', label: '扩展名', width: '100px', copyable: true },
    { key: 'mime', label: 'MIME 类型', width: '220px', copyable: true },
    { key: 'category', label: '分类', width: '100px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    { ext: '.html', mime: 'text/html', category: '文本', description: 'HTML 文档' },
    { ext: '.htm', mime: 'text/html', category: '文本', description: 'HTML 文档' },
    { ext: '.css', mime: 'text/css', category: '文本', description: 'CSS 样式表' },
    { ext: '.js', mime: 'text/javascript', category: '文本', description: 'JavaScript 脚本' },
    { ext: '.json', mime: 'application/json', category: '应用', description: 'JSON 数据' },
    { ext: '.xml', mime: 'application/xml', category: '应用', description: 'XML 数据' },
    { ext: '.txt', mime: 'text/plain', category: '文本', description: '纯文本' },
    { ext: '.md', mime: 'text/markdown', category: '文本', description: 'Markdown 文档' },
    { ext: '.csv', mime: 'text/csv', category: '文本', description: 'CSV 表格' },
    { ext: '.yaml', mime: 'application/x-yaml', category: '应用', description: 'YAML 配置' },
    { ext: '.yml', mime: 'application/x-yaml', category: '应用', description: 'YAML 配置' },
    { ext: '.toml', mime: 'application/toml', category: '应用', description: 'TOML 配置' },
    { ext: '.pdf', mime: 'application/pdf', category: '应用', description: 'PDF 文档' },
    { ext: '.zip', mime: 'application/zip', category: '应用', description: 'ZIP 压缩包' },
    { ext: '.gz', mime: 'application/gzip', category: '应用', description: 'Gzip 压缩' },
    { ext: '.tar', mime: 'application/x-tar', category: '应用', description: 'TAR 归档' },
    { ext: '.7z', mime: 'application/x-7z-compressed', category: '应用', description: '7-Zip 压缩包' },
    { ext: '.rar', mime: 'application/x-rar-compressed', category: '应用', description: 'RAR 压缩包' },
    { ext: '.png', mime: 'image/png', category: '图片', description: 'PNG 图片' },
    { ext: '.jpg', mime: 'image/jpeg', category: '图片', description: 'JPEG 图片' },
    { ext: '.jpeg', mime: 'image/jpeg', category: '图片', description: 'JPEG 图片' },
    { ext: '.gif', mime: 'image/gif', category: '图片', description: 'GIF 动图' },
    { ext: '.svg', mime: 'image/svg+xml', category: '图片', description: 'SVG 矢量图' },
    { ext: '.webp', mime: 'image/webp', category: '图片', description: 'WebP 图片' },
    { ext: '.ico', mime: 'image/x-icon', category: '图片', description: '图标文件' },
    { ext: '.bmp', mime: 'image/bmp', category: '图片', description: 'BMP 位图' },
    { ext: '.avif', mime: 'image/avif', category: '图片', description: 'AVIF 图片' },
    { ext: '.mp3', mime: 'audio/mpeg', category: '音频', description: 'MP3 音频' },
    { ext: '.wav', mime: 'audio/wav', category: '音频', description: 'WAV 音频' },
    { ext: '.ogg', mime: 'audio/ogg', category: '音频', description: 'OGG 音频' },
    { ext: '.m4a', mime: 'audio/mp4', category: '音频', description: 'M4A 音频' },
    { ext: '.flac', mime: 'audio/flac', category: '音频', description: 'FLAC 无损音频' },
    { ext: '.mp4', mime: 'video/mp4', category: '视频', description: 'MP4 视频' },
    { ext: '.webm', mime: 'video/webm', category: '视频', description: 'WebM 视频' },
    { ext: '.avi', mime: 'video/x-msvideo', category: '视频', description: 'AVI 视频' },
    { ext: '.mov', mime: 'video/quicktime', category: '视频', description: 'QuickTime 视频' },
    { ext: '.mkv', mime: 'video/x-matroska', category: '视频', description: 'Matroska 视频' },
    { ext: '.woff', mime: 'font/woff', category: '字体', description: 'Web 字体' },
    { ext: '.woff2', mime: 'font/woff2', category: '字体', description: 'Web 字体 2.0' },
    { ext: '.ttf', mime: 'font/ttf', category: '字体', description: 'TrueType 字体' },
    { ext: '.otf', mime: 'font/otf', category: '字体', description: 'OpenType 字体' },
    { ext: '.eot', mime: 'application/vnd.ms-fontobject', category: '字体', description: '嵌入式字体' },
    { ext: '.sql', mime: 'application/sql', category: '应用', description: 'SQL 脚本' },
    { ext: '.sh', mime: 'application/x-sh', category: '应用', description: 'Shell 脚本' },
    { ext: '.bat', mime: 'application/x-msdownload', category: '应用', description: '批处理脚本' },
    { ext: '.exe', mime: 'application/octet-stream', category: '应用', description: '可执行文件' },
    { ext: '.wasm', mime: 'application/wasm', category: '应用', description: 'WebAssembly 二进制' },
    { ext: '.form', mime: 'application/x-www-form-urlencoded', category: '应用', description: '表单编码' },
    { ext: '.multipart', mime: 'multipart/form-data', category: '应用', description: '多部分表单' },
  ],
}

// 所有速查表
export const cheatSheets: CheatSheetData[] = [httpStatusCodes, mimeTypes]
