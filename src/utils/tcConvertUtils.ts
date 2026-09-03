// 中文繁简转换封装：基于 opencc-js 词组级标准 s2t/t2s（不涉及台/港用词习惯差异）
// ponytail: opencc-js 词典约 1MB，首次转换时才动态 import，避免打开工具页即阻塞主线程；
// 若未来只需单一方向，可改引 'opencc-js/cn2t' 等定向子包进一步减小体积

export type TcDirection = 's2t' | 't2s' | 'auto'

export interface TcResult {
  text: string
  /** 实际生效方向（auto 时是检测结果） */
  direction: 's2t' | 't2s'
  /** 被替换的字符数 */
  replaced: number
}

type TextConverter = (text: string) => string

let s2tConverter: TextConverter | null = null
let t2sConverter: TextConverter | null = null
let loadPromise: Promise<unknown> | null = null

async function loadConverters(): Promise<void> {
  if (s2tConverter && t2sConverter) return
  if (!loadPromise) {
    loadPromise = import('opencc-js').then((m) => {
      const { Converter } = m
      // cn→t = s2t(简→繁)，t→cn = t2s(繁→简)，t 为 OpenCC 标准繁体中间态
      s2tConverter = Converter({ from: 'cn', to: 't' })
      t2sConverter = Converter({ from: 't', to: 'cn' })
    })
  }
  await loadPromise
}

function countDiff(a: string, b: string): number {
  const len = Math.min(a.length, b.length)
  let n = 0
  for (let i = 0; i < len; i++) {
    if (a[i] !== b[i]) n++
  }
  return n
}

/**
 * 自动检测文本主体方向：取前 2000 字分别做双向转换，
 * 繁→简改动数更多说明原文以繁体为主（返回 t2s），否则默认简→繁（s2t）
 */
export async function detectDirection(text: string): Promise<'s2t' | 't2s'> {
  await loadConverters()
  const sample = text.slice(0, 2000)
  const toSimp = t2sConverter!(sample)
  const toTrad = s2tConverter!(sample)
  return countDiff(sample, toSimp) > countDiff(sample, toTrad) ? 't2s' : 's2t'
}

/** 转换整段文本；direction 缺省 auto（自动检测） */
export async function convertTc(text: string, direction: TcDirection = 'auto'): Promise<TcResult> {
  if (!text.trim()) return { text: '', direction: 's2t', replaced: 0 }
  await loadConverters()
  const dir = direction === 'auto' ? await detectDirection(text) : direction
  const converter = dir === 's2t' ? s2tConverter : t2sConverter
  const output = converter!(text)
  return { text: output, direction: dir, replaced: countDiff(text, output) }
}
