/**
 * 速查表通用类型定义
 * 新增速查表：
 *   1. 在 cheatSheets/ 下新建文件实现 CheatSheetData 接口并 export
 *   2. 在本文件 import 并加入 cheatSheets 数组
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

export interface CheatSheetRow {
  /** 动态列数据（由各速查表 columns 定义） */
  [key: string]: string | string[] | undefined
  /** 示例（可选，展开行显示） */
  examples?: string[]
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
  rows: CheatSheetRow[]
}

import { httpStatusCodes } from './http'
import { mimeTypes } from './mime'
import { linuxCommands } from './linux'
import { gitCommands } from './git'
import { sqlCheatsheet } from './sql'
import { sqlDialectCheatsheet } from './sqlDialect'
import { dockerCommands } from './docker'
import { condaCommands } from './conda'
import { packageManagerCheatsheet } from './packageManager'
import { redisCommands } from './redis'
import { powerShellCommands } from './powershell'
import { regexCheatsheet } from './regex'

// 所有速查表（按 Tab 顺序排列）
export const cheatSheets: CheatSheetData[] = [
  httpStatusCodes,
  mimeTypes,
  linuxCommands,
  gitCommands,
  sqlCheatsheet,
  sqlDialectCheatsheet,
  dockerCommands,
  redisCommands,
  condaCommands,
  powerShellCommands,
  packageManagerCheatsheet,
  regexCheatsheet,
]
