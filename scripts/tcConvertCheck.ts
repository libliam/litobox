// 中文繁简转换核心逻辑自检（node scripts/tcConvertCheck.ts，Node>=23.6 直接运行）
// 覆盖词组级多义字（发→發/髮）、双向转换与自动检测；opencc 词典行为变更时同步更新用例
import { convertTc, detectDirection } from '../src/utils/tcConvertUtils.ts'

async function main() {
  let failed = 0
  function eq(actual: string, expected: string, label: string) {
    if (actual === expected) {
      console.log(`  ok  ${label}: ${expected}`)
    } else {
      failed++
      console.error(`FAIL  ${label}: expected "${expected}", got "${actual}"`)
    }
  }

  console.log('convertTc 简→繁（词组级多义字）:')
  const a = await convertTc('我的头发发生了问题', 's2t')
  eq(a.text, '我的頭髮發生了問題', 's2t 头发/发生 区分')
  eq(a.direction, 's2t', 'direction 透传')

  const b = await convertTc('云龙区的档案里有一个文件', 's2t')
  eq(b.text, '雲龍區的檔案裏有一個文件', 's2t 常见词')

  console.log('convertTc 繁→简:')
  const c = await convertTc('雲龍區的頭髮與檔案，裡面包裹著', 't2s')
  eq(c.text, '云龙区的头发与档案，里面包裹著', 't2s 词组转换')

  console.log('detectDirection 自动检测:')
  eq(await detectDirection('云龙区发生了问题'), 's2t', '简体文本')
  eq(await detectDirection('雲龍區發生瞭問題'), 't2s', '繁体文本')
  eq(await detectDirection(''), 's2t', '空文本默认')

  console.log('convertTc auto + replaced:')
  const d = await convertTc('云龙区的档案')
  eq(d.direction, 's2t', 'auto 简体→繁体')
  eq(d.replaced > 0 ? 'yes' : 'no', 'yes', '有替换字符')

  if (failed) {
    console.error(`\n${failed} 个用例失败`)
    process.exit(1)
  }
  console.log('\n全部用例通过')
}

main()
