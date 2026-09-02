import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as db from '@/utils/dbClient'

export interface ToolboxConfig {
  theme: 'auto' | 'dark' | 'light'
  editorTheme: string
  jsonIndent: 2 | 4
  hotkey: string
  lastTool: string
  favorites: string[]
  shortcuts: Record<string, string>
}

export interface HistoryRecord {
  id?: number
  tool: string
  action: string
  timestamp: string
  inputPreview: string
  outputPreview: string
  inputFull?: string
  outputFull?: string
  options?: Record<string, any>
}

export interface HistoryRestoreState {
  tool: string
  action?: string
  input: string
  output: string
  options: Record<string, any>
  timestamp: string
}

export interface ToolItem {
  id: string
  name: string
  icon: string
  iconSvg: string
  description: string
  keywords: string[]
  category?: string
}

export const TOOL_LIST: ToolItem[] = [
  { id: 'home', name: '首页', icon: '🏠', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9.5L12 3l9 6.5V20a1 1 0 01-1 1H4a1 1 0 01-1-1V9.5z"/><polyline points="9 21 9 14 15 14 15 21"/></svg>`, description: '搜索工具、最近使用、常用推荐', keywords: ['首页', '搜索', '主页'] },
  { id: 'json', name: 'JSON工具', icon: '{ }', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H6a2 2 0 00-2 2v4a2 2 0 01-2 2 2 2 0 012 2v4a2 2 0 002 2h2"/><path d="M16 3h2a2 2 0 012 2v4a2 2 0 002 2 2 2 0 00-2 2v4a2 2 0 01-2 2h-2"/></svg>`, description: 'JSON格式化、压缩、校验、JSON5 兼容解析（注释/尾逗号）、数据统计、Key 筛选', keywords: ['json', '格式化', '压缩', '校验', 'json5', '注释', '统计', 'key筛选'], category: 'text' },
  { id: 'string', name: '字符串工具', icon: 'Aa', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M12 4v16"/><path d="M8 20h8"/><path d="M17 14l-3-3 3-3"/></svg>`, description: '空格处理、拼接分割、大小写转换、文本清理、批量处理', keywords: ['字符串', '空格', '大小写', '文本', '批量'], category: 'text' },
  { id: 'pinyin', name: '拼音工具', icon: '拼', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M8 8h8M8 12h8M8 16h5"/></svg>`, description: '中文转拼音，全拼带声调/无声调、首字母、驼峰、全小写，多音字候选，批量处理', keywords: ['拼音', 'pinyin', '中文', '声调', '多音字', '首字母', '驼峰'], category: 'text' },
  { id: 'markdown', name: 'Markdown', icon: 'MD', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M7 15V9l2.5 3L12 9v6"/><path d="M17 9v6"/><path d="M15 12h4"/></svg>`, description: 'Markdown 实时预览、HTML 互转、导出、统计', keywords: ['markdown', 'md', '预览', 'html', '转换', '统计'], category: 'text' },
  { id: 'wordCount', name: '字数统计', icon: 'ABC', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M16 16l-2-4-2 4"/><path d="M14 13h-4"/></svg>`, description: '字符数、单词数、行数、阅读时间估算', keywords: ['字数', '统计', '字符', '单词', '行数'], category: 'text' },
  { id: 'diff', name: '文本对比', icon: '≠', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h16M4 12h16M4 18h16"/><path d="M9 3v18M15 3v18"/></svg>`, description: '文本/代码对比，支持行级和字符级差异高亮', keywords: ['对比', 'diff', '差异', '代码对比'], category: 'text' },
  { id: 'dedup', name: '文本去重', icon: '≡', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h16M4 12h12M4 18h8"/><circle cx="20" cy="12" r="1" fill="currentColor"/><circle cx="16" cy="18" r="1" fill="currentColor"/></svg>`, description: '按行去重，支持首次/末次保留', keywords: ['去重', '重复', 'dedup', '清理'], category: 'text' },
  { id: 'regex', name: '正则测试', icon: '.*', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>`, description: '正则表达式测试、匹配、替换', keywords: ['正则', 'regex', '匹配', '替换'], category: 'text' },
  { id: 'note', name: '文本编辑器', icon: '📝', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>`, description: '草稿本/便签，支持语法高亮、查找替换、自动保存', keywords: ['文本编辑器', '草稿', '笔记', 'notepad'], category: 'text' },
  { id: 'time', name: '时间工具', icon: '🕐', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>`, description: '时间戳转换、日期计算、相对时间', keywords: ['时间', 'timestamp', '日期', '转换'], category: 'dev' },
  { id: 'encode', name: '编码转换', icon: 'En', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M16 16l-2-4-2 4"/></svg>`, description: 'URL、Base64、HTML实体、Unicode 编解码，时间戳转换，人民币大写（数字→中文大写），多行批量转换', keywords: ['编码', 'base64', 'url', 'unicode', 'html实体', '时间戳', '人民币大写', '金额大写', '大写', '批量'], category: 'dev' },
  { id: 'hash', name: '加密解密', icon: '#', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9h16M4 15h16"/><path d="M10 3l-2 18M16 3l-2 18"/></svg>`, description: 'MD5/SHA 哈希、AES/DES 加密解密、HMAC、文件哈希', keywords: ['hash', 'md5', 'sha', '哈希', '加密', '解密', 'aes', 'des', 'hmac', '文件哈希'], category: 'dev' },
  { id: 'jwt', name: 'JWT解析', icon: 'JWT', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>`, description: '解析 JWT token，查看 Header/Payload', keywords: ['jwt', 'token', '解析', '认证'], category: 'dev' },
  { id: 'cron', name: 'Cron表达式', icon: '⏰', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>`, description: '可视化生成 Cron 表达式，支持5/6字段格式', keywords: ['cron', '定时', '调度', '表达式'], category: 'dev' },
  { id: 'url', name: 'URL工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>`, description: 'URL解析、编码、参数提取', keywords: ['url', '链接', '解析', '参数'], category: 'dev' },
  { id: 'baseConverter', name: '进制转换', icon: '0x', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>`, description: '二进制、八进制、十进制、十六进制互转', keywords: ['进制', '转换', 'binary', 'hex', 'octal'], category: 'dev' },
  { id: 'ipSubnet', name: 'IP子网', icon: 'IP', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M3 15h18M9 3v18M15 3v18"/></svg>`, description: 'IPv4 子网计算、子网划分、IP范围合并、IP↔整数互转', keywords: ['ip', '子网', 'subnet', 'cidr', '掩码', '网络', '子网划分', 'ip转整数'], category: 'dev' },
  { id: 'uuid', name: 'ID生成器', icon: 'ID', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>`, description: 'UUID v1/v4/v5/v7、雪花ID、ObjectId、NanoID、ULID、KSUID、CUID2、XID、自增序列等 12 种 ID 生成', keywords: ['uuid', 'guid', '生成', '唯一标识', '雪花', 'objectid', 'id', '自增', 'nanoid', 'ulid', 'ksuid', 'cuid2', 'xid', 'v7', 'v5'], category: 'dev' },
  { id: 'color', name: '颜色工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="8" cy="9" r="1.5" fill="currentColor"/><circle cx="12" cy="7" r="1.5" fill="currentColor"/><circle cx="16" cy="9" r="1.5" fill="currentColor"/><circle cx="8" cy="13" r="1.5" fill="currentColor"/><circle cx="16" cy="13" r="1.5" fill="currentColor"/><circle cx="12" cy="17" r="1.5" fill="currentColor"/></svg>`, description: '颜色选择器、格式转换、色板生成、对比度检查、渐变生成', keywords: ['颜色', 'color', '拾色器', '色板', '对比度', '渐变', 'hex', 'rgb', 'hsl'], category: 'dev' },
  { id: 'css', name: 'CSS工具', icon: 'CSS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3h12l-3 9H9z"/><path d="M9 12l-2 9h10l-2-9"/></svg>`, description: '颜色转换、单位换算、CSS压缩/格式化', keywords: ['css', '颜色', '单位', '压缩'], category: 'dev' },
  { id: 'js', name: 'JS工具', icon: 'JS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M10 12l2 2 4-4"/></svg>`, description: 'JS沙箱运行、格式化、压缩、JSON提取', keywords: ['js', 'javascript', '沙箱', '格式化', '压缩'], category: 'dev' },
  { id: 'sql', name: 'SQL工具', icon: 'SQL', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M4 9h16"/><path d="M9 4v16"/></svg>`, description: 'SQL格式化、压缩、校验、JSON数组转Insert、MyBatis日志解析', keywords: ['sql', '格式化', '压缩', '校验', 'insert', 'mybatis', '日志解析', '转换'], category: 'dev' },
  { id: 'xmlYaml', name: 'XML/YAML', icon: 'XML', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H6a2 2 0 00-2 2v4"/><path d="M16 3h2a2 2 0 012 2v4"/><path d="M14 17l-3 3-3-3"/><path d="M4 21h16"/></svg>`, description: 'XML/YAML格式化、校验、JSON互转，JSON/YAML/TOML/INI/Properties 配置互转', keywords: ['xml', 'yaml', '格式化', '校验', '转换', 'toml', 'ini', 'properties', '配置互转'], category: 'dev' },
  { id: 'csv', name: 'CSV工具', icon: 'CSV', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M4 9h16"/><path d="M9 4v16"/></svg>`, description: 'CSV解析、表格预览、导出JSON/SQL', keywords: ['csv', '表格', '解析', '导出'], category: 'dev' },
  { id: 'excelTool', name: 'Excel工具', icon: 'XL', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M3 15h18M9 3v18M15 3v18"/><path d="M8 12l4 4M12 12l-4 4"/></svg>`, description: 'Excel 多Sheet浏览、文件合并、数据清洗、导出CSV/JSON/Markdown/SQL', keywords: ['excel', 'xlsx', '表格', '合并', '清洗', '多sheet', '电子表格'], category: 'dev' },
  { id: 'schemaTool', name: 'JSON Schema', icon: 'JS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v18"/><path d="M8 8l-4 4 4 4"/><path d="M16 8l4 4-4 4"/></svg>`, description: 'JSON Schema 校验、Mock 数据生成、导出 TypeScript 接口', keywords: ['schema', 'json', 'json校验', '校验', 'mock', 'typescript', '接口', 'draft-07', '数据校验'], category: 'dev' },
  { id: 'openApi', name: 'OpenAPI解析', icon: 'API', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7h13M16 7l-3-3M16 7l-3 3"/><path d="M21 17H8M8 17l3-3M8 17l3 3"/><rect x="3" y="15" width="4" height="4" rx="1"/><circle cx="19" cy="6" r="1.5"/></svg>`, description: '解析 OpenAPI 文档：接口清单与详情、curl/fetch 请求示例、Mock 数据、TS 类型导出', keywords: ['openapi', 'swagger', '接口', 'api', 'yaml', '解析', 'mock', 'typescript', '请求示例', 'curl', 'fetch', '接口文档'], category: 'dev' },
  { id: 'templateTool', name: '模板渲染', icon: 'Tb', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M7 7h10v10H7z"/><path d="M10 4v16"/><path d="M6 10l-3 2 3 2"/><path d="M18 10l3 2-3 2"/></svg>`, description: 'Handlebars 模板实时预览，支持变量、循环、条件渲染，快捷插入标签', keywords: ['模板', 'template', 'handlebars', 'mustache', '渲染', '预览', 'each', 'if'], category: 'dev' },
  { id: 'gitStats', name: 'Git统计', icon: 'Git', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="12" r="3"/><path d="M6 9v6"/><path d="M18 12H9"/><path d="M9 9c3 1 6 0 9 3"/><path d="M9 15c3-1 6 0 9-3"/></svg>`, description: 'Git 仓库提交统计：贡献者排行、提交趋势、文件改动 Top', keywords: ['git', 'github', '仓库', '统计', '提交', 'commit', '贡献者', '趋势', '代码量'], category: 'dev' },
  { id: 'codeFormatter', name: '代码格式化', icon: 'fmt', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h16M4 12h16M4 18h16"/><path d="M9 3l-2 6 4 3-4 3 2 6"/><path d="M16 3l2 6-4 3 4 3-2 6"/></svg>`, description: '多语言代码格式化：JS/TS/JSON/CSS/HTML/Vue/Markdown/YAML，基于 Prettier', keywords: ['格式化', 'format', 'prettier', '代码', '美化', 'javascript', 'typescript', 'css', 'html', 'markdown', 'yaml', 'vue', '缩进'], category: 'dev' },
  { id: 'pdf', name: 'PDF工具', icon: 'PDF', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M9 15v-2a2 2 0 014 0v2"/><path d="M9 13h4"/></svg>`, description: 'PDF转图片、图片转PDF、文本提取、转Markdown、合并拆分、压缩、提取内嵌图片、加水印', keywords: ['pdf', '转换', '合并', '拆分', '提取', '压缩', 'markdown', '水印', '图片'], category: 'utility' },
  { id: 'curl', name: 'Curl构建器', icon: 'curl', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/><rect x="2" y="3" width="20" height="14" rx="2"/></svg>`, description: '根据参数实时生成 curl 命令，支持请求头、Basic/Bearer 认证、JSON/Form/原始请求体、Cookie、超时', keywords: ['curl', '请求', '命令', '构建', 'http', 'api', '运维', 'bash', 'command'], category: 'dev' },
  { id: 'http', name: 'HTTP 请求', icon: '🌐', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v20M5 5h14l-7 7 7 7H5"/></svg>`, description: 'HTTP 请求测试，支持 GET/POST/PUT/DELETE，绕过 CORS 限制', keywords: ['http', '请求', 'api', 'get', 'post', 'put', 'delete', 'cors'], category: 'dev' },
  { id: 'staticServer', name: '本地静态服务器', icon: '🖥', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="4" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="6" rx="2"/><line x1="6" y1="8" x2="6.01" y2="8"/><line x1="6" y1="17" x2="6.01" y2="17"/><line x1="10" y1="8" x2="18" y2="8"/><line x1="10" y1="17" x2="18" y2="17"/></svg>`, description: '一键启动本地 HTTP 静态文件服务器，目录浏览/下载/上传/ZIP打包，支持局域网访问', keywords: ['静态服务器', 'http', 'server', '本地服务', '局域网', '文件分享', '上传', 'zip', '目录浏览'], category: 'dev' },
  { id: 'password', name: '密码工具', icon: '🔑', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/><circle cx="12" cy="16" r="1"/></svg>`, description: '随机密码生成、API Key 生成、密码强度检测', keywords: ['密码', 'password', '随机', '生成', '强度', 'api key', 'token'], category: 'security' },
  { id: 'calculator', name: '计算器', icon: '∑', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="2" width="16" height="20" rx="2"/><line x1="8" y1="6" x2="16" y2="6"/><line x1="8" y1="10" x2="16" y2="10"/><line x1="8" y1="14" x2="12" y2="14"/><line x1="14" y1="14" x2="16" y2="14"/><line x1="8" y1="18" x2="12" y2="18"/><line x1="14" y1="18" x2="16" y2="18"/></svg>`, description: '表达式计算、单位换算、日期计算、时间戳转换', keywords: ['计算器', '计算', '单位换算', '日期', '时间戳'], category: 'utility' },
  { id: 'qr', name: '二维码', icon: 'QR', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="8" height="8" rx="1"/><rect x="14" y="2" width="8" height="8" rx="1"/><rect x="2" y="14" width="8" height="8" rx="1"/><rect x="14" y="14" width="4" height="4"/><rect x="20" y="14" width="2" height="2"/><rect x="14" y="20" width="2" height="2"/><rect x="20" y="20" width="2" height="2"/></svg>`, description: '二维码生成与解码，支持文本/URL转二维码、图片解码', keywords: ['二维码', 'qr', 'qrcode', '生成', '解码', '扫码'], category: 'utility' },
  { id: 'barcode', name: '条形码', icon: 'BC', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="4" x2="4" y2="20"/><line x1="7" y1="4" x2="7" y2="20"/><line x1="9" y1="4" x2="9" y2="20"/><line x1="12" y1="4" x2="12" y2="20"/><line x1="14" y1="4" x2="14" y2="20"/><line x1="17" y1="4" x2="17" y2="20"/><line x1="20" y1="4" x2="20" y2="20"/></svg>`, description: '一维条形码生成，支持 CODE128/EAN13/CODE39/UPC 等 9 种格式，批量生成打包下载', keywords: ['条形码', 'barcode', 'ean13', 'code128', 'code39', 'upc', '商品码', '一维码'], category: 'utility' },
  { id: 'image', name: '图片工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>`, description: '批量压缩/格式转换、尺寸缩放、图片转Base64、加水印、拼图、图片增强、调色板提取', keywords: ['图片', '压缩', '缩放', 'base64', '拼接', '水印', '调色板', '格式转换', '增强', 'image'], category: 'utility' },
  { id: 'exif', name: '图片EXIF', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7h4l2-3h6l2 3h4v13H3z"/><circle cx="12" cy="13" r="3.5"/></svg>`, description: '查看图片 EXIF 信息（相机/时间/GPS 等），一键清除隐私元数据', keywords: ['exif', '图片', '元数据', 'gps', '相机', '拍摄时间', '隐私', '清除', '位置'], category: 'utility' },
  { id: 'imageCompare', name: '图片对比', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-3"/><circle cx="14" cy="8" r="3"/><path d="M3 16l4-4 5 5"/><path d="M19 13l2 2-2 2"/><path d="M15 17h6"/></svg>`, description: '两张图片像素级对比：滑动对比、差异高亮（可调容差）、半透明叠加，差异率统计', keywords: ['图片对比', '对比', 'diff', '差异', '像素', '比对', '还原', 'compare'], category: 'utility' },
  { id: 'svg', name: 'SVG工具', icon: 'SVG', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M4 8h16"/><path d="M8 4v16"/></svg>`, description: 'SVG 实时预览、优化压缩、转 PNG 栅格化', keywords: ['svg', '预览', '优化', '压缩', '转png', '矢量图'], category: 'utility' },
  { id: 'audioTool', name: '音频工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>`, description: '音频裁剪、格式转换，支持 MP3/WAV/M4A，波形可视化、实时预览', keywords: ['音频', '裁剪', 'mp3', 'wav', 'm4a', '波形', 'audio'], category: 'utility' },
  { id: 'videoTool', name: '视频工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2"/></svg>`, description: '视频裁剪/转码/音频提取/压缩/合并，支持 ffmpeg 增强', keywords: ['视频', '裁剪', '转码', '音频提取', '压缩', '合并', 'mp4', 'video', 'ffmpeg'], category: 'utility' },
  { id: 'mediaInfo', name: '媒体信息', icon: 'ℹ️', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`, description: '查看音视频文件的详细信息（编解码器、分辨率、比特率、元数据等）', keywords: ['媒体', '信息', 'ffprobe', '视频', '音频', 'metadata'], category: 'utility' },
  { id: 'iconGenerator', name: '图标生成', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><rect x="7" y="7" width="3" height="3" rx="0.5"/><rect x="14" y="7" width="3" height="3" rx="0.5"/><rect x="7" y="14" width="3" height="3" rx="0.5"/><rect x="14" y="14" width="3" height="3" rx="0.5"/></svg>`, description: '一张图生成多尺寸图标（favicon/icon set），支持 PNG/ICO 格式', keywords: ['图标', 'icon', 'favicon', 'ico', '生成', '尺寸'], category: 'utility' },
  { id: 'ocr', name: 'OCR识别', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>`, description: '图片文字识别、表格识别、Markdown 转换，支持 PDF 转图后识别', keywords: ['ocr', '文字识别', '图片', '识别', '表格', 'markdown', 'pdf', '文字提取'], category: 'utility' },
  { id: 'mockData', name: '随机数据', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/><circle cx="8" cy="8" r="1" fill="currentColor"/><circle cx="16" cy="8" r="1" fill="currentColor"/><circle cx="8" cy="16" r="1" fill="currentColor"/><circle cx="16" cy="16" r="1" fill="currentColor"/></svg>`, description: '姓名、身份证、手机号、邮箱、地址、银行卡、金额（人民币大写）等 25 种随机数据', keywords: ['随机', '假数据', 'mock', '测试数据', '姓名', '身份证', '手机', '邮箱', '地址', '银行卡', '金额', '大写', '公司', '经纬度', '用户名', '快递'], category: 'utility' },
  { id: 'fileprocessing', name: '文件处理', icon: '📁', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>`, description: '批量文本处理、文件编码转换', keywords: ['文件', '编码', '转换', '批量', '替换'], category: 'utility' },
  { id: 'zipTool', name: '压缩/解压', icon: '🗜️', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 8v8a2 2 0 01-1 1.73l-7 4a2 2 0 01-2 0l-7-4A2 2 0 013 16V8a2 2 0 011-1.73l7-4a2 2 0 012 0l7 4A2 2 0 0121 8z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>`, description: 'ZIP 压缩/解压，支持压缩级别、密码加密、勾选部分解压、防路径穿越', keywords: ['压缩', '解压', 'zip', '打包', '加密', '密码', '解压缩', 'archive'], category: 'utility' },
  { id: 'mermaid', name: 'Mermaid图表', icon: '📊', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 3v18h18"/><path d="M7 14l4-4 3 3 5-6"/></svg>`, description: 'Mermaid 图表渲染：流程图、时序图、甘特图、类图等，实时预览并导出 PNG/SVG', keywords: ['mermaid', '图表', '流程图', '时序图', '甘特图', '类图', '状态图', '思维导图', 'ER图', '渲染', 'uml'], category: 'utility' },
  { id: 'pomodoro', name: '番茄钟', icon: '🍅', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="14" r="7.5"/><path d="M12 7c0-3 2.5-4.5 4.5-5M12 7c0-3-2.5-4.5-4.5-5"/><path d="M12 7c1.2-1.2 2.8-1.6 4-1.4"/></svg>`, description: '番茄钟专注计时：专注+休息循环、置顶浮窗、全局快捷键、到点系统通知', keywords: ['番茄钟', '番茄', 'pomodoro', '专注', '计时', '倒计时', '休息', '提醒', '浮窗'], category: 'utility' },
  { id: 'fileRenamer', name: '文件重命名', icon: '✏️', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 114 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>`, description: '批量文件重命名，支持替换/正则/前后缀/序号模式', keywords: ['重命名', 'rename', '批量', '文件', '替换', '正则', '序号'], category: 'utility' },
  { id: 'batchReplace', name: '批量替换', icon: '⇄', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7h13M16 7l-3-3M16 7l-3 3"/><path d="M21 17H8M8 17l3-3M8 17l3 3"/></svg>`, description: '多文件批量内容搜索替换，支持正则，UTF-8/GBK 编码保留，自动备份', keywords: ['批量', '替换', 'replace', '正则', '多文件', '搜索替换', '全局替换', 'find', 'replacement'], category: 'utility' },
  { id: 'quickLaunch', name: '快速启动', icon: '⚡', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>`, description: '全盘文件名快速搜索，一键打开', keywords: ['快速启动', '文件搜索', 'Everything', '启动', '搜索文件', '打开', 'quick', 'launch'], category: 'utility' },
  { id: 'clipboard', name: '剪贴板', icon: '📋', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2"/><path d="M9 14l2 2 4-4"/></svg>`, description: '系统剪贴板历史记录', keywords: ['剪贴板', '复制', '历史', 'clipboard'] },
  { id: 'clipboardConvert', name: '剪贴板转换', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2"/><path d="M7 10l4 4 4-4"/><path d="M11 8v6"/></svg>`, description: '一键剪贴板内容格式转换：图片↔Base64、HTML→Markdown、表格↔CSV/JSON/Markdown、配置互转', keywords: ['剪贴板', '转换', 'clipboard', 'base64', 'markdown', 'html', '表格', 'csv'] },
  { id: 'snippet', name: '代码片段', icon: '<>', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`, description: '代码片段管理，支持分类、搜索、导入导出', keywords: ['代码', '片段', 'snippet', '管理', '收藏', '模板'], category: 'utility' },
  { id: 'history', name: '历史记录', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/><path d="M3 12a9 9 0 0115.36-6.36L21 3"/></svg>`, description: '查看和清空操作历史', keywords: ['历史', '记录', '操作'] },
  { id: 'workflow', name: '工作流', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h4l3 3h6l3-3h4"/><path d="M4 12h4l3 3h6l3-3h4"/><path d="M4 18h4l3 3h6l3-3h4"/><circle cx="4" cy="6" r="1" fill="currentColor"/><circle cx="20" cy="6" r="1" fill="currentColor"/><circle cx="4" cy="12" r="1" fill="currentColor"/><circle cx="20" cy="12" r="1" fill="currentColor"/><circle cx="4" cy="18" r="1" fill="currentColor"/><circle cx="20" cy="18" r="1" fill="currentColor"/></svg>`, description: '工作流编排，链式处理，变量池管理', keywords: ['工作流', '编排', '链式', '变量池'], category: 'utility' },
  { id: 'systemInfo', name: '系统信息', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>`, description: '查看操作系统、CPU、内存、磁盘信息', keywords: ['系统', 'cpu', '内存', '磁盘', 'system'], category: 'system' },
  { id: 'networkInfo', name: '网络信息', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/></svg>`, description: '查看网络接口、IP、连接、端口', keywords: ['网络', 'ip', 'mac', '端口', 'netstat'], category: 'system' },
  { id: 'processList', name: '进程列表', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>`, description: '查看运行中的进程及资源占用', keywords: ['进程', 'process', '任务管理器'], category: 'system' },
  { id: 'serviceList', name: '服务管理', icon: '⚙️', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>`, description: '查看和管理 Windows 系统服务，支持启动/停止/重启', keywords: ['服务', 'service', '启动', '停止', '重启'], category: 'system' },
  { id: 'hardwareInfo', name: '硬件外设', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>`, description: '查看GPU、显示器、音频设备', keywords: ['硬件', 'gpu', '显卡', '显示器', '音频'], category: 'system' },
  { id: 'softwareEnv', name: '软件环境', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>`, description: '已安装软件、环境变量、启动项', keywords: ['软件', '环境变量', '启动项', 'env'], category: 'system' },
  { id: 'sqliteViewer', name: 'SQLite查看器', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 5v6c0 1.66-4.03 3-9 3s-9-1.34-9-3V5"/><path d="M21 11v6c0 1.66-4.03 3-9 3s-9-1.34-9-3v-6"/></svg>`, description: '浏览本地SQLite数据库表结构和数据，执行SELECT查询，导出CSV', keywords: ['sqlite', '数据库', 'db', '查询', '查看', 'database'], category: 'dev' },
  { id: 'diskAnalyzer', name: '磁盘分析', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/><line x1="12" y1="2" x2="12" y2="6"/><line x1="12" y1="18" x2="12" y2="22"/><line x1="2" y1="12" x2="6" y2="12"/><line x1="18" y1="12" x2="22" y2="12"/></svg>`, description: '分析磁盘空间占用，查找大文件和重复文件', keywords: ['磁盘', '空间', '重复', '清理', 'disk', 'space', 'duplicate'], category: 'system' },
  { id: 'fileSearcher', name: '全文搜索', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>`, description: '按文件名或内容搜索，支持正则表达式，类似 Everything + grep', keywords: ['搜索', '全文', '文件名', '内容', 'grep', 'find', 'search'], category: 'system' },
  { id: 'hotkeyViewer', name: '快捷键占用', icon: '⌨', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M6 14h.01M18 14h.01M10 14h4"/></svg>`, description: '探测 Windows 已注册的全局快捷键，标注占用进程', keywords: ['快捷键', '热键', 'hotkey', '冲突', '占用'], category: 'system' },
  { id: 'hostsManager', name: 'Hosts管理', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>`, description: '编辑 hosts 文件，多环境 profile 切换，自动备份恢复', keywords: ['hosts', '域名', 'dns', '解析', 'profile'], category: 'system' },
  { id: 'networkConnections', name: '网络连接', icon: '🔌', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M8 8h8v8H8z"/><line x1="12" y1="4" x2="12" y2="8"/><line x1="12" y1="16" x2="12" y2="20"/><line x1="4" y1="12" x2="8" y2="12"/><line x1="16" y1="12" x2="20" y2="12"/></svg>`, description: '查看所有 TCP/UDP 连接，关联进程，支持筛选/结束进程/释放端口/导出', keywords: ['网络连接', 'tcp', 'udp', 'netstat', '端口', '连接', 'network'], category: 'system' },
  { id: 'scheduledTasks', name: '计划任务', icon: '🗓', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/><circle cx="8" cy="15" r="1.5"/><circle cx="12" cy="15" r="1.5"/><circle cx="16" cy="15" r="1.5"/></svg>`, description: '查看 Windows 计划任务列表，支持启用/禁用/立即运行/删除', keywords: ['计划任务', 'scheduled', 'task', 'schtasks', '定时', 'task scheduler'], category: 'system' },
  { id: 'startupItems', name: '开机启动项', icon: '🚀', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5l7 7-7 7"/></svg>`, description: '查看和管理 Windows 开机启动项，支持启用/禁用/删除/新增', keywords: ['启动项', 'startup', '开机', '自启动', 'autorun', '启动'], category: 'system' },
  { id: 'envVars', name: '环境变量', icon: '📝', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M14 14l-2-4-2 4"/><path d="M14 13h-4"/></svg>`, description: '查看和管理 Windows 环境变量，支持新增/修改/删除，PATH 逐行编辑', keywords: ['环境变量', 'env', 'PATH', '变量', 'environment'], category: 'system' },
  { id: 'certViewer', name: '证书查看器', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/><circle cx="12" cy="16" r="1"/></svg>`, description: '查看 Windows 证书存储，解析 .cer/.crt/.pfx 证书文件，查看 SSL 证书详情', keywords: ['证书', 'cert', 'ssl', 'tls', 'x509', 'pfx', 'cer', 'crt', '密钥', '指纹'], category: 'system' },
  { id: 'boost', name: '一键加速', icon: '⚡', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>`, description: '一键释放内存、清理临时文件、清空回收站，让电脑更流畅', keywords: ['加速', '清理', 'boost', '内存', '释放', '临时文件', '回收站', 'clean'], category: 'system' },
  { id: 'passwordVault', name: '密码保管箱', icon: '🔐', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/><path d="M12 15a1 1 0 100-2 1 1 0 000 2z"/></svg>`, description: '本地密码保管箱，主密码保护，加密存储凭据', keywords: ['密码保管箱', 'password', 'vault', '凭据', '加密', '安全'], category: 'security' },
  { id: 'changelog', name: '更新日志', icon: '📋', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>`, description: '查看每个版本的功能更新与修复记录', keywords: ['更新日志', 'changelog', '版本', '历史', '更新记录', 'release', 'whats new'] },
]

const MAX_HISTORY = 100
const MAX_RECENT = 8

export const useToolboxStore = defineStore('toolbox', () => {
  const config = ref<ToolboxConfig>({
    theme: 'auto',
    editorTheme: 'oneDark',
    jsonIndent: 2,
    hotkey: 'Ctrl+Alt+T',
    lastTool: 'home',
    favorites: ['note', 'pdf', 'ocr'],
    shortcuts: {
      json: 'CmdOrCtrl+Alt+J',
      string: 'CmdOrCtrl+Alt+S',
      devtools: 'CmdOrCtrl+Alt+D',
      fileprocessing: 'CmdOrCtrl+Alt+F',
      __palette__: 'CmdOrCtrl+Alt+P',
    },
  })

  const history = ref<HistoryRecord[]>([])
  const recentTools = ref<string[]>([])

  // ============ 命令面板 ============
  const isCommandPaletteOpen = ref(false)
  const openCommandPalette = () => { isCommandPaletteOpen.value = true }
  const closeCommandPalette = () => { isCommandPaletteOpen.value = false }
  const isQuickLaunchOpen = ref(false)
  const openQuickLaunch = () => { isQuickLaunchOpen.value = true }
  const closeQuickLaunch = () => { isQuickLaunchOpen.value = false }

  // ============ 后台采集状态 ============
  type CollectKind = 'system' | 'network' | 'process' | 'hardware' | 'software'
  const collectResults = ref<Record<CollectKind, unknown>>({
    system: null, network: null, process: null, hardware: null, software: null,
  })
  const collecting = ref<Record<CollectKind, boolean>>({
    system: false, network: false, process: false, hardware: false, software: false,
  })

  const pendingHistoryRestore = ref<HistoryRestoreState | null>(null)
  let restoreTimeout: ReturnType<typeof setTimeout> | null = null

  // 快捷键占用查看器：上次探测结果缓存（仅内存，不持久化 — 探测结果是实时快照）
  const hotkeyLastResult = ref<any[]>([])
  const hotkeyLastStats = ref<any | null>(null)

  // ============ 多 Tab 状态 ============
  interface Tab {
    toolId: string  // 同工具不允许多实例，toolId 即 tab 唯一标识
  }

  const MAX_TABS = 8
  const tabs = ref<Tab[]>([{ toolId: 'home' }])
  const activeTabId = ref('home')
  // 记录每个 toolId 被关闭的次数，作为 KeepAlive :key 的一部分，
  // 关闭后重新打开时 key 变化 → 强制创建新实例（不复用旧缓存状态）
  const closedCount = ref<Record<string, number>>({})

  /** 打开工具：已存在则激活，否则新建 tab（超出上限 LRU 关闭最早非 home tab） */
  const openTab = (toolId: string) => {
    const existing = tabs.value.find(t => t.toolId === toolId)
    if (existing) {
      activeTabId.value = toolId
      return
    }
    // LRU：超出上限时关闭最早的非 home tab
    if (tabs.value.length >= MAX_TABS) {
      const idx = tabs.value.findIndex(t => t.toolId !== 'home')
      if (idx !== -1) {
        const removed = tabs.value.splice(idx, 1)[0]
        closedCount.value[removed.toolId] = (closedCount.value[removed.toolId] || 0) + 1
      }
    }
    tabs.value.push({ toolId })
    activeTabId.value = toolId
  }

  /** 切换 tab */
  const switchTab = (toolId: string) => {
    if (tabs.value.find(t => t.toolId === toolId)) {
      activeTabId.value = toolId
    }
  }

  /** 关闭 tab：home 不可关闭；关闭当前 tab 时激活相邻 tab */
  const closeTab = (toolId: string) => {
    if (toolId === 'home') return
    const idx = tabs.value.findIndex(t => t.toolId === toolId)
    if (idx === -1) return
    tabs.value.splice(idx, 1)
    closedCount.value[toolId] = (closedCount.value[toolId] || 0) + 1
    // 调整 activeTabId
    if (activeTabId.value === toolId) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)]
      activeTabId.value = next ? next.toolId : 'home'
    }
  }

  /** 关闭其他：保留 home 和指定 tab */
  const closeOthers = (keepToolId: string) => {
    const removed = tabs.value.filter(t => t.toolId !== 'home' && t.toolId !== keepToolId)
    for (const t of removed) {
      closedCount.value[t.toolId] = (closedCount.value[t.toolId] || 0) + 1
    }
    tabs.value = tabs.value.filter(t => t.toolId === 'home' || t.toolId === keepToolId)
    activeTabId.value = keepToolId
  }

  /** 关闭全部：仅保留 home */
  const closeAllTabs = () => {
    const removed = tabs.value.filter(t => t.toolId !== 'home')
    for (const t of removed) {
      closedCount.value[t.toolId] = (closedCount.value[t.toolId] || 0) + 1
    }
    tabs.value = [{ toolId: 'home' }]
    activeTabId.value = 'home'
  }

  /** 获取 KeepAlive 的 :key（toolId + 关闭计数，保证关闭后重开是新实例） */
  const getTabKey = (toolId: string) => `${toolId}-${closedCount.value[toolId] || 0}`

  /** 兼容旧代码：activeTool 作为计算属性指向 activeTabId，setter 走 openTab 以复用 LRU/tab 创建逻辑 */
  const activeTool = computed({
    get: () => activeTabId.value,
    set: (val: string) => { openTab(val) }
  })

  const triggerHistoryRestore = (data: HistoryRestoreState) => {
    if (restoreTimeout) clearTimeout(restoreTimeout)
    pendingHistoryRestore.value = data
    // 30 秒未消费自动清除
    restoreTimeout = setTimeout(() => {
      pendingHistoryRestore.value = null
    }, 30000)
  }

  const clearHistoryRestore = () => {
    if (restoreTimeout) clearTimeout(restoreTimeout)
    pendingHistoryRestore.value = null
  }

  // 从 SQLite 加载配置
  const loadConfigFromDB = async () => {
    try {
      const savedConfig = await db.getConfig('main')
      if (savedConfig) {
        const parsed = JSON.parse(savedConfig)
        config.value = { ...config.value, ...parsed }
      }
    } catch (error) {
      console.error('加载配置失败:', error)
    }
  }

  // 从 SQLite 加载历史
  const loadHistoryFromDB = async () => {
    try {
      const records = await db.getHistory(MAX_HISTORY, 0)
      history.value = records.map(r => ({
        id: r.id,
        tool: r.tool,
        action: r.action,
        timestamp: r.created_at || new Date().toISOString(),
        inputPreview: r.input_preview,
        outputPreview: r.output_preview,
      }))
    } catch (error) {
      console.error('加载历史失败:', error)
    }
  }

  // 从 SQLite 加载最近工具
  const loadRecentFromDB = async () => {
    try {
      recentTools.value = await db.listRecentTools(MAX_RECENT)
    } catch (error) {
      console.error('加载最近工具失败:', error)
    }
  }

  // 初始化加载
  const loadFromDB = async () => {
    await Promise.all([
      loadConfigFromDB(),
      loadHistoryFromDB(),
      loadRecentFromDB(),
    ])
  }

  // 保存配置到 SQLite
  const saveConfig = async (newConfig: Partial<ToolboxConfig>) => {
    config.value = { ...config.value, ...newConfig }
    await db.setConfig('main', JSON.stringify(config.value))
  }

  // 添加历史记录到 SQLite
  const addHistory = async (record: Omit<HistoryRecord, 'timestamp'>) => {
    const newRecord = {
      ...record,
      timestamp: new Date().toISOString()
    }
    // 保存到 SQLite
    try {
      const id = await db.addHistory({
        tool: newRecord.tool,
        action: newRecord.action,
        input_preview: newRecord.inputPreview,
        output_preview: newRecord.outputPreview,
      })

      // 如果有完整数据，写入 details 表
      if (record.inputFull !== undefined || record.outputFull !== undefined || record.options) {
        await db.addHistoryDetail({
          history_id: id,
          input_full: record.inputFull ?? null,
          output_full: record.outputFull ?? null,
          options_json: JSON.stringify(record.options || {}),
        })
      }

      // 同步更新本地状态（带 id）
      newRecord.id = id
      history.value.unshift(newRecord)
      if (history.value.length > MAX_HISTORY) {
        history.value = history.value.slice(0, MAX_HISTORY)
      }
    } catch (error) {
      console.error('保存历史失败:', error)
    }
  }

  // 清空历史（SQLite + 本地状态）
  const clearHistory = async () => {
    history.value = []
    await db.clearHistory()
  }

  // 记录最近使用的工具
  const addRecentTool = async (toolId: string) => {
    if (toolId === 'home') return
    recentTools.value = recentTools.value.filter(id => id !== toolId)
    recentTools.value.unshift(toolId)
    if (recentTools.value.length > MAX_RECENT) {
      recentTools.value = recentTools.value.slice(0, MAX_RECENT)
    }
    await db.addRecentTool(toolId)
  }

  // 切换收藏
  const toggleFavorite = async (toolId: string) => {
    const idx = config.value.favorites.indexOf(toolId)
    if (idx > -1) {
      config.value.favorites.splice(idx, 1)
    } else {
      config.value.favorites.push(toolId)
    }
    await saveConfig({ favorites: config.value.favorites })
  }

  // 初始化加载
  loadFromDB()

  return {
    config,
    history,
    recentTools,
    saveConfig,
    addHistory,
    clearHistory,
    addRecentTool,
    toggleFavorite,
    pendingHistoryRestore,
    hotkeyLastResult,
    hotkeyLastStats,
    triggerHistoryRestore,
    clearHistoryRestore,
    activeTool,
    // 多 Tab
    tabs,
    activeTabId,
    closedCount,
    openTab,
    switchTab,
    closeTab,
    closeOthers,
    closeAllTabs,
    getTabKey,
    // 后台采集
    collectResults,
    collecting,
    // 命令面板
    isCommandPaletteOpen,
    openCommandPalette,
    closeCommandPalette,
    // 快速启动浮层
    isQuickLaunchOpen,
    openQuickLaunch,
    closeQuickLaunch,
  }
})