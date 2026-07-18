import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as db from '@/utils/dbClient'

export interface ToolboxConfig {
  theme: 'auto' | 'dark' | 'light'
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
  { id: 'json', name: 'JSON工具', icon: '{ }', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H6a2 2 0 00-2 2v4a2 2 0 01-2 2 2 2 0 012 2v4a2 2 0 002 2h2"/><path d="M16 3h2a2 2 0 012 2v4a2 2 0 002 2 2 2 0 00-2 2v4a2 2 0 01-2 2h-2"/></svg>`, description: 'JSON格式化、压缩、校验、兼容解析', keywords: ['json', '格式化', '压缩', '校验'], category: 'text' },
  { id: 'string', name: '字符串工具', icon: 'Aa', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M12 4v16"/><path d="M8 20h8"/><path d="M17 14l-3-3 3-3"/></svg>`, description: '空格处理、拼接分割、大小写转换、文本清理、批量处理', keywords: ['字符串', '空格', '大小写', '文本', '批量'], category: 'text' },
  { id: 'markdown', name: 'Markdown', icon: 'MD', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M7 15V9l2.5 3L12 9v6"/><path d="M17 9v6"/><path d="M15 12h4"/></svg>`, description: 'Markdown 实时预览、HTML 互转、导出、统计', keywords: ['markdown', 'md', '预览', 'html', '转换', '统计'], category: 'text' },
  { id: 'wordCount', name: '字数统计', icon: 'ABC', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M16 16l-2-4-2 4"/><path d="M14 13h-4"/></svg>`, description: '字符数、单词数、行数、阅读时间估算', keywords: ['字数', '统计', '字符', '单词', '行数'], category: 'text' },
  { id: 'diff', name: '文本对比', icon: '≠', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h16M4 12h16M4 18h16"/><path d="M9 3v18M15 3v18"/></svg>`, description: '文本/代码对比，支持行级和字符级差异高亮', keywords: ['对比', 'diff', '差异', '代码对比'], category: 'text' },
  { id: 'dedup', name: '文本去重', icon: '≡', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h16M4 12h12M4 18h8"/><circle cx="20" cy="12" r="1" fill="currentColor"/><circle cx="16" cy="18" r="1" fill="currentColor"/></svg>`, description: '按行去重，支持首次/末次保留', keywords: ['去重', '重复', 'dedup', '清理'], category: 'text' },
  { id: 'regex', name: '正则测试', icon: '.*', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>`, description: '正则表达式测试、匹配、替换', keywords: ['正则', 'regex', '匹配', '替换'], category: 'text' },
  { id: 'note', name: '文本编辑器', icon: '📝', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>`, description: '草稿本/便签，支持语法高亮、查找替换、自动保存', keywords: ['文本编辑器', '草稿', '笔记', 'notepad'], category: 'text' },
  { id: 'encode', name: '编码工具', icon: 'En', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M16 16l-2-4-2 4"/></svg>`, description: 'Base64、URL、HTML实体、Unicode编解码', keywords: ['编码', 'base64', 'url', 'unicode'], category: 'dev' },
  { id: 'hash', name: '哈希计算', icon: '#', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9h16M4 15h16"/><path d="M10 3l-2 18M16 3l-2 18"/></svg>`, description: 'MD5/SHA-1/SHA-256/SHA-512哈希计算', keywords: ['hash', 'md5', 'sha', '哈希', '摘要'], category: 'dev' },
  { id: 'jwt', name: 'JWT解析', icon: 'JWT', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>`, description: '解析 JWT token，查看 Header/Payload', keywords: ['jwt', 'token', '解析', '认证'], category: 'dev' },
  { id: 'time', name: '时间工具', icon: '🕐', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>`, description: '时间戳转换、日期计算、相对时间', keywords: ['时间', 'timestamp', '日期', '转换'], category: 'dev' },
  { id: 'cron', name: 'Cron表达式', icon: '⏰', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>`, description: '可视化生成 Cron 表达式，支持5/6字段格式', keywords: ['cron', '定时', '调度', '表达式'], category: 'dev' },
  { id: 'url', name: 'URL工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>`, description: 'URL解析、编码、参数提取', keywords: ['url', '链接', '解析', '参数'], category: 'dev' },
  { id: 'baseConverter', name: '进制转换', icon: '0x', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>`, description: '二进制、八进制、十进制、十六进制互转', keywords: ['进制', '转换', 'binary', 'hex', 'octal'], category: 'dev' },
  { id: 'uuid', name: 'UUID生成', icon: 'UUID', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>`, description: 'UUID v4 生成', keywords: ['uuid', 'guid', '生成', '唯一标识'], category: 'dev' },
  { id: 'color', name: '颜色工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="8" cy="9" r="1.5" fill="currentColor"/><circle cx="12" cy="7" r="1.5" fill="currentColor"/><circle cx="16" cy="9" r="1.5" fill="currentColor"/><circle cx="8" cy="13" r="1.5" fill="currentColor"/><circle cx="16" cy="13" r="1.5" fill="currentColor"/><circle cx="12" cy="17" r="1.5" fill="currentColor"/></svg>`, description: '颜色选择器、格式转换、色板生成、对比度检查、渐变生成', keywords: ['颜色', 'color', '拾色器', '色板', '对比度', '渐变', 'hex', 'rgb', 'hsl'], category: 'dev' },
  { id: 'css', name: 'CSS工具', icon: 'CSS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3h12l-3 9H9z"/><path d="M9 12l-2 9h10l-2-9"/></svg>`, description: '颜色转换、单位换算、CSS压缩/格式化', keywords: ['css', '颜色', '单位', '压缩'], category: 'dev' },
  { id: 'js', name: 'JS工具', icon: 'JS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M10 12l2 2 4-4"/></svg>`, description: 'JS沙箱运行、格式化、压缩、JSON提取', keywords: ['js', 'javascript', '沙箱', '格式化', '压缩'], category: 'dev' },
  { id: 'sql', name: 'SQL工具', icon: 'SQL', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M4 9h16"/><path d="M9 4v16"/></svg>`, description: '字符串列表转SQL IN查询条件', keywords: ['sql', 'in', '查询', '转换'], category: 'dev' },
  { id: 'xmlYaml', name: 'XML/YAML', icon: 'XML', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H6a2 2 0 00-2 2v4"/><path d="M16 3h2a2 2 0 012 2v4"/><path d="M14 17l-3 3-3-3"/><path d="M4 21h16"/></svg>`, description: 'XML/YAML格式化、校验、JSON互转', keywords: ['xml', 'yaml', '格式化', '校验', '转换'], category: 'dev' },
  { id: 'csv', name: 'CSV工具', icon: 'CSV', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M4 9h16"/><path d="M9 4v16"/></svg>`, description: 'CSV解析、表格预览、导出JSON/SQL', keywords: ['csv', '表格', '解析', '导出'], category: 'dev' },
  { id: 'pdf', name: 'PDF工具', icon: 'PDF', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M9 15v-2a2 2 0 014 0v2"/><path d="M9 13h4"/></svg>`, description: 'PDF转图片、图片转PDF、文本提取、合并拆分、压缩', keywords: ['pdf', '转换', '合并', '拆分', '提取', '压缩'], category: 'utility' },
  { id: 'http', name: 'HTTP 请求', icon: '🌐', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v20M5 5h14l-7 7 7 7H5"/></svg>`, description: 'HTTP 请求测试，支持 GET/POST/PUT/DELETE，绕过 CORS 限制', keywords: ['http', '请求', 'api', 'get', 'post', 'put', 'delete', 'cors'], category: 'dev' },
  { id: 'password', name: '密码工具', icon: '🔑', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/><circle cx="12" cy="16" r="1"/></svg>`, description: '随机密码生成、API Key 生成、密码强度检测', keywords: ['密码', 'password', '随机', '生成', '强度', 'api key', 'token'], category: 'security' },
  { id: 'calculator', name: '计算器', icon: '∑', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="2" width="16" height="20" rx="2"/><line x1="8" y1="6" x2="16" y2="6"/><line x1="8" y1="10" x2="16" y2="10"/><line x1="8" y1="14" x2="12" y2="14"/><line x1="14" y1="14" x2="16" y2="14"/><line x1="8" y1="18" x2="12" y2="18"/><line x1="14" y1="18" x2="16" y2="18"/></svg>`, description: '表达式计算、单位换算、日期计算、时间戳转换', keywords: ['计算器', '计算', '单位换算', '日期', '时间戳'], category: 'utility' },
  { id: 'qr', name: '二维码', icon: 'QR', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="8" height="8" rx="1"/><rect x="14" y="2" width="8" height="8" rx="1"/><rect x="2" y="14" width="8" height="8" rx="1"/><rect x="14" y="14" width="4" height="4"/><rect x="20" y="14" width="2" height="2"/><rect x="14" y="20" width="2" height="2"/><rect x="20" y="20" width="2" height="2"/></svg>`, description: '二维码生成与解码，支持文本/URL转二维码、图片解码', keywords: ['二维码', 'qr', 'qrcode', '生成', '解码', '扫码'], category: 'utility' },
  { id: 'image', name: '图片工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>`, description: '图片压缩、尺寸缩放、转Base64', keywords: ['图片', '压缩', '缩放', 'base64', 'image'], category: 'utility' },
  { id: 'imageToolEnhanced', name: '图片增强', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>`, description: '批量压缩/格式转换、图片拼接、加水印、调色板提取', keywords: ['图片', '压缩', '拼接', '水印', '调色板', '格式转换', 'image'], category: 'utility' },
  { id: 'audioTool', name: '音频工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>`, description: '音频裁剪、格式转换，支持 MP3/WAV/M4A，波形可视化、实时预览', keywords: ['音频', '裁剪', 'mp3', 'wav', 'm4a', '波形', 'audio'], category: 'utility' },
  { id: 'videoTool', name: '视频工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2"/></svg>`, description: '视频裁剪/转码/音频提取/压缩/合并，支持 ffmpeg 增强', keywords: ['视频', '裁剪', '转码', '音频提取', '压缩', '合并', 'mp4', 'video', 'ffmpeg'], category: 'utility' },
  { id: 'iconGenerator', name: '图标生成', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><rect x="7" y="7" width="3" height="3" rx="0.5"/><rect x="14" y="7" width="3" height="3" rx="0.5"/><rect x="7" y="14" width="3" height="3" rx="0.5"/><rect x="14" y="14" width="3" height="3" rx="0.5"/></svg>`, description: '一张图生成多尺寸图标（favicon/icon set），支持 PNG/ICO 格式', keywords: ['图标', 'icon', 'favicon', 'ico', '生成', '尺寸'], category: 'utility' },
  { id: 'ocr', name: 'OCR识别', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>`, description: '图片文字识别，支持上传和剪贴板粘贴', keywords: ['ocr', '文字识别', '图片', '识别'], category: 'utility' },
  { id: 'mockData', name: '随机数据', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/><circle cx="8" cy="8" r="1" fill="currentColor"/><circle cx="16" cy="8" r="1" fill="currentColor"/><circle cx="8" cy="16" r="1" fill="currentColor"/><circle cx="16" cy="16" r="1" fill="currentColor"/></svg>`, description: '姓名、身份证、手机号等随机生成', keywords: ['随机', '假数据', 'mock', '测试数据'], category: 'utility' },
  { id: 'fileprocessing', name: '文件处理', icon: '📁', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>`, description: '批量文本处理、文件编码转换', keywords: ['文件', '编码', '转换', '批量', '替换'], category: 'utility' },
  { id: 'clipboard', name: '剪贴板', icon: '📋', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2"/><path d="M9 14l2 2 4-4"/></svg>`, description: '系统剪贴板历史记录', keywords: ['剪贴板', '复制', '历史', 'clipboard'] },
  { id: 'snippet', name: '代码片段', icon: '<>', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`, description: '代码片段管理，支持分类、搜索、导入导出', keywords: ['代码', '片段', 'snippet', '管理', '收藏', '模板'], category: 'utility' },
  { id: 'history', name: '历史记录', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/><path d="M3 12a9 9 0 0115.36-6.36L21 3"/></svg>`, description: '查看和清空操作历史', keywords: ['历史', '记录', '操作'] },
  { id: 'workflow', name: '工作流', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h4l3 3h6l3-3h4"/><path d="M4 12h4l3 3h6l3-3h4"/><path d="M4 18h4l3 3h6l3-3h4"/><circle cx="4" cy="6" r="1" fill="currentColor"/><circle cx="20" cy="6" r="1" fill="currentColor"/><circle cx="4" cy="12" r="1" fill="currentColor"/><circle cx="20" cy="12" r="1" fill="currentColor"/><circle cx="4" cy="18" r="1" fill="currentColor"/><circle cx="20" cy="18" r="1" fill="currentColor"/></svg>`, description: '工作流编排，链式处理，变量池管理', keywords: ['工作流', '编排', '链式', '变量池'], category: 'utility' },
  { id: 'systemInfo', name: '系统信息', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>`, description: '查看操作系统、CPU、内存、磁盘信息', keywords: ['系统', 'cpu', '内存', '磁盘', 'system'], category: 'system' },
  { id: 'networkInfo', name: '网络信息', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/></svg>`, description: '查看网络接口、IP、连接、端口', keywords: ['网络', 'ip', 'mac', '端口', 'netstat'], category: 'system' },
  { id: 'processList', name: '进程列表', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>`, description: '查看运行中的进程及资源占用', keywords: ['进程', 'process', '任务管理器'], category: 'system' },
  { id: 'hardwareInfo', name: '硬件外设', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>`, description: '查看GPU、显示器、音频设备', keywords: ['硬件', 'gpu', '显卡', '显示器', '音频'], category: 'system' },
  { id: 'softwareEnv', name: '软件环境', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>`, description: '已安装软件、环境变量、启动项', keywords: ['软件', '环境变量', '启动项', 'env'], category: 'system' },
  { id: 'sqliteViewer', name: 'SQLite查看器', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 5v6c0 1.66-4.03 3-9 3s-9-1.34-9-3V5"/><path d="M21 11v6c0 1.66-4.03 3-9 3s-9-1.34-9-3v-6"/></svg>`, description: '浏览本地SQLite数据库表结构和数据，执行SELECT查询，导出CSV', keywords: ['sqlite', '数据库', 'db', '查询', '查看', 'database'], category: 'dev' },
  { id: 'diskAnalyzer', name: '磁盘分析', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/><line x1="12" y1="2" x2="12" y2="6"/><line x1="12" y1="18" x2="12" y2="22"/><line x1="2" y1="12" x2="6" y2="12"/><line x1="18" y1="12" x2="22" y2="12"/></svg>`, description: '分析磁盘空间占用，查找大文件和重复文件', keywords: ['磁盘', '空间', '重复', '清理', 'disk', 'space', 'duplicate'], category: 'system' },
  { id: 'fileSearcher', name: '全文搜索', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>`, description: '按文件名或内容搜索，支持正则表达式，类似 Everything + grep', keywords: ['搜索', '全文', '文件名', '内容', 'grep', 'find', 'search'], category: 'system' },
]

const MAX_HISTORY = 100
const MAX_RECENT = 8

export const useToolboxStore = defineStore('toolbox', () => {
  const config = ref<ToolboxConfig>({
    theme: 'auto',
    jsonIndent: 2,
    hotkey: 'Ctrl+Alt+T',
    lastTool: 'home',
    favorites: ['note', 'pdf', 'ocr'],
    shortcuts: {
      json: 'CmdOrCtrl+Alt+J',
      string: 'CmdOrCtrl+Alt+S',
      devtools: 'CmdOrCtrl+Alt+D',
      fileprocessing: 'CmdOrCtrl+Alt+F'
    }
  })

  const history = ref<HistoryRecord[]>([])
  const recentTools = ref<string[]>([])

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
  }
})