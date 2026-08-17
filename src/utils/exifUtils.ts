import exifr from 'exifr'

// 常用 EXIF 标签的中文映射（key 为 exifr 返回的标签名）
const TAG_LABELS: Record<string, string> = {
  Make: '相机品牌',
  Model: '相机型号',
  LensModel: '镜头型号',
  LensMake: '镜头品牌',
  DateTimeOriginal: '拍摄时间',
  CreateDate: '拍摄时间',
  DateTime: '日期时间',
  ModifyDate: '修改时间',
  FNumber: '光圈',
  ExposureTime: '快门',
  ISO: 'ISO',
  ISOSpeedRatings: 'ISO',
  FocalLength: '焦距',
  FocalLengthIn35mmFormat: '等效35mm焦距',
  ExposureCompensation: '曝光补偿',
  ExposureBiasValue: '曝光补偿',
  ExposureMode: '曝光模式',
  ExposureProgram: '曝光程序',
  WhiteBalance: '白平衡',
  Flash: '闪光灯',
  MeteringMode: '测光模式',
  LightSource: '光源',
  ColorSpace: '色彩空间',
  Orientation: '方向',
  Software: '软件',
  Artist: '作者',
  Copyright: '版权',
  ImageDescription: '图片描述',
  UserComment: '用户注释',
  XResolution: '水平分辨率',
  YResolution: '垂直分辨率',
  ResolutionUnit: '分辨率单位',
  PixelXDimension: '图片宽度',
  PixelYDimension: '图片高度',
  ExifImageWidth: '图片宽度',
  ExifImageHeight: '图片高度',
  FileSource: '文件来源',
  DigitalZoomRatio: '数字变焦',
  SceneType: '场景类型',
  SceneCaptureType: '场景拍摄类型',
  SensingMethod: '感光方式',
  Contrast: '对比度',
  Saturation: '饱和度',
  Sharpness: '锐度',
  GPSLatitude: 'GPS纬度',
  GPSLongitude: 'GPS经度',
  GPSAltitude: 'GPS海拔',
  GPSDateTime: 'GPS时间',
}

export interface ExifItem {
  key: string
  label: string
  value: string
}

export interface ExifResult {
  /** 有序的标签列表 */
  items: ExifItem[]
  /** 是否有 EXIF 信息 */
  hasExif: boolean
  /** 是否含 GPS 定位信息 */
  hasGps: boolean
  /** 十进制经纬度（可复制） */
  lat?: number
  lng?: number
  /** GPS 文字描述，如 31°13'48.0"N, 121°28'12.0"E */
  gpsText?: string
  /** 图片格式，如 jpeg/png/webp */
  format: string
}

function roundNum(v: number, digits = 2): string {
  const n = Number(v)
  if (!Number.isFinite(n)) return String(v)
  return String(Number(n.toFixed(digits)))
}

/** 把 exifr 返回的任意值转成可展示字符串 */
function formatValue(v: unknown): string {
  if (v == null) return ''
  if (typeof v === 'number') return roundNum(v, 3)
  if (typeof v === 'boolean') return v ? '是' : '否'
  if (typeof v === 'string') return v
  if (Array.isArray(v)) return v.map((x) => formatValue(x)).filter(Boolean).join(', ')
  if (typeof v === 'object') {
    // 角度对象 {34: 0.5, ...} 这类秒值对象 → 数值列表
    const nums = Object.values(v as Record<string, unknown>)
      .filter((x): x is number => typeof x === 'number')
    if (nums.length) return nums.map((n) => roundNum(n, 4)).join(', ')
    return JSON.stringify(v)
  }
  return String(v)
}

function toDms(deg: number): string {
  const abs = Math.abs(deg)
  const d = Math.floor(abs)
  const mFloat = (abs - d) * 60
  const m = Math.floor(mFloat)
  const s = ((mFloat - m) * 60).toFixed(1)
  return `${d}°${String(m).padStart(2, '0')}'${s}"`
}

/**
 * 读取图片 EXIF 信息（JPEG/TIFF/PNG/WebP 等）。
 * 合并默认精选标签 + IFD0 隐私相关标签 + GPS。
 */
export async function readExif(file: File): Promise<ExifResult> {
  const fmt = (file.type.split('/')[1] || file.name.split('.').pop() || '').toLowerCase()
  const [tags, gps] = await Promise.all([
    exifr.parse(file, {
      // 全量翻译 IFD0 + ExifIFD 标签，GPS 单独用 exifr.gps 提取
      ifd0: { translateKeys: true, translateValues: true },
      exif: { translateKeys: true, translateValues: true },
      ifd1: false,
      gps: false,
      xmp: false,
    }).catch(() => null),
    exifr.gps(file).catch(() => null),
  ])

  const merged: Record<string, unknown> = {}
  if (tags) Object.assign(merged, tags)

  const items: ExifItem[] = []
  // GPS 放在最前（隐私重点）
  if (gps && typeof gps.latitude === 'number' && typeof gps.longitude === 'number') {
    const lat = gps.latitude
    const lng = gps.longitude
    items.push({
      key: 'gps', label: 'GPS 定位',
      value: `${toDms(lat)} ${lat >= 0 ? 'N' : 'S'}, ${toDms(lng)} ${lng >= 0 ? 'E' : 'W'}`,
    })
  }

  for (const [key, v] of Object.entries(merged)) {
    const value = formatValue(v)
    if (!value) continue
    items.push({ key, label: TAG_LABELS[key] || key, value })
  }

  const gpsLat = gps && typeof gps.latitude === 'number' ? gps.latitude : undefined
  const gpsLng = gps && typeof gps.longitude === 'number' ? gps.longitude : undefined
  return {
    items,
    hasExif: items.length > 0,
    hasGps: gpsLat !== undefined && gpsLng !== undefined,
    lat: gpsLat,
    lng: gpsLng,
    gpsText:
      gpsLat !== undefined && gpsLng !== undefined
        ? `${gpsLat.toFixed(6)}, ${gpsLng.toFixed(6)}`
        : undefined,
    format: fmt,
  }
}

// JPEG 需要剥离的元数据段 marker（EXIF/XMP=APP1, Ducky=APP12, IPTC=APP13, 注释=COM）
const JPEG_STRIP_MARKERS = new Set([0xe1, 0xec, 0xed, 0xfe])

/**
 * 无损剥离 JPEG 的 EXIF 等元数据段（不重编码，保留画质）。
 * 保留 JFIF(APP0)、ICC(APP2)、Adobe(APP14)、量化表/霍夫曼表/扫描数据。
 */
export function stripJpegExif(bytes: Uint8Array): Uint8Array {
  if (bytes.length < 4 || bytes[0] !== 0xff || bytes[1] !== 0xd8) {
    throw new Error('不是有效的 JPEG 文件')
  }
  const out: number[] = [0xff, 0xd8]
  let i = 2
  let sawSos = false
  while (i < bytes.length && !sawSos) {
    // 找到 marker（0xFF 可能填充多个）
    while (i < bytes.length && bytes[i] === 0xff) i++
    if (i >= bytes.length) break
    const marker = bytes[i]
    i++
    if (marker === 0xd9) break // EOI，结束
    if (marker === 0x01 || (marker >= 0xd0 && marker <= 0xd7)) {
      out.push(0xff, marker) // TEM / RSTn：无长度字段
      continue
    }
    if (i + 2 > bytes.length) break
    const len = (bytes[i] << 8) | bytes[i + 1]
    if (len < 2 || i + len > bytes.length) break
    const seg = bytes.subarray(i, i + len)
    i += len
    if (marker === 0xda) {
      // SOS：其后是压缩数据，整体保留后退出
      out.push(0xff, marker, ...seg, ...bytes.subarray(i))
      sawSos = true
      break
    }
    if (JPEG_STRIP_MARKERS.has(marker)) continue // 剥离元数据段
    out.push(0xff, marker, ...seg) // 保留结构段（SOF/DQT/DHT/DRI/APP0/APP2/APP14 等）
  }
  if (!sawSos) throw new Error('JPEG 结构不完整（缺少图像数据段）')
  return new Uint8Array(out)
}

/** 通过 Canvas 重绘剥离元数据（PNG/WebP 等非 JPEG 格式降级方案） */
function stripExifViaCanvas(file: File, mime: string): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const url = URL.createObjectURL(file)
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      canvas.width = img.naturalWidth
      canvas.height = img.naturalHeight
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        URL.revokeObjectURL(url)
        reject(new Error('无法创建画布'))
        return
      }
      ctx.drawImage(img, 0, 0)
      canvas.toBlob(
        (blob) => {
          URL.revokeObjectURL(url)
          blob ? resolve(blob) : reject(new Error('图片重绘失败'))
        },
        mime,
        0.95,
      )
    }
    img.onerror = () => {
      URL.revokeObjectURL(url)
      reject(new Error('图片解码失败，浏览器可能不支持该格式'))
    }
    img.src = url
  })
}

/** 剥离图片的 EXIF 等隐私元数据，返回新 Blob */
export async function stripExif(file: File): Promise<Blob> {
  const fmt = (file.type.split('/')[1] || file.name.split('.').pop() || '').toLowerCase()
  if (fmt === 'jpeg' || fmt === 'jpg') {
    const bytes = new Uint8Array(await file.arrayBuffer())
    const clean = stripJpegExif(bytes)
    // 拷贝为独立 ArrayBuffer，兼容 Blob 构造类型约束
    const out = new Uint8Array(clean)
    return new Blob([out], { type: 'image/jpeg' })
  }
  if (fmt === 'png' || fmt === 'webp') {
    return stripExifViaCanvas(file, `image/${fmt}`)
  }
  throw new Error(`暂不支持剥离 ${fmt || '未知'} 格式的元数据`)
}

/** 自检：验证 JPEG 剥离逻辑 */
export function selfCheck(): string[] {
  const errors: string[] = []
  try {
    // 构造一个最小 JPEG：SOI + APP1(EXIF) + DQT + SOF0 + DHT + SOS + 数据 + EOI
    const bytes = new Uint8Array([
      0xff, 0xd8, // SOI
      0xff, 0xe1, 0x00, 0x08, 0x45, 0x78, 0x69, 0x66, 0x00, // APP1 EXIF (6字节内容)
      0xff, 0xdb, 0x00, 0x04, 0x00, 0x01, // DQT
      0xff, 0xc0, 0x00, 0x0b, 0x08, 0x00, 0x02, 0x00, 0x02, 0x01, 0x01, 0x11, 0x00, // SOF0
      0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, // SOS
      0x12, 0x34, 0x56, 0x78, // 压缩数据
      0xff, 0xd9, // EOI
    ])
    const clean = stripJpegExif(bytes)
    const cleanStr = String.fromCharCode(...clean)
    if (cleanStr.includes('Exif')) errors.push('EXIF 段未被剥离')
    if (!cleanStr.includes('\xff\xda')) errors.push('SOS 段丢失')
    if (!cleanStr.includes('\xff\xc0')) errors.push('SOF 段丢失')
    if (clean[clean.length - 1] !== 0xd9) errors.push('EOI 缺失')
  } catch (e) {
    errors.push(`selfCheck 异常: ${e instanceof Error ? e.message : String(e)}`)
  }
  return errors
}
