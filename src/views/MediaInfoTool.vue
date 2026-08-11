<template>
  <div class="tool-container">
    <!-- ffmpeg 状态横幅 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': ffmpegAvailable, 'ffmpeg-missing': !ffmpegAvailable }" v-if="ffmpegChecked">
      <template v-if="ffmpegAvailable">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，媒体信息分析功能可用
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        未检测到 ffmpeg，此功能需要 ffmpeg 才能使用。
        <span class="ffmpeg-tip">
          安装 ffmpeg：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
          <a href="https://www.wikihow.com/Install-FFmpeg-on-Windows" target="_blank" class="ffmpeg-link">详细教程</a>
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
            <span class="info-value">{{ translateFormatName(mediaInfo.structured.format.format_long_name) || translateFormatName(mediaInfo.structured.format.format_name) }}</span>
          </div>
          <div class="info-row" v-if="mediaInfo.structured.format.duration > 0">
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

    <!-- 图片文件原图预览 -->
    <div v-if="mediaInfo && imagePreviewUrl" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片预览</span>
      </div>
      <div class="card-body">
        <div class="image-preview-wrap">
          <img :src="imagePreviewUrl" class="image-preview" @click="showImagePreview" alt="图片预览" />
        </div>
      </div>
    </div>

    <!-- 视频流 / 封面图信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.video_streams" :key="'video-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ isCoverArt(stream) ? '封面图' : (isImageFile(mediaInfo?.structured.file_path || '') ? '图像流' : '视频流') }} #{{ stream.index }}</span>
        <div class="card-actions">
          <el-button size="small" @click="copyVideoStreamInfo(stream)">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <!-- 封面图预览 -->
        <div v-if="isCoverArt(stream)" class="cover-preview">
          <img
            v-if="coverArtUrls[idx]"
            :src="coverArtUrls[idx]"
            class="cover-thumb"
            @click="showCoverPreview(idx)"
            alt="封面图"
          />
          <div v-else-if="coverArtLoading[idx]" class="cover-loading">
            <el-icon class="is-loading"><Loading /></el-icon>
            <span>加载中...</span>
          </div>
          <div v-else class="cover-error">封面图加载失败</div>
        </div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">
              编解码器
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>视频/音频使用的压缩算法</p>
                    <p>常见格式：H.264、H.265、VP9、AV1（视频）</p>
                    <p>AAC、MP3、FLAC、Opus（音频）</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ translateCodecName(stream.codec_name) }}{{ stream.profile && !isCoverArt(stream) ? ` (${translateProfile(stream.profile)}${stream.level ? `, Level ${stream.level}` : ''})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">
              分辨率
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>视频画面的宽度和高度（像素）</p>
                    <p>常见分辨率：1920×1080（1080p）、3840×2160（4K）</p>
                    <p>宽高比影响画面形状，如 16:9、4:3</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ stream.width }}×{{ stream.height }}{{ stream.display_aspect_ratio ? ` (${stream.display_aspect_ratio})` : '' }}</span>
          </div>
          <template v-if="!isCoverArt(stream)">
            <div class="info-row" v-if="stream.fps > 0">
              <span class="info-key">
                帧率
                <el-tooltip placement="top" effect="dark">
                  <template #content>
                    <div class="tooltip-content">
                      <p>每秒显示的画面数量（FPS）</p>
                      <p>常见帧率：24fps（电影）、30fps（电视）、60fps（游戏）</p>
                      <p>帧率越高，画面越流畅</p>
                    </div>
                  </template>
                  <el-icon class="hint-icon"><QuestionFilled /></el-icon>
                </el-tooltip>
              </span>
              <span class="info-value">{{ stream.fps.toFixed(3) }} fps</span>
            </div>
            <div class="info-row" v-if="stream.color_space || stream.color_primaries || stream.color_transfer">
              <span class="info-key">
                色彩空间
                <el-tooltip placement="top" effect="dark">
                  <template #content>
                    <div class="tooltip-content">
                      <p>定义颜色的表示方式</p>
                      <p>• 色彩空间：BT.709（高清）、BT.2020（HDR）</p>
                      <p>• 色域：定义可显示的颜色范围</p>
                      <p>• 传输特性：定义亮度与电压的关系</p>
                    </div>
                  </template>
                  <el-icon class="hint-icon"><QuestionFilled /></el-icon>
                </el-tooltip>
              </span>
              <span class="info-value">{{ translateColorSpace(stream.color_space) || '未知' }} / {{ stream.color_primaries || '未知' }} / {{ translateColorTransfer(stream.color_transfer) || '未知' }}</span>
            </div>
            <div class="info-row" v-if="stream.bitrate > 0">
              <span class="info-key">
                比特率
                <el-tooltip placement="top" effect="dark">
                  <template #content>
                    <div class="tooltip-content">
                      <p>每秒传输的数据量（kbps）</p>
                      <p>比特率越高，画质/音质越好，文件越大</p>
                      <p>常见：视频 2000-8000 kbps，音频 128-320 kbps</p>
                    </div>
                  </template>
                  <el-icon class="hint-icon"><QuestionFilled /></el-icon>
                </el-tooltip>
              </span>
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
          </template>
          <div class="info-row">
            <span class="info-key">
              像素格式
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>定义每个像素的存储方式</p>
                    <p>• YUV 4:2:0：最常用，压缩率高</p>
                    <p>• YUV 4:2:2/4:4:4：专业级，色彩更精确</p>
                    <p>• RGB：直接存储红绿蓝，文件较大</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ translatePixFmt(stream.pix_fmt) }}{{ stream.bit_depth ? ` (${stream.bit_depth} bit)` : '' }}</span>
          </div>
          <div class="info-row" v-if="stream.field_order">
            <span class="info-key">
              场序
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>定义隔行扫描的场顺序</p>
                    <p>• 逐行扫描（progressive）：现代标准</p>
                    <p>• 顶场优先（tt）：传统电视标准</p>
                    <p>• 底场优先（bb）：另一种隔行方式</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ translateFieldOrder(stream.field_order) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'vtag-' + tidx">
            <span class="info-key">{{ translateTag(tag.key) }}</span>
            <span class="info-value">{{ translateTag(tag.value) }}</span>
          </div>
          <div class="info-row" v-for="(ext, eidx) in stream.extra" :key="'vext-' + eidx">
            <span class="info-key">{{ translateTag(ext.key) }}</span>
            <span class="info-value">{{ translateTag(ext.value) }}</span>
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
            <span class="info-key">
              编解码器
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>音频使用的压缩算法</p>
                    <p>• 无损：FLAC、ALAC、WAV（音质完美）</p>
                    <p>• 有损：AAC、MP3、Opus（文件更小）</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ translateCodecName(stream.codec_name) }}{{ stream.profile ? ` (${translateProfile(stream.profile)})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">
              采样率
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>每秒采集的声音样本数（Hz）</p>
                    <p>• 44100 Hz：CD 音质标准</p>
                    <p>• 48000 Hz：DVD/蓝光标准</p>
                    <p>• 96000+ Hz：高解析度音频</p>
                    <p>采样率越高，高频还原越好</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ stream.sample_rate.toLocaleString() }} Hz</span>
          </div>
          <div class="info-row">
            <span class="info-key">
              声道
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>音频的声道数量和布局</p>
                    <p>• 单声道（mono）：1 个声道</p>
                    <p>• 立体声（stereo）：2 个声道，左右</p>
                    <p>• 5.1/7.1：环绕声，多声道布局</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ stream.channel_layout || (stream.channels === 2 ? '立体声' : stream.channels === 1 ? '单声道' : `${stream.channels} 声道`) }}</span>
          </div>
          <div class="info-row" v-if="stream.bit_depth">
            <span class="info-key">
              位深度
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>每个采样点的精度（bit）</p>
                    <p>• 16 bit：CD 标准，动态范围 96 dB</p>
                    <p>• 24 bit：专业录音，动态范围 144 dB</p>
                    <p>位深度越高，细节越丰富，底噪越低</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ stream.bit_depth }} bit</span>
          </div>
          <div class="info-row" v-if="stream.sample_fmt">
            <span class="info-key">
              采样格式
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>音频数据的内部存储方式</p>
                    <p>• fltp：32 位浮点平面（ffmpeg 内部常用）</p>
                    <p>• s16/s32：整数格式</p>
                    <p>• 平面（p 后缀）：各声道数据分开存储</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ translateSampleFmt(stream.sample_fmt) }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">
              比特率
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>每秒音频数据量（kbps）</p>
                    <p>• 128 kbps：MP3 标准音质</p>
                    <p>• 320 kbps：MP3 最高音质</p>
                    <p>• 1000+ kbps：无损音频（FLAC）</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </span>
            <span class="info-value">{{ (stream.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row" v-if="stream.duration > 0">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(stream.duration) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'atag-' + tidx">
            <span class="info-key">{{ translateTag(tag.key) }}</span>
            <span class="info-value">{{ translateTag(tag.value) }}</span>
          </div>
          <div class="info-row" v-for="(ext, eidx) in stream.extra" :key="'aext-' + eidx">
            <span class="info-key">{{ translateTag(ext.key) }}</span>
            <span class="info-value">{{ translateTag(ext.value) }}</span>
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
            <span class="info-value">{{ translateCodecName(stream.codec_name) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'stag-' + tidx">
            <span class="info-key">{{ translateTag(tag.key) }}</span>
            <span class="info-value">{{ translateTag(tag.value) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 其他流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.other_streams" :key="'other-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ translateTag(stream.codec_type) }} 流 #{{ stream.index }}</span>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_name }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'otag-' + tidx">
            <span class="info-key">{{ translateTag(tag.key) }}</span>
            <span class="info-value">{{ translateTag(tag.value) }}</span>
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
            <span class="info-key">{{ translateTag(meta.key) }}</span>
            <span class="info-value">{{ translateTag(meta.value) }}</span>
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

    <!-- 封面图预览弹窗 -->
    <el-dialog
      v-model="coverPreviewVisible"
      title="图片预览"
      width="80%"
      center
      @close="coverPreviewUrl = ''"
    >
      <div class="cover-preview-large">
        <img :src="coverPreviewUrl" alt="封面图" />
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Loading } from '@element-plus/icons-vue'
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

// 图片文件扩展名（ffprobe 支持的静态/动态图片格式）
const IMAGE_EXTENSIONS = ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp', 'tif', 'tiff', 'avif', 'ico']

function isImageFile(path: string): boolean {
  const ext = path.split('.').pop()?.toLowerCase() ?? ''
  return IMAGE_EXTENSIONS.includes(ext)
}

// 图片文件原图预览（data URL）
const imagePreviewUrl = ref('')

// 封面图相关
const coverArtUrls = ref<Record<number, string>>({})
const coverArtLoading = ref<Record<number, boolean>>({})
const coverPreviewVisible = ref(false)
const coverPreviewUrl = ref('')

onMounted(async () => {
  await checkFfmpeg()
})

async function checkFfmpeg() {
  try {
    const available = await invoke<boolean>('check_ffmpeg')
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
    filters: [
      // 默认选中第一项：所有文件
      { name: '所有文件', extensions: ['*'] },
      { name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'm4v', 'mpg', 'mpeg', 'ts', 'm2ts'] },
      { name: '音频文件', extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'wma', 'opus', 'amr'] },
      { name: '图片文件', extensions: IMAGE_EXTENSIONS },
    ]
  })

  if (!selected) return

  await loadMediaInfo(selected as string)
}

async function loadMediaInfo(path: string) {
  isLoading.value = true
  errorMessage.value = ''
  mediaInfo.value = null
  coverArtUrls.value = {}
  coverArtLoading.value = {}

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

    // 加载封面图
    await loadCoverArts(path)
    // 图片文件直接展示原图预览
    await loadImagePreview(path)
  } catch (e) {
    errorMessage.value = String(e)
    ElMessage.error('加载失败')
  } finally {
    isLoading.value = false
  }
}

async function loadCoverArts(filePath: string) {
  if (!mediaInfo.value) return
  
  const coverStreams = mediaInfo.value.structured.video_streams.filter(isCoverArt)
  if (coverStreams.length === 0) return

  // 用循环索引（而非 stream.index）作为 key，与模板 v-for 的 idx 对应
  const allStreams = mediaInfo.value.structured.video_streams
  for (let loopIdx = 0; loopIdx < allStreams.length; loopIdx++) {
    const stream = allStreams[loopIdx]
    if (!isCoverArt(stream)) continue
    
    coverArtLoading.value[loopIdx] = true
    try {
      const tempPath = await invoke<string>('extract_cover_art', { filePath })
      const base64 = await invoke<string>('read_file_base64', { filePath: tempPath })
      coverArtUrls.value[loopIdx] = `data:image/jpeg;base64,${base64}`
    } catch (e) {
      console.error('封面图加载失败:', e)
    } finally {
      coverArtLoading.value[loopIdx] = false
    }
  }
}

function showCoverPreview(loopIdx: number) {
  const url = coverArtUrls.value[loopIdx]
  if (url) {
    coverPreviewUrl.value = url
    coverPreviewVisible.value = true
  }
}

/** 图片文件原图预览：后端读文件转 base64，前端以 data URL 展示 */
async function loadImagePreview(filePath: string) {
  if (!isImageFile(filePath)) return
  try {
    const base64 = await invoke<string>('read_file_base64', { filePath })
    const ext = filePath.split('.').pop()?.toLowerCase() ?? 'png'
    const mime = ext === 'jpg' ? 'jpeg' : ext
    imagePreviewUrl.value = `data:image/${mime};base64,${base64}`
  } catch (e) {
    console.error('图片预览加载失败:', e)
  }
}

function showImagePreview() {
  if (imagePreviewUrl.value) {
    coverPreviewUrl.value = imagePreviewUrl.value
    coverPreviewVisible.value = true
  }
}

function clearInfo() {
  mediaInfo.value = null
  errorMessage.value = ''
  rawJsonVisible.value = false
  coverArtUrls.value = {}
  coverArtLoading.value = {}
  coverPreviewVisible.value = false
  coverPreviewUrl.value = ''
  imagePreviewUrl.value = ''
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

/** ffprobe 标签键名翻译 */
const TAG_KEY_MAP: Record<string, string> = {
  language: '语言', title: '标题', encoder: '编码器', handler_name: '处理器名称',
  creation_time: '创建时间', artist: '艺术家', album: '专辑', genre: '类型',
  date: '日期', comment: '注释', copyright: '版权', description: '描述',
  album_artist: '专辑艺术家', track: '曲目', disc: '碟片',
  major_brand: '主要品牌', minor_version: '次要版本', compatible_brands: '兼容品牌',
  encoder_settings: '编码设置', lyrics: '歌词', composer: '作曲',
  conductor: '指挥', performer: '表演者', publisher: '发行商',
  encoded_by: '编码者', original_filename: '原始文件名',
  media_type: '媒体类型', mimetype: 'MIME类型',
}

/** disposition / 值翻译 */
const TAG_VALUE_MAP: Record<string, string> = {
  default: '默认', forced: '强制', dub: '配音', original: '原始',
  comment: '评论', lyrics: '歌词', karaoke: '卡拉 OK',
  hearing_impaired: '听力障碍', visual_impaired: '视觉障碍',
  clean_effects: '清洁效果', attached_pic: '附带图片',
  timed_thumbnails: '定时缩略图', non_diegetic: '非剧情音',
  stereo: '立体声', mono: '单声道',
  unknown: '未知',
  data: '数据', attachment: '附件',
}

/** 格式名称翻译 */
const FORMAT_NAME_MAP: Record<string, string> = {
  mp4: 'MP4', mov: 'QuickTime', avi: 'AVI', mkv: 'Matroska',
  webm: 'WebM', flv: 'Flash Video', wmv: 'Windows Media',
  mp3: 'MP3', wav: 'WAV', flac: 'FLAC', aac: 'AAC',
  ogg: 'Ogg Vorbis', m4a: 'M4A', m4v: 'M4V',
  'is,om,iso2,mp41': 'MP4 (ISO Base)',
  'image2 sequence': '图片', image2: '图片', png: 'PNG', gif: 'GIF',
  webp: 'WebP', bmp: 'BMP', tiff: 'TIFF', ico: 'ICO', avif: 'AVIF',
}

/** 编解码器名称翻译 */
const CODEC_NAME_MAP: Record<string, string> = {
  h264: 'H.264/AVC', hevc: 'H.265/HEVC', h265: 'H.265/HEVC',
  vp8: 'VP8', vp9: 'VP9', av1: 'AV1',
  mpeg4: 'MPEG-4', mpegvideo: 'MPEG 视频',
  aac: 'AAC', mp3: 'MP3', flac: 'FLAC', vorbis: 'Vorbis',
  opus: 'Opus', ac3: 'AC-3', eac3: 'E-AC-3', dts: 'DTS',
  pcm_s16le: 'PCM 16 位 LE', pcm_s24le: 'PCM 24 位 LE', pcm_f32le: 'PCM 32 位浮点',
  subrip: 'SubRip (SRT)', ass: 'ASS/SSA', srt: 'SRT',
  png: 'PNG', mjpeg: 'MJPEG/JPEG', gif: 'GIF', webp: 'WebP',
  bmp: 'BMP', tiff: 'TIFF', targa: 'TGA',
}

/** 像素格式翻译 */
const PIX_FMT_MAP: Record<string, string> = {
  yuv420p: 'YUV 4:2:0', yuv422p: 'YUV 4:2:2', yuv444p: 'YUV 4:4:4',
  yuv420p10le: 'YUV 4:2:0 10 位', yuv422p10le: 'YUV 4:2:2 10 位', yuv444p10le: 'YUV 4:4:4 10 位',
  rgb24: 'RGB 24 位', rgba: 'RGBA', bgr24: 'BGR 24 位', bgra: 'BGRA', pal8: '8 位调色板',
  nv12: 'NV12', nv21: 'NV21',
}

/** 色彩空间翻译 */
const COLOR_SPACE_MAP: Record<string, string> = {
  bt709: 'BT.709', bt2020: 'BT.2020', bt470bg: 'BT.470 BG',
  smpte170m: 'SMPTE 170M', smpte240m: 'SMPTE 240M',
}

/** 色彩传输特性翻译 */
const COLOR_TRANSFER_MAP: Record<string, string> = {
  bt709: 'BT.709', bt2020_10: 'BT.2020 10 位', bt2020_12: 'BT.2020 12 位',
  smpte2084: 'SMPTE 2084 (PQ)', arib_std_b67: 'ARIB STD-B67 (HLG)',
  iec61966_2_1: 'sRGB', iec61966_2_4: 'xvYCC',
}

/** Profile 翻译 */
const PROFILE_MAP: Record<string, string> = {
  High: 'High', Main: 'Main', Baseline: 'Baseline',
  'High 10': 'High 10', 'High 4:4:4': 'High 4:4:4',
  'Main 10': 'Main 10',
}

/** 采样格式翻译 */
const SAMPLE_FMT_MAP: Record<string, string> = {
  fltp: '浮点平面', flt: '浮点',
  s16: '16 位有符号', s32: '32 位有符号',
  s16p: '16 位有符号平面', s32p: '32 位有符号平面',
  dbl: '双精度浮点', dblp: '双精度浮点平面',
}

/** 场序翻译 */
const FIELD_ORDER_MAP: Record<string, string> = {
  progressive: '逐行扫描', tt: '顶场优先', bb: '底场优先',
  tb: '顶底交错', bt: '底顶交错',
}

function translateTag(text: string): string {
  return TAG_KEY_MAP[text.toLowerCase()] || TAG_VALUE_MAP[text.toLowerCase()] || text
}

function translateFormatName(name: string): string {
  return FORMAT_NAME_MAP[name.toLowerCase()] || name
}

function translateCodecName(name: string): string {
  return CODEC_NAME_MAP[name.toLowerCase()] || name
}

function translatePixFmt(fmt: string): string {
  return PIX_FMT_MAP[fmt.toLowerCase()] || fmt
}

function translateColorSpace(cs: string): string {
  return COLOR_SPACE_MAP[cs.toLowerCase()] || cs
}

function translateColorTransfer(ct: string): string {
  return COLOR_TRANSFER_MAP[ct.toLowerCase()] || ct
}

function translateProfile(profile: string): string {
  return PROFILE_MAP[profile] || profile
}

function translateSampleFmt(fmt: string): string {
  return SAMPLE_FMT_MAP[fmt.toLowerCase()] || fmt
}

function translateFieldOrder(order: string): string {
  return FIELD_ORDER_MAP[order.toLowerCase()] || order
}

/** 判断视频流是否为内嵌封面图 */
function isCoverArt(stream: VideoStreamInfo): boolean {
  return stream.disposition.includes('attached_pic')
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
  lines.push(`编解码器: ${translateCodecName(stream.codec_name)}`)

  if (type === '视频流') {
    lines.push(`分辨率: ${stream.width}×${stream.height}`)
    lines.push(`帧率: ${stream.fps.toFixed(3)} fps`)
    lines.push(`像素格式: ${translatePixFmt(stream.pix_fmt)}`)
    lines.push(`比特率: ${(stream.bitrate / 1000).toFixed(0)} kbps`)
  } else if (type === '音频流') {
    lines.push(`采样率: ${stream.sample_rate} Hz`)
    lines.push(`声道: ${stream.channel_layout || stream.channels}`)
    lines.push(`比特率: ${(stream.bitrate / 1000).toFixed(0)} kbps`)
  }

  if (stream.profile) lines.push(`Profile: ${translateProfile(stream.profile)}`)
  if (stream.tags && stream.tags.length > 0) {
    lines.push('标签:')
    stream.tags.forEach((t: KeyValue) => lines.push(`  ${translateTag(t.key)}: ${translateTag(t.value)}`))
  }
  if (stream.extra && stream.extra.length > 0) {
    lines.push('其他:')
    stream.extra.forEach((t: KeyValue) => lines.push(`  ${translateTag(t.key)}: ${translateTag(t.value)}`))
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
  padding: 8px 16px;
  border-radius: 6px;
  margin-bottom: 12px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ffmpeg-banner.ffmpeg-detected {
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--accent-green);
}

.ffmpeg-banner.ffmpeg-missing {
  background: rgba(59, 130, 246, 0.12);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: var(--accent-blue);
}

.ffmpeg-link {
  color: var(--accent-cyan);
  margin-left: 4px;
}

.ffmpeg-link:hover {
  text-decoration: underline;
}

.ffmpeg-icon {
  font-size: 16px;
}

.ffmpeg-tip {
  margin-left: auto;
  font-size: 13px;
}

.ffmpeg-cmd {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--accent-orange);
  user-select: all;
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
  display: flex;
  align-items: center;
  gap: 4px;
}

.info-key .hint-icon {
  font-size: 14px;
  color: var(--text-secondary, #94a3b8);
  cursor: help;
  flex-shrink: 0;
}

.info-key .hint-icon:hover {
  color: var(--accent-cyan, #06b6d4);
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

/* 封面图预览 */
.cover-preview {
  display: flex;
  justify-content: center;
  padding: 16px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  margin-bottom: 12px;
}

.cover-thumb {
  width: 200px;
  height: 200px;
  object-fit: cover;
  border-radius: 8px;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  border: 2px solid rgba(255, 255, 255, 0.1);
}

.cover-thumb:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 20px rgba(6, 182, 212, 0.3);
  border-color: var(--accent-cyan, #06b6d4);
}

.cover-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary, #94a3b8);
  font-size: 13px;
}

.cover-error {
  color: var(--text-secondary, #94a3b8);
  font-size: 13px;
}

.cover-preview-large {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
}

.cover-preview-large img {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: 8px;
}

/* 图片文件原图预览 */
.image-preview-wrap {
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.image-preview {
  max-width: 100%;
  max-height: 400px;
  object-fit: contain;
  border-radius: 8px;
  cursor: pointer;
  border: 2px solid rgba(255, 255, 255, 0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}

.image-preview:hover {
  transform: scale(1.02);
  box-shadow: 0 4px 20px rgba(6, 182, 212, 0.3);
  border-color: var(--accent-cyan, #06b6d4);
}

/* Tooltip 内容 */
.tooltip-content {
  max-width: 280px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 4px 0;
  font-size: 13px;
}
</style>
