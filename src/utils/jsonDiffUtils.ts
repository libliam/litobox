// JSON 结构化对比：忽略对象键顺序，数组按索引逐项对比
// ponytail: 采用显式栈迭代（DFS）而非递归，避免超深 JSON（数千层）导致调用栈溢出；
// 若未来需要"数组忽略顺序"语义，可在 compare 中按数组元素集合匹配后再逐对对比

export type JsonDiffKind = 'added' | 'removed' | 'changed'

export interface JsonDiffRow {
  kind: JsonDiffKind
  /** 差异路径，如 a.b[2].c（根键无前缀） */
  path: string
  /** 左值；仅右存在时为空（added） */
  left?: unknown
  /** 右值；仅左存在时为空（removed） */
  right?: unknown
}

export interface JsonDiffResult {
  rows: JsonDiffRow[]
  /** 是否完全一致 */
  identical: boolean
  /** 各类差异计数（供统计徽标使用） */
  counts: Record<JsonDiffKind, number>
}

const hasOwn = (obj: Record<string, unknown>, key: string): boolean =>
  Object.prototype.hasOwnProperty.call(obj, key)

const isPlainObject = (v: unknown): v is Record<string, unknown> =>
  v !== null && typeof v === 'object' && !Array.isArray(v)

const isArray = (v: unknown): v is unknown[] => Array.isArray(v)

interface Frame {
  l: unknown
  r: unknown
  path: string
}

/** 路径数字段自然排序：a[2] < a[10] */
const comparePath = (p1: string, p2: string): number => {
  const chunk = /(\d+|\D+)/g
  const s1 = p1.match(chunk) ?? []
  const s2 = p2.match(chunk) ?? []
  const len = Math.max(s1.length, s2.length)
  for (let i = 0; i < len; i++) {
    const c1 = s1[i] ?? ''
    const c2 = s2[i] ?? ''
    if (c1 === c2) continue
    const n1 = /^\d+$/.test(c1) ? Number(c1) : null
    const n2 = /^\d+$/.test(c2) ? Number(c2) : null
    if (n1 !== null && n2 !== null) return n1 - n2
    return c1 < c2 ? -1 : 1
  }
  return 0
}

/**
 * 结构化对比两个 JSON 值。忽略对象键书写顺序；数组按索引逐项比较。
 * 对象/数组内嵌相等子树不产生差异行，仅在两端缺失/多出/终端值不等时记录。
 */
export function diffJson(left: unknown, right: unknown): JsonDiffResult {
  const rows: JsonDiffRow[] = []
  const stack: Frame[] = [{ l: left, r: right, path: '' }]

  const joinPath = (path: string, key: string) => (path ? `${path}.${key}` : key)

  while (stack.length) {
    const { l, r, path } = stack.pop()!
    if (isPlainObject(l) && isPlainObject(r)) {
      const keys = new Set([...Object.keys(l), ...Object.keys(r)])
      for (const key of keys) {
        const childPath = joinPath(path, key)
        const inL = hasOwn(l, key)
        const inR = hasOwn(r, key)
        if (!inL) {
          rows.push({ kind: 'added', path: childPath, right: r[key] })
        } else if (!inR) {
          rows.push({ kind: 'removed', path: childPath, left: l[key] })
        } else {
          stack.push({ l: l[key], r: r[key], path: childPath })
        }
      }
      continue
    }
    if (isArray(l) && isArray(r)) {
      const len = Math.max(l.length, r.length)
      for (let i = len - 1; i >= 0; i--) {
        // 逆序压栈仅影响遍历顺序，最终 rows 会按路径重排
        const childPath = `${path}[${i}]`
        const inL = i < l.length
        const inR = i < r.length
        if (!inL) {
          rows.push({ kind: 'added', path: childPath, right: r[i] })
        } else if (!inR) {
          rows.push({ kind: 'removed', path: childPath, left: l[i] })
        } else {
          stack.push({ l: l[i], r: r[i], path: childPath })
        }
      }
      continue
    }
    // 终端：基本类型 / null / 类型不匹配的容器（如 对象 vs 数组），按引用值判等
    if (l !== r) {
      rows.push({ kind: 'changed', path, left: l, right: r })
    }
  }

  rows.sort((a, b) => comparePath(a.path, b.path))
  const counts: JsonDiffResult['counts'] = { added: 0, removed: 0, changed: 0 }
  for (const row of rows) counts[row.kind]++
  return { rows, identical: rows.length === 0, counts }
}

/** 值格式化为单行展示文本（对象/数组压成 JSON） */
export function formatValue(v: unknown): string {
  if (v === undefined) return '∅'
  if (typeof v === 'string') return v
  try {
    return JSON.stringify(v)
  } catch {
    return String(v)
  }
}
