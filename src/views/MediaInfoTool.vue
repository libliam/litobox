<template>
  <div class="tool-container">
    <!-- ffmpeg 状态横幅 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': ffmpegAvailable, 'ffmpeg-missing': !ffmpegAvailable }" v-if="ffmpegChecked">
      <template v-if="ffmpegAvailable">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，所有功能可用
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        此功能需要 ffmpeg，请先安装。
        <span class="ffmpeg-tip">
          安装命令：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
        </span>
      </template>
    </div>

    <!-- 文件选择卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">选择媒体文件</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" size="small" @click="selectFile" :loading="isLoading" :disabled="!ffmpegAvailable">
              选择文件
            </el-button>
            <el-button size="small" @click="clearInfo" v-if="mediaInfo">
              清除
            </el-button>
          </div>
        </div>
        <div v-if="mediaInfo" class="file-info">
          <span class="file-name">{{ mediaInfo.structured.file_name }}</span>
          <span class="file-detail">{{ formatFileSize(mediaInfo.structured.file_size) }}</span>
        </div>
      </div>
    </div>

    <!-- 错误信息 -->
    <div v-if="errorMessage" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ errorMessage }}</div>
      </div>
    </div>

    <!-- 容器信息 -->
    <div v-if="mediaInfo" class="tool-card">
      <div class="card-header">
        <span class="card-title">容器信息</span>
        <div class="card-actions">
          <el-button size="small" @click="copyFormatInfo">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">格式名称</span>
            <span class="info-value">{{ mediaInfo.structured.format.format_long_name || mediaInfo.structured.format.format_name }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(mediaInfo.structured.format.duration) }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">文件大小</span>
            <span class="info-value">{{ formatFileSize(mediaInfo.structured.file_size) }} ({{ mediaInfo.structured.file_size.toLocaleString() }} bytes)</span>
          </div>
          <div class="info-row">
            <span class="info-key">总比特率</span>
            <span class="info-value">{{ (mediaInfo.structured.format.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row">
            <span class="info-key">流数量</span>
            <span class="info-value">{{ mediaInfo.structured.format.stream_count }} (视频×{{ mediaInfo.structured.video_streams.length }} + 音频×{{ mediaInfo.structured.audio_streams.length }} + 字幕×{{ mediaInfo.structured.subtitle_streams.length }})</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 视频流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.video_streams" :key="'video-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">视频流 #{{ stream.index }}</span>
        <div class="card-actions">
          <el-button size="small" @click="copyVideoStreamInfo(stream)">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_long_name || stream.codec_name }}{{ stream.profile ? ` (${stream.profile}${stream.level ? `, Level ${stream.level}` : ''})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">分辨率</span>
            <span class="info-value">{{ stream.width }}×{{ stream.height }}{{ stream.display_aspect_ratio ? ` (${stream.display_aspect_ratio})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">帧率</span>
            <span class="info-value">{{ stream.fps.toFixed(3) }} fps</span>
          </div>
          <div class="info-row">
            <span class="info-key">像素格式</span>
            <span class="info-value">{{ stream.pix_fmt }}{{ stream.bit_depth ? ` (${stream.bit_depth} bit)` : '' }}</span>
          </div>
          <div class="info-row" v-if="stream.color_space || stream.color_primaries || stream.color_transfer">
            <span class="info-key">色彩空间</span>
            <span class="info-value">{{ stream.color_space || '未知' }} / {{ stream.color_primaries || '未知' }} / {{ stream.color_transfer || '未知' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">比特率</span>
            <span class="info-value">{{ (stream.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row" v-if="stream.nb_frames > 0">
            <span class="info-key">帧数</span>
            <span class="info-value">{{ stream.nb_frames.toLocaleString() }}</span>
          </div>
          <div class="info-row" v-if="stream.duration > 0">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(stream.duration) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'vtag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
          <div class="info-row" v-for="(ext, eidx) in stream.extra" :key="'vext-' + eidx">
            <span class="info-key">{{ ext.key }}</span>
            <span class="info-value">{{ ext.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 音频流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.audio_streams" :key="'audio-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">音频流 #{{ stream.index }}</span>
        <div class="card-actions">
          <el-button size="small" @click="copyAudioStreamInfo(stream)">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_long_name || stream.codec_name }}{{ stream.profile ? ` (${stream.profile})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">采样率</span>
            <span class="info-value">{{ stream.sample_rate.toLocaleString() }} Hz</span>
          </div>
          <div class="info-row">
            <span class="info-key">声道</span>
            <span class="info-value">{{ stream.channel_layout || (stream.channels === 2 ? '立体声' : stream.channels === 1 ? '单声道' : `${stream.channels} 声道`) }}</span>
          </div>
          <div class="info-row" v-if="stream.bit_depth">
            <span class="info-key">位深度</span>
            <span class="info-value">{{ stream.bit_depth }} bit</span>
          </div>
          <div class="info-row">
            <span class="info-key">比特率</span>
            <span class="info-value">{{ (stream.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row" v-if="stream.duration > 0">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(stream.duration) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'atag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
          <div class="info-row" v-for="(ext, eidx) in stream.extra" :key="'aext-' + eidx">
            <span class="info-key">{{ ext.key }}</span>
            <span class="info-value">{{ ext.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 字幕流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.subtitle_streams" :key="'sub-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">字幕流 #{{ stream.index }}</span>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_long_name || stream.codec_name }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'stag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 其他流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.other_streams" :key="'other-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ stream.codec_type }} 流 #{{ stream.index }}</span>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_name }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'otag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 元数据 -->
    <div v-if="mediaInfo && mediaInfo.structured.metadata.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">元数据</span>
        <div class="card-actions">
          <el-button size="small" @click="copyMetadata">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row" v-for="(meta, idx) in mediaInfo.structured.metadata" :key="'meta-' + idx">
            <span class="info-key">{{ meta.key }}</span>
            <span class="info-value">{{ meta.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 章节信息 -->
    <div v-if="mediaInfo && mediaInfo.structured.chapters.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">章节</span>
      </div>
      <div class="card-body">
        <div class="chapter-list">
          <div v-for="(chap, idx) in mediaInfo.structured.chapters" :key="'chap-' + idx" class="chapter-item">
            <span class="chapter-time">{{ formatDuration(chap.start_time) }} - {{ formatDuration(chap.end_time) }}</span>
            <span class="chapter-title">{{ chap.title || `章节 ${idx + 1}` }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 原始 JSON -->
    <div v-if="mediaInfo" class="tool-card">
      <div class="card-header">
        <span class="card-title">原始 JSON</span>
        <div class="card-actions">
          <el-button size="small" @click="toggleRawJson">
            {{ rawJsonVisible ? '收起' : '展开' }}
          </el-button>
          <el-button size="small" @click="copyRawJson">复制</el-button>
        </div>
      </div>
      <div class="card-body" v-if="rawJsonVisible">
        <pre class="raw-json">{{ mediaInfo.raw }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

interface KeyValue {
  key: string
  value: string
}

interface FormatInfo {
  format_name: string
  format_long_name: string
  duration: number
  bitrate: number
  stream_count: number
  nb_programs: number
}

interface VideoStreamInfo {
  index: number
  codec_name: string
  codec_long_name: string
  profile: string
  level: number | null
  width: number
  height: number
  coded_width: number
  coded_height: number
  display_aspect_ratio: string
  sample_aspect_ratio: string
  pix_fmt: string
  color_space: string
  color_primaries: string
  color_transfer: string
  color_range: string
  chroma_location: string
  field_order: string
  fps: number
  avg_fps: number
  bitrate: number
  bit_depth: number | null
  duration: number
  nb_frames: number
  disposition: string[]
  tags: KeyValue[]
  extra: KeyValue[]
}

interface AudioStreamInfo {
  index: number
  codec_name: string
  codec_long_name: string
  profile: string
  sample_rate: number
  channels: number
  channel_layout: string
  sample_fmt: string
  bit_depth: number | null
  bitrate: number
  duration: number
  nb_frames: number
  disposition: string[]
  tags: KeyValue[]
  extra: KeyValue[]
}

interface SubtitleStreamInfo {
  index: number
  codec_name: string
  codec_long_name: string
  tags: KeyValue[]
}

interface OtherStreamInfo {
  index: number
  codec_type: string
  codec_name: string
  tags: KeyValue[]
}

interface ChapterInfo {
  id: number
  start_time: number
  end_time: number
  title: string
  tags: KeyValue[]
}

interface StructuredMediaInfo {
  file_path: string
  file_name: string
  file_size: number
  format: FormatInfo
  video_streams: VideoStreamInfo[]
  audio_streams: AudioStreamInfo[]
  subtitle_streams: SubtitleStreamInfo[]
  other_streams: OtherStreamInfo[]
  metadata: KeyValue[]
  chapters: ChapterInfo[]
}

interface MediaInfoResult {
  structured: StructuredMediaInfo
  raw: string
}

const ffmpegChecked = ref(false)
const ffmpegAvailable = ref(false)
const isLoading = ref(false)
const mediaInfo = ref<MediaInfoResult | null>(null)
const errorMessage = ref('')
const rawJsonVisible = ref(false)

onMounted(async () => {
  await checkFfmpeg()
})

async function checkFfmpeg() {
  try {
    const available = await invoke<boolean>('check_ffmpeg_available')
    ffmpegAvailable.value = available
  } catch (e) {
    ffmpegAvailable.value = false
  } finally {
    ffmpegChecked.value = true
  }
}

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [{
      name: '媒体文件',
      extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'm4v']
    }]
  })

  if (!selected) return

  await loadMediaInfo(selected as string)
}

async function loadMediaInfo(path: string) {
  isLoading.value = true
  errorMessage.value = ''
  mediaInfo.value = null

  try {
    const result = await invoke<MediaInfoResult>('get_media_info', { path })
    mediaInfo.value = result

    // 记录历史
    const format = result.structured.format.format_name.split(',')[0].toUpperCase()
    const videoInfo = result.structured.video_streams[0]
    const resolution = videoInfo ? `${videoInfo.width}x${videoInfo.height}` : ''
    const codec = videoInfo ? videoInfo.codec_name.toUpperCase() : ''
    const duration = formatDuration(result.structured.format.duration)

    store.addHistory({
      tool: 'mediaInfo',
      action: '查看媒体信息',
      inputPreview: result.structured.file_name.slice(0, 50),
      outputPreview: `${format} | ${resolution} | ${codec} | ${duration}`,
      inputFull: path,
      outputFull: JSON.stringify(result.structured, null, 2),
    })

    ElMessage.success('媒体信息加载成功')
  } catch (e) {
    errorMessage.value = String(e)
    ElMessage.error('加载失败')
  } finally {
    isLoading.value = false
  }
}

function clearInfo() {
  mediaInfo.value = null
  errorMessage.value = ''
  rawJsonVisible.value = false
}

function toggleRawJson() {
  rawJsonVisible.value = !rawJsonVisible.value
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

function formatDuration(seconds: number): string {
  if (seconds <= 0) return '00:00:00'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  const ms = Math.floor((seconds % 1) * 1000)
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(3, '0')}`
}

function formatStreamInfo(stream: any, type: string): string {
  const lines: string[] = []
  lines.push(`类型: ${type}`)
  lines.push(`索引: #${stream.index}`)
  lines.push(`编解码器: ${stream.codec_long_name || stream.codec_name}`)

  if (type === '视频流') {
    lines.push(`分辨率: ${stream.width}×${stream.height}`)
    lines.push(`帧率: ${stream.fps.toFixed(3)} fps`)
    lines.push(`像素格式: ${stream.pix_fmt}`)
    lines.push(`比特率: ${(stream.bitrate / 1000).toFixed(0)} kbps`)
  } else if (type === '音频流') {
    lines.push(`采样率: ${stream.sample_rate} Hz`)
    lines.push(`声道: ${stream.channel_layout || stream.channels}`)
    lines.push(`比特率: ${(stream.bitrate / 1000).toFixed(0)} kbps`)
  }

  if (stream.profile) lines.push(`Profile: ${stream.profile}`)
  if (stream.tags && stream.tags.length > 0) {
    lines.push('标签:')
    stream.tags.forEach((t: KeyValue) => lines.push(`  ${t.key}: ${t.value}`))
  }
  if (stream.extra && stream.extra.length > 0) {
    lines.push('其他:')
    stream.extra.forEach((t: KeyValue) => lines.push(`  ${t.key}: ${t.value}`))
  }

  return lines.join('\n')
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch (e) {
    ElMessage.error('复制失败')
  }
}

function copyFormatInfo() {
  if (!mediaInfo.value) return
  const f = mediaInfo.value.structured.format
  const text = `格式: ${f.format_long_name || f.format_name}
时长: ${formatDuration(f.duration)}
大小: ${formatFileSize(mediaInfo.value.structured.file_size)}
比特率: ${(f.bitrate / 1000).toFixed(0)} kbps
流数量: ${f.stream_count}`
  copyToClipboard(text)
}

function copyVideoStreamInfo(stream: VideoStreamInfo) {
  copyToClipboard(formatStreamInfo(stream, '视频流'))
}

function copyAudioStreamInfo(stream: AudioStreamInfo) {
  copyToClipboard(formatStreamInfo(stream, '音频流'))
}

function copyMetadata() {
  if (!mediaInfo.value) return
  const lines = mediaInfo.value.structured.metadata.map(m => `${m.key}: ${m.value}`)
  copyToClipboard(lines.join('\n'))
}

function copyRawJson() {
  if (!mediaInfo.value) return
  copyToClipboard(mediaInfo.value.raw)
}
</script>

<style scoped>
.ffmpeg-banner {
  padding: 12px 16px;
  border-radius: 6px;
  margin-bottom: 16px;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ffmpeg-detected {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid var(--accent-green, #10b981);
  color: var(--accent-green, #10b981);
}

.ffmpeg-missing {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red, #ef4444);
  color: var(--accent-red, #ef4444);
}

.ffmpeg-icon {
  font-size: 16px;
}

.ffmpeg-tip {
  margin-left: auto;
  font-size: 13px;
}

.ffmpeg-cmd {
  background: rgba(0, 0, 0, 0.2);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.file-info {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-name {
  font-weight: 600;
  color: var(--text-primary, #e2e8f0);
}

.file-detail {
  color: var(--text-secondary, #94a3b8);
  font-size: 13px;
}

.error-message {
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red, #ef4444);
  border-radius: 4px;
  color: var(--accent-red, #ef4444);
  font-size: 13px;
  line-height: 1.5;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}

.info-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-key {
  color: var(--text-secondary, #94a3b8);
  font-size: 12px;
  font-weight: 500;
}

.info-value {
  color: var(--text-primary, #e2e8f0);
  font-size: 14px;
  word-break: break-all;
}

.chapter-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chapter-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input, #0d1520);
  border-radius: 4px;
}

.chapter-time {
  color: var(--accent-cyan, #00d4ff);
  font-family: 'Courier New', monospace;
  font-size: 13px;
  min-width: 180px;
}

.chapter-title {
  color: var(--text-primary, #e2e8f0);
  font-size: 14px;
}

.raw-json {
  background: var(--bg-input, #0d1520);
  padding: 16px;
  border-radius: 4px;
  color: var(--text-primary, #e2e8f0);
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 500px;
  overflow-y: auto;
}
</style>
