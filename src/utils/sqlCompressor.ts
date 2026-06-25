export function compressSql(sql: string): string {
  if (!sql.trim()) return ''

  let i = 0
  let output = ''

  while (i < sql.length) {
    // 单引号字符串
    if (sql[i] === "'") {
      let j = i + 1
      while (j < sql.length) {
        if (sql[j] === "'" && sql[j + 1] === "'") {
          j += 2
        } else if (sql[j] === "'") {
          break
        } else {
          j++
        }
      }
      output += sql.slice(i, j + 1)
      i = j + 1
      continue
    }

    // 双引号字符串
    if (sql[i] === '"') {
      let j = i + 1
      while (j < sql.length && sql[j] !== '"') {
        if (sql[j] === '\\') j++
        j++
      }
      output += sql.slice(i, j + 1)
      i = j + 1
      continue
    }

    // 单行注释 --
    if (sql[i] === '-' && sql[i + 1] === '-') {
      while (i < sql.length && sql[i] !== '\n') {
        i++
      }
      continue
    }

    // 多行注释 /* */
    if (sql[i] === '/' && sql[i + 1] === '*') {
      i += 2
      while (i < sql.length && !(sql[i] === '*' && sql[i + 1] === '/')) {
        i++
      }
      i += 2
      continue
    }

    // 普通字符
    output += sql[i]
    i++
  }

  // 将连续空白替换为单个空格
  output = output.replace(/\s+/g, ' ').trim()

  return output
}
