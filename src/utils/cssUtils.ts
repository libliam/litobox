// src/utils/cssUtils.ts

// ============ 颜色转换 ============

export interface RGB { r: number; g: number; b: number }
export interface HSL { h: number; s: number; l: number }

export function hexToRgb(hex: string): RGB | null {
  const match = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex.trim())
  if (!match) return null
  return {
    r: parseInt(match[1], 16),
    g: parseInt(match[2], 16),
    b: parseInt(match[3], 16)
  }
}

export function rgbToHex(r: number, g: number, b: number): string {
  return '#' + [r, g, b].map(x => {
    const hex = Math.round(Math.max(0, Math.min(255, x))).toString(16)
    return hex.length === 1 ? '0' + hex : hex
  }).join('')
}

export function rgbToHsl(r: number, g: number, b: number): HSL {
  r /= 255; g /= 255; b /= 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b)
  let h = 0, s = 0
  const l = (max + min) / 2

  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break
      case g: h = ((b - r) / d + 2) / 6; break
      case b: h = ((r - g) / d + 4) / 6; break
    }
  }

  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) }
}

export function hslToRgb(h: number, s: number, l: number): RGB {
  h /= 360; s /= 100; l /= 100
  let r: number, g: number, b: number

  if (s === 0) {
    r = g = b = l
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1
      if (t > 1) t -= 1
      if (t < 1/6) return p + (q - p) * 6 * t
      if (t < 1/2) return q
      if (t < 2/3) return p + (q - p) * (2/3 - t) * 6
      return p
    }
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s
    const p = 2 * l - q
    r = hue2rgb(p, q, h + 1/3)
    g = hue2rgb(p, q, h)
    b = hue2rgb(p, q, h - 1/3)
  }

  return { r: Math.round(r * 255), g: Math.round(g * 255), b: Math.round(b * 255) }
}

export function parseColor(input: string): { hex: string; rgb: string; hsl: string } | null {
  input = input.trim()

  // Try hex
  if (/^#?[a-f\d]{6}$/i.test(input)) {
    const hex = input.startsWith('#') ? input : '#' + input
    const rgb = hexToRgb(hex)
    if (rgb) {
      const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b)
      return {
        hex,
        rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
        hsl: `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`
      }
    }
  }

  // Try rgb
  const rgbMatch = input.match(/rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/)
  if (rgbMatch) {
    const r = parseInt(rgbMatch[1]), g = parseInt(rgbMatch[2]), b = parseInt(rgbMatch[3])
    const hex = rgbToHex(r, g, b)
    const hsl = rgbToHsl(r, g, b)
    return {
      hex,
      rgb: input,
      hsl: `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`
    }
  }

  // Try hsl
  const hslMatch = input.match(/hsl\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*\)/)
  if (hslMatch) {
    const h = parseInt(hslMatch[1]), s = parseInt(hslMatch[2]), l = parseInt(hslMatch[3])
    const rgb = hslToRgb(h, s, l)
    const hex = rgbToHex(rgb.r, rgb.g, rgb.b)
    return {
      hex,
      rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
      hsl: input
    }
  }

  return null
}

// ============ 单位换算 ============

export interface UnitConversion {
  from: string
  to: string
  value: number
  result: number
}

export function convertUnit(value: number, from: string, to: string, baseFontSize: number = 16): number | null {
  // 先转换为 px
  let px: number
  switch (from.toLowerCase()) {
    case 'px': px = value; break
    case 'rem': px = value * baseFontSize; break
    case 'em': px = value * baseFontSize; break
    case 'vw': px = value * 19.2; break  // 假设 1920px 视口
    case 'vh': px = value * 10.8; break  // 假设 1080px 视口
    default: return null
  }

  // 从 px 转换为目标单位
  switch (to.toLowerCase()) {
    case 'px': return px
    case 'rem': return px / baseFontSize
    case 'em': return px / baseFontSize
    case 'vw': return px / 19.2
    case 'vh': return px / 10.8
    default: return null
  }
}

// ============ CSS 压缩 ============

export function compressCss(css: string): string {
  return css
    .replace(/\/\*[\s\S]*?\*\//g, '')  // 移除注释
    .replace(/\s+/g, ' ')               // 合并空白
    .replace(/\s*([{}:;,])\s*/g, '$1') // 移除符号周围空格
    .replace(/;}/g, '}')               // 移除最后一个分号
    .trim()
}

export function formatCss(css: string): string {
  let formatted = ''
  let indent = 0
  const tab = '  '

  for (const char of css) {
    if (char === '{') {
      formatted += ' {\n'
      indent++
      formatted += tab.repeat(indent)
    } else if (char === '}') {
      indent--
      formatted += '\n' + tab.repeat(indent) + '}\n\n'
    } else if (char === ';') {
      formatted += ';\n' + tab.repeat(indent)
    } else if (char === '\n' || char === '\r') {
      // 跳过原始换行
    } else if (char === ' ' && formatted.endsWith(tab.repeat(indent))) {
      // 跳过多余空格
    } else {
      formatted += char
    }
  }

  return formatted.trim()
}
