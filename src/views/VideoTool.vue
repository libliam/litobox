<template>
  <div class="tool-container">
    <!-- ffmpeg 状态横幅 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': useFfmpeg, 'ffmpeg-missing': !useFfmpeg }" v-if="ffmpegChecked">
      <template v-if="useFfmpeg">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，所有功能可用
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        未检测到 ffmpeg，仅支持 MP4 关键帧裁剪。
        <span class="ffmpeg-tip">
          安装 ffmpeg 可解锁全部功能：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
        </span>
      </template>
    </div>

    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="video-tool-tabs">
        <el-tab-pane label="视频裁剪" name="crop" />
        <el-tab-pane label="视频转码" name="transcode" />
        <el-tab-pane label="音频提取" name="audioExtract" />
        <el-tab-pane label="视频压缩" name="compress" />
        <el-tab-pane label="视频合并" name="merge" />
        <el-tab-pane label="截图提取" name="frameExtract" />
        <el-tab-pane label="画面裁剪" name="cropRegion" />
      </el-tabs>
    </div>

    <!-- ==================== Tab: 视频裁剪 ==================== -->
    <template v-if="activeTab === 'crop'">
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择视频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openFile" :loading="isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="filePath" class="video-file-info">
            <span class="file-name">{{ fileName }}</span>
            <span class="file-detail" v-if="videoInfo">
              {{ formatDuration(videoInfo.duration) }} | {{ videoInfo.width }}x{{ videoInfo.height }} |
              {{ videoInfo.codec.toUpperCase() }} | {{ videoInfo.fps.toFixed(1) }}fps |
              {{ videoInfo.bitrate }}kbps | {{ formatFileSize(videoInfo.file_size) }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">时间轴</span>
        </div>
        <div class="card-body">
          <div class="timeline-container" ref="timelineContainer" @contextmenu.prevent>
            <!-- 底层：缩略图（不重绘） -->
            <canvas ref="thumbnailCanvasRef" class="timeline-canvas thumbnail-layer"></canvas>
            <!-- 顶层：选中区域（拖拽时重绘） -->
            <canvas ref="canvasRef" class="timeline-canvas selection-layer" @mousedown="onCanvasMouseDown" @contextmenu.prevent></canvas>
            <div
              class="slider-handle start-handle"
              :style="{ left: timeToPercent(startTime) + '%' }"
              @mousedown.stop="onSliderMouseDown($event, 'start')"
            ></div>
            <div
              class="slider-handle end-handle"
              :style="{ left: timeToPercent(endTime) + '%' }"
              @mousedown.stop="onSliderMouseDown($event, 'end')"
            ></div>
          </div>
          <div class="timeline-labels">
            <span>{{ formatTime(startTime) }}</span>
            <span>{{ formatTime(endTime) }}</span>
          </div>
        </div>
      </div>

      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">裁剪设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">起始时间</div>
              <el-input-number
                v-model="startTime"
                :min="0"
                :max="endTime - 0.1"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 140px"
              />
              <span class="unit-text">秒</span>
            </div>
            <div class="action-group">
              <div class="group-label">结束时间</div>
              <el-input-number
                v-model="endTime"
                :min="startTime + 0.1"
                :max="videoInfo.duration"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 140px"
              />
              <span class="unit-text">秒</span>
            </div>
          </div>
          <div class="segment-info" v-if="videoInfo">
            片段时长: {{ formatDuration(segmentDuration) }}
          </div>
          <div v-if="actualRange" class="keyframe-hint">
            实际裁剪区间（关键帧对齐）: {{ formatTime(actualRange.start) }} - {{ formatTime(actualRange.end) }}
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="saveToSamePath" size="small">
                与源文件相同路径
              </el-checkbox>
            </div>
          </div>
        </div>
      </div>

      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="cropVideo" :loading="isProcessing" :disabled="!isRangeValid">
                裁剪并导出
              </el-button>
              <el-button size="small" @click="resetForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="isProcessing" :percentage="cropProgress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- ==================== Tab: 视频转码 ==================== -->
    <template v-if="activeTab === 'transcode'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            视频转码需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">选择视频文件</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="openTranscodeFile" :loading="isLoadingInfo">
                  打开文件
                </el-button>
              </div>
            </div>
            <div v-if="transcodeFilePath" class="video-file-info">
              <span class="file-name">{{ transcodeFileName }}</span>
              <span class="file-detail" v-if="transcodeVideoInfo">
                {{ formatDuration(transcodeVideoInfo.duration) }} | {{ transcodeVideoInfo.width }}x{{ transcodeVideoInfo.height }} |
                {{ transcodeVideoInfo.codec.toUpperCase() }} | {{ formatFileSize(transcodeVideoInfo.file_size) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="transcodeVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">转码设置</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">输出格式</div>
                <el-select v-model="transcodeFormat" size="small" style="width: 120px">
                  <el-option label="MP4" value="mp4" />
                  <el-option label="MKV" value="mkv" />
                  <el-option label="AVI" value="avi" />
                  <el-option label="MOV" value="mov" />
                  <el-option label="WebM" value="webm" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">视频编码器</div>
                <el-select v-model="transcodeVideoCodec" size="small" style="width: 120px">
                  <el-option label="H.264" value="libx264" />
                  <el-option label="H.265" value="libx265" />
                  <el-option label="VP9" value="libvpx-vp9" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">音频编码器</div>
                <el-select v-model="transcodeAudioCodec" size="small" style="width: 120px">
                  <el-option label="AAC" value="aac" />
                  <el-option label="MP3" value="libmp3lame" />
                  <el-option label="Opus" value="libopus" />
                </el-select>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <div class="group-label">分辨率（可选）</div>
                <el-input-number v-model="transcodeWidth" :min="0" :step="2" size="small" style="width: 100px" placeholder="宽" />
                <span style="color:var(--text-secondary);margin:0 4px">x</span>
                <el-input-number v-model="transcodeHeight" :min="0" :step="2" size="small" style="width: 100px" placeholder="高" />
                <span class="unit-text">留空=原始分辨率</span>
              </div>
              <div class="action-group">
                <div class="group-label">帧率（可选）</div>
                <el-input-number v-model="transcodeFps" :min="0" :max="120" :step="1" size="small" style="width: 100px" />
                <span class="unit-text">留空=原始帧率</span>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <div class="group-label">视频比特率</div>
                <el-select v-model="transcodeVideoBitrate" size="small" style="width: 120px" clearable>
                  <el-option label="1 Mbps" value="1M" />
                  <el-option label="2 Mbps" value="2M" />
                  <el-option label="4 Mbps" value="4M" />
                  <el-option label="8 Mbps" value="8M" />
                  <el-option label="12 Mbps" value="12M" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">音频比特率</div>
                <el-select v-model="transcodeAudioBitrate" size="small" style="width: 120px" clearable>
                  <el-option label="96 kbps" value="96k" />
                  <el-option label="128 kbps" value="128k" />
                  <el-option label="192 kbps" value="192k" />
                  <el-option label="256 kbps" value="256k" />
                </el-select>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 8px">
              <div class="action-group">
                <el-checkbox v-model="transcodeSaveToSamePath" size="small">
                  与源文件相同路径
                </el-checkbox>
              </div>
            </div>
          </div>
        </div>

        <div v-if="transcodeVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="doTranscode" :loading="transcodeProcessing">
                  开始转码
                </el-button>
                <el-button size="small" @click="resetTranscode">重置</el-button>
              </div>
            </div>
            <el-progress v-if="transcodeProcessing" :percentage="transcodeProgress" :stroke-width="6" style="margin-top: 12px" />
            <div v-if="transcodeResult" class="result-info">
              <span>输出大小: {{ formatFileSize(transcodeResult.output_size) }}</span>
              <span class="result-sep">|</span>
              <span>输入大小: {{ formatFileSize(transcodeResult.input_size) }}</span>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- ==================== Tab: 音频提取 ==================== -->
    <template v-if="activeTab === 'audioExtract'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            音频提取需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">选择视频文件</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="openAudioExtractFile" :loading="isLoadingInfo">
                  打开文件
                </el-button>
              </div>
            </div>
            <div v-if="audioExtractFilePath" class="video-file-info">
              <span class="file-name">{{ audioExtractFileName }}</span>
              <span class="file-detail" v-if="audioExtractVideoInfo">
                {{ formatDuration(audioExtractVideoInfo.duration) }} | {{ formatFileSize(audioExtractVideoInfo.file_size) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="audioExtractVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">提取设置</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">输出格式</div>
                <el-select v-model="audioExtractFormat" size="small" style="width: 120px" @change="onAudioFormatChange">
                  <el-option label="MP3" value="mp3" />
                  <el-option label="AAC" value="aac" />
                  <el-option label="WAV" value="wav" />
                  <el-option label="FLAC" value="flac" />
                  <el-option label="OGG" value="ogg" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">音频编码器</div>
                <el-select v-model="audioExtractCodec" size="small" style="width: 140px">
                  <el-option label="AAC" value="aac" />
                  <el-option label="MP3 (libmp3lame)" value="libmp3lame" />
                  <el-option label="FLAC" value="flac" />
                  <el-option label="Opus" value="libopus" />
                  <el-option label="PCM (WAV)" value="pcm_s16le" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">比特率</div>
                <el-select v-model="audioExtractBitrate" size="small" style="width: 120px" clearable>
                  <el-option label="96 kbps" value="96k" />
                  <el-option label="128 kbps" value="128k" />
                  <el-option label="192 kbps" value="192k" />
                  <el-option label="256 kbps" value="256k" />
                  <el-option label="320 kbps" value="320k" />
                </el-select>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 8px">
              <div class="action-group">
                <el-checkbox v-model="audioExtractSaveToSamePath" size="small">
                  与源文件相同路径
                </el-checkbox>
              </div>
            </div>
          </div>
        </div>

        <div v-if="audioExtractVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="doAudioExtract" :loading="audioExtractProcessing">
                  提取音频
                </el-button>
                <el-button size="small" @click="resetAudioExtract">重置</el-button>
              </div>
            </div>
            <el-progress v-if="audioExtractProcessing" :percentage="audioExtractProgress" :stroke-width="6" style="margin-top: 12px" />
            <div v-if="audioExtractResult" class="result-info">
              <span>输出大小: {{ formatFileSize(audioExtractResult.output_size) }}</span>
              <span class="result-sep">|</span>
              <span>时长: {{ formatDuration(audioExtractResult.duration) }}</span>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- ==================== Tab: 视频压缩 ==================== -->
    <template v-if="activeTab === 'compress'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            视频压缩需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">选择视频文件</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="openCompressFile" :loading="isLoadingInfo">
                  打开文件
                </el-button>
              </div>
            </div>
            <div v-if="compressFilePath" class="video-file-info">
              <span class="file-name">{{ compressFileName }}</span>
              <span class="file-detail" v-if="compressVideoInfo">
                {{ formatDuration(compressVideoInfo.duration) }} | {{ compressVideoInfo.width }}x{{ compressVideoInfo.height }} |
                {{ formatFileSize(compressVideoInfo.file_size) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="compressVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">压缩设置</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">CRF（质量）: {{ compressCrf }}</div>
                <div class="crf-slider">
                  <el-slider v-model="compressCrf" :min="18" :max="40" :step="1" show-input size="small" style="width: 260px" />
                </div>
                <span class="unit-text">越低质量越好，文件越大</span>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <div class="group-label">预设速度</div>
                <el-select v-model="compressPreset" size="small" style="width: 140px">
                  <el-option label="极快" value="ultrafast" />
                  <el-option label="超快" value="superfast" />
                  <el-option label="很快" value="veryfast" />
                  <el-option label="较快" value="faster" />
                  <el-option label="快" value="fast" />
                  <el-option label="中等" value="medium" />
                  <el-option label="慢" value="slow" />
                  <el-option label="很慢" value="slower" />
                  <el-option label="极慢" value="veryslow" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">视频编码器</div>
                <el-select v-model="compressVideoCodec" size="small" style="width: 120px">
                  <el-option label="H.264" value="libx264" />
                  <el-option label="H.265" value="libx265" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">音频编码器</div>
                <el-select v-model="compressAudioCodec" size="small" style="width: 120px">
                  <el-option label="AAC" value="aac" />
                  <el-option label="MP3" value="libmp3lame" />
                </el-select>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <el-checkbox v-model="compressKeepResolution" size="small">
                  保留原始分辨率
                </el-checkbox>
              </div>
              <div class="action-group" v-if="!compressKeepResolution">
                <div class="group-label">缩放宽度</div>
                <el-input-number v-model="compressWidth" :min="320" :max="7680" :step="2" size="small" style="width: 120px" />
                <span class="unit-text">高度按比例自动</span>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 8px">
              <div class="action-group">
                <el-checkbox v-model="compressSaveToSamePath" size="small">
                  与源文件相同路径
                </el-checkbox>
              </div>
            </div>
          </div>
        </div>

        <div v-if="compressVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="doCompress" :loading="compressProcessing">
                  开始压缩
                </el-button>
                <el-button size="small" @click="resetCompress">重置</el-button>
              </div>
            </div>
            <el-progress v-if="compressProcessing" :percentage="compressProgress" :stroke-width="6" style="margin-top: 12px" />
            <div v-if="compressResult" class="result-info">
              <span>输出大小: {{ formatFileSize(compressResult.output_size) }}</span>
              <span class="result-sep">|</span>
              <span>压缩率: {{ compressResult.compression_ratio }}%</span>
              <span class="result-sep">|</span>
              <span>节省: {{ formatFileSize(compressResult.input_size - compressResult.output_size) }}</span>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- ==================== Tab: 视频合并 ==================== -->
    <template v-if="activeTab === 'merge'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            视频合并需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">添加视频文件</span>
            <div class="card-actions">
              <el-button size="small" @click="openMergeFiles">添加文件</el-button>
              <el-button size="small" @click="clearMergeFiles" :disabled="mergeFiles.length === 0">清空列表</el-button>
            </div>
          </div>
          <div class="card-body">
            <div v-if="mergeFiles.length === 0" class="empty-hint">
              请添加至少 2 个视频文件进行合并
            </div>
            <div v-else class="merge-file-list">
              <div v-for="(f, idx) in mergeFiles" :key="idx" class="merge-file-item">
                <span class="merge-file-idx">{{ idx + 1 }}</span>
                <span class="merge-file-name">{{ f.name }}</span>
                <span class="merge-file-size">{{ formatFileSize(f.size) }}</span>
                <el-button size="small" type="danger" text @click="removeMergeFile(idx)">移除</el-button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="mergeFiles.length >= 2" class="tool-card">
          <div class="card-header">
            <span class="card-title">合并设置</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">输出格式</div>
                <el-select v-model="mergeFormat" size="small" style="width: 120px">
                  <el-option label="MP4" value="mp4" />
                  <el-option label="MKV" value="mkv" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">视频编码器</div>
                <el-select v-model="mergeVideoCodec" size="small" style="width: 120px">
                  <el-option label="H.264" value="libx264" />
                  <el-option label="H.265" value="libx265" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">音频编码器</div>
                <el-select v-model="mergeAudioCodec" size="small" style="width: 120px">
                  <el-option label="AAC" value="aac" />
                  <el-option label="MP3" value="libmp3lame" />
                </el-select>
              </div>
            </div>
            <div class="merge-hint" v-if="allMergeSameFormat">
              所有文件格式相同，将使用无损合并（速度更快，无损失）
            </div>
            <div class="merge-hint-warn" v-else>
              文件格式不同，将进行转码合并（可能需要较长时间）
            </div>
            <div class="action-grid" style="margin-top: 8px">
              <div class="action-group">
                <el-checkbox v-model="mergeSaveToSamePath" size="small">
                  与第一个文件相同路径
                </el-checkbox>
              </div>
            </div>
          </div>
        </div>

        <div v-if="mergeFiles.length >= 2" class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="doMerge" :loading="mergeProcessing">
                  开始合并
                </el-button>
                <el-button size="small" @click="resetMerge">重置</el-button>
              </div>
            </div>
            <el-progress v-if="mergeProcessing" :percentage="mergeProgress" :stroke-width="6" style="margin-top: 12px" />
            <div v-if="mergeResult" class="result-info">
              <span>文件数: {{ mergeResult.file_count }}</span>
              <span class="result-sep">|</span>
              <span>输出大小: {{ formatFileSize(mergeResult.output_size) }}</span>
              <span class="result-sep">|</span>
              <span>总时长: {{ formatDuration(mergeResult.duration) }}</span>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- ==================== Tab: 截图提取 (F24) ==================== -->
    <template v-if="activeTab === 'frameExtract'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            视频截图需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">选择视频文件</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="openFrameExtractFile" :loading="isLoadingInfo">
                  打开文件
                </el-button>
              </div>
            </div>
            <div v-if="frameExtractFilePath" class="video-file-info">
              <span class="file-name">{{ frameExtractFileName }}</span>
              <span class="file-detail" v-if="frameExtractVideoInfo">
                {{ formatDuration(frameExtractVideoInfo.duration) }} | {{ frameExtractVideoInfo.width }}x{{ frameExtractVideoInfo.height }} |
                {{ formatFileSize(frameExtractVideoInfo.file_size) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="frameExtractVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">时间点与预览</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">时间点</div>
                <el-input-number
                  v-model="frameExtractTime"
                  :min="0"
                  :max="frameExtractVideoInfo.duration"
                  :step="0.1"
                  :precision="1"
                  size="small"
                  style="width: 140px"
                  @change="onFrameExtractTimeChange"
                />
                <span class="unit-text">秒（视频总长 {{ formatDuration(frameExtractVideoInfo.duration) }}）</span>
              </div>
            </div>
            <!-- 预览区域 -->
            <div class="frame-preview-container" v-if="frameExtractVideoInfo">
              <div class="frame-preview-label">预览</div>
              <div class="frame-preview-box" :class="{ 'is-loading': framePreviewLoading }">
                <img v-if="framePreviewSrc" :src="framePreviewSrc" class="frame-preview-img" />
                <span v-else-if="framePreviewLoading" class="frame-preview-hint">加载中...</span>
                <span v-else class="frame-preview-hint">拖动时间点以预览</span>
              </div>
            </div>
          </div>
        </div>

        <div v-if="frameExtractVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">导出设置</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">输出格式</div>
                <el-select v-model="frameExtractFormat" size="small" style="width: 100px">
                  <el-option label="JPG" value="jpg" />
                  <el-option label="PNG" value="png" />
                </el-select>
              </div>
              <div class="action-group" v-if="frameExtractFormat === 'jpg'">
                <div class="group-label">质量</div>
                <el-slider v-model="frameExtractQuality" :min="2" :max="31" :step="1" show-input size="small" style="width: 200px" />
                <span class="unit-text">越低质量越好</span>
              </div>
              <div class="action-group">
                <el-checkbox v-model="frameExtractSaveToSamePath" size="small">保存到源文件目录</el-checkbox>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <el-button type="primary" size="small" @click="doFrameExtract" :loading="frameExtractProcessing">
                  提取截图
                </el-button>
                <el-button size="small" @click="resetFrameExtract">重置</el-button>
              </div>
            </div>
            <el-progress v-if="frameExtractProcessing" :percentage="frameExtractProgress" :stroke-width="6" style="margin-top: 12px" />
            <!-- 提取结果预览 -->
            <div v-if="frameExtractResult" class="frame-result-container">
              <div class="frame-preview-label">
                截图结果 ({{ frameExtractResult.width }}x{{ frameExtractResult.height }}, {{ formatFileSize(frameExtractResult.output_size) }})
              </div>
              <div class="frame-preview-box">
                <img :src="frameExtractResultSrc" class="frame-preview-img" />
              </div>
              <div class="action-grid" style="margin-top: 12px">
                <div class="action-group">
                  <el-button type="success" size="small" @click="saveFrameExtractResult">
                    保存到文件
                  </el-button>
                  <el-button size="small" @click="copyFrameExtractResult">
                    复制到剪贴板
                  </el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- ==================== Tab: 画面裁剪 (F25) ==================== -->
    <template v-if="activeTab === 'cropRegion'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            视频画面裁剪需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">选择视频文件</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="openCropRegionFile" :loading="isLoadingInfo">
                  打开文件
                </el-button>
              </div>
            </div>
            <div v-if="cropRegionFilePath" class="video-file-info">
              <span class="file-name">{{ cropRegionFileName }}</span>
              <span class="file-detail" v-if="cropRegionVideoInfo">
                {{ formatDuration(cropRegionVideoInfo.duration) }} | {{ cropRegionVideoInfo.width }}x{{ cropRegionVideoInfo.height }} |
                {{ formatFileSize(cropRegionVideoInfo.file_size) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="cropRegionVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">裁剪预览</span>
            <span class="card-actions">
              <span class="crop-dim-hint">
                裁剪区域: {{ cropRegionW }}x{{ cropRegionH }} (原视频: {{ cropRegionVideoInfo.width }}x{{ cropRegionVideoInfo.height }})
              </span>
            </span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">预览时间点</div>
                <el-input-number
                  v-model="cropRegionPreviewTime"
                  :min="0"
                  :max="cropRegionVideoInfo.duration"
                  :step="0.1"
                  :precision="1"
                  size="small"
                  style="width: 140px"
                  @change="onCropPreviewTimeChange"
                />
                <span class="unit-text">秒（视频总长 {{ formatDuration(cropRegionVideoInfo.duration) }}）</span>
              </div>
            </div>
            <div class="crop-preview-wrapper" ref="cropPreviewEl"
              :class="{ 'is-loading': cropPreviewLoading }"
              @mousedown="onCropMouseDown"
              @mousemove="onCropMouseMove"
              @mouseup="onCropMouseUp"
              @mouseleave="onCropMouseUp"
            >
              <img
                v-if="cropPreviewSrc"
                :src="cropPreviewSrc"
                class="crop-preview-img"
                ref="cropImgEl"
                @load="onCropImgLoad"
              />
              <div v-else class="crop-preview-placeholder">
                <span v-if="cropPreviewLoading">加载预览中...</span>
                <span v-else>打开视频文件以预览</span>
              </div>
              <!-- ponytail: 遮罩 loading 仅在已有图像时叠加，避免与 placeholder 文案重复 -->
              <div v-if="cropPreviewLoading && cropPreviewSrc" class="crop-preview-mask">
                <span class="crop-preview-mask-text">加载预览中...</span>
              </div>
              <!-- 裁剪框叠加层 -->
              <div
                v-if="cropPreviewSrc && cropOverlayStyle"
                class="crop-overlay-box"
                :style="cropOverlayStyle"
              >
                <!-- 裁剪框边框 -->
                <div class="crop-border" />
                <!-- 拖拽手柄 -->
                <div class="crop-handle crop-handle-tl" @mousedown.stop="onCropHandleMouseDown($event, 'tl')" />
                <div class="crop-handle crop-handle-tr" @mousedown.stop="onCropHandleMouseDown($event, 'tr')" />
                <div class="crop-handle crop-handle-bl" @mousedown.stop="onCropHandleMouseDown($event, 'bl')" />
                <div class="crop-handle crop-handle-br" @mousedown.stop="onCropHandleMouseDown($event, 'br')" />
              </div>
            </div>
            <!-- 快捷操作 -->
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <div class="group-label">预设比例</div>
                <el-select v-model="cropRegionPreset" size="small" style="width: 100px" @change="onCropPresetChange" clearable placeholder="手动">
                  <el-option label="16:9" value="16:9" />
                  <el-option label="4:3" value="4:3" />
                  <el-option label="1:1" value="1:1" />
                  <el-option label="9:16" value="9:16" />
                  <el-option label="3:2" value="3:2" />
                  <el-option label="21:9" value="21:9" />
                </el-select>
              </div>
              <div class="action-group">
                <el-button size="small" @click="resetCropToFull">重置为全画面</el-button>
              </div>
            </div>
            <div class="crop-region-grid" v-if="cropRegionVideoInfo">
              <div class="crop-region-input">
                <div class="group-label">X 偏移</div>
                <el-input-number v-model="cropRegionX" :min="0" :max="cropRegionVideoInfo.width - 2" :step="2" size="small" style="width: 100px" />
              </div>
              <div class="crop-region-input">
                <div class="group-label">Y 偏移</div>
                <el-input-number v-model="cropRegionY" :min="0" :max="cropRegionVideoInfo.height - 2" :step="2" size="small" style="width: 100px" />
              </div>
              <div class="crop-region-input">
                <div class="group-label">宽度</div>
                <el-input-number v-model="cropRegionW" :min="2" :max="cropRegionVideoInfo.width" :step="2" size="small" style="width: 100px" />
              </div>
              <div class="crop-region-input">
                <div class="group-label">高度</div>
                <el-input-number v-model="cropRegionH" :min="2" :max="cropRegionVideoInfo.height" :step="2" size="small" style="width: 100px" />
              </div>
            </div>
          </div>
        </div>

        <div v-if="cropRegionVideoInfo" class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-checkbox v-model="cropRegionSaveToSamePath" size="small">保存到源文件目录</el-checkbox>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <el-button type="primary" size="small" @click="doCropRegion" :loading="cropRegionProcessing">
                  裁剪并导出
                </el-button>
                <el-button size="small" @click="resetCropRegion">重置</el-button>
              </div>
            </div>
            <el-progress v-if="cropRegionProcessing" :percentage="cropRegionProgress" :stroke-width="6" style="margin-top: 12px" />
            <div v-if="cropRegionResult" class="result-info">
              <span>输出大小: {{ formatFileSize(cropRegionResult.output_size) }}</span>
              <span class="result-sep">|</span>
              <span>尺寸: {{ cropRegionResult.width }}x{{ cropRegionResult.height }}</span>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, onActivated, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ 类型定义 ============
interface VideoInfo {
  duration: number
  width: number
  height: number
  codec: string
  fps: number
  bitrate: number
  file_size: number
  format: string
}

interface ThumbnailResult {
  images: string[]
  timestamps: number[]
}

interface CropResult {
  output_path: string
  output_size: number
  duration: number
  actual_start: number | null
  actual_end: number | null
}

interface TranscodeResult {
  output_path: string
  output_size: number
  input_size: number
  duration: number
}

interface AudioExtractResult {
  output_path: string
  output_size: number
  duration: number
}

interface CompressResult {
  output_path: string
  output_size: number
  input_size: number
  compression_ratio: number
  duration: number
}

interface MergeResult {
  output_path: string
  output_size: number
  duration: number
  file_count: number
}

interface MergeFileItem {
  path: string
  name: string
  size: number
}

interface FrameExtractResult {
  output_path: string
  output_size: number
  width: number
  height: number
}

interface CropPresetResult {
  x: number
  y: number
  width: number
  height: number
}

interface CropRegionResult {
  output_path: string
  output_size: number
  width: number
  height: number
}

// ============ Tab 状态 ============
const activeTab = ref('crop')

// ============ 裁剪状态 ============
const filePath = ref('')
const fileName = ref('')
const videoInfo = ref<VideoInfo | null>(null)
const thumbnails = ref<ThumbnailResult>({ images: [], timestamps: [] })
const startTime = ref(0)
const endTime = ref(0)
const isProcessing = ref(false)
const isLoadingInfo = ref(false)
const saveToSamePath = ref(true)
const cropProgress = ref(0)
const useFfmpeg = ref(false)
const ffmpegChecked = ref(false)
const error = ref('')
const actualRange = ref<{ start: number; end: number } | null>(null)

// ============ 转码状态 ============
const transcodeFilePath = ref('')
const transcodeFileName = ref('')
const transcodeVideoInfo = ref<VideoInfo | null>(null)
const transcodeFormat = ref('mp4')
const transcodeVideoCodec = ref('libx264')
const transcodeAudioCodec = ref('aac')
const transcodeWidth = ref(0)
const transcodeHeight = ref(0)
const transcodeFps = ref(0)
const transcodeVideoBitrate = ref('')
const transcodeAudioBitrate = ref('')
const transcodeSaveToSamePath = ref(true)
const transcodeProcessing = ref(false)
const transcodeProgress = ref(0)
const transcodeResult = ref<TranscodeResult | null>(null)

// ============ 音频提取状态 ============
const audioExtractFilePath = ref('')
const audioExtractFileName = ref('')
const audioExtractVideoInfo = ref<VideoInfo | null>(null)
const audioExtractFormat = ref('mp3')
const audioExtractCodec = ref('libmp3lame')
const audioExtractBitrate = ref('')
const audioExtractSaveToSamePath = ref(true)
const audioExtractProcessing = ref(false)
const audioExtractProgress = ref(0)
const audioExtractResult = ref<AudioExtractResult | null>(null)

// ============ 压缩状态 ============
const compressFilePath = ref('')
const compressFileName = ref('')
const compressVideoInfo = ref<VideoInfo | null>(null)
const compressCrf = ref(23)
const compressPreset = ref('medium')
const compressVideoCodec = ref('libx264')
const compressAudioCodec = ref('aac')
const compressKeepResolution = ref(true)
const compressWidth = ref(1280)
const compressSaveToSamePath = ref(true)
const compressProcessing = ref(false)
const compressProgress = ref(0)
const compressResult = ref<CompressResult | null>(null)

// ============ 合并状态 ============
const mergeFiles = ref<MergeFileItem[]>([])
const mergeFormat = ref('mp4')
const mergeVideoCodec = ref('libx264')
const mergeAudioCodec = ref('aac')
const mergeSaveToSamePath = ref(true)
const mergeProcessing = ref(false)
const mergeProgress = ref(0)
const mergeResult = ref<MergeResult | null>(null)

// ============ 截图提取状态 (F24) ============
const frameExtractFilePath = ref('')
const frameExtractFileName = ref('')
const frameExtractVideoInfo = ref<VideoInfo | null>(null)
const frameExtractTime = ref(0)
const frameExtractFormat = ref('jpg')
const frameExtractQuality = ref(2)
const frameExtractSaveToSamePath = ref(true)
const frameExtractProcessing = ref(false)
const frameExtractProgress = ref(0)
const frameExtractResult = ref<FrameExtractResult | null>(null)
const frameExtractResultSrc = ref('')
const framePreviewSrc = ref('')
const framePreviewLoading = ref(false)
let framePreviewTimer: ReturnType<typeof setTimeout> | null = null

// ============ 画面裁剪状态 (F25) ============
const cropRegionFilePath = ref('')
const cropRegionFileName = ref('')
const cropRegionVideoInfo = ref<VideoInfo | null>(null)
const cropRegionPreset = ref('')
const cropRegionX = ref(0)
const cropRegionY = ref(0)
const cropRegionW = ref(0)
const cropRegionH = ref(0)
const cropRegionSaveToSamePath = ref(true)
const cropRegionPreviewTime = ref(0)
const cropRegionProcessing = ref(false)
const cropRegionProgress = ref(0)
const cropRegionResult = ref<CropRegionResult | null>(null)
const cropPreviewSrc = ref('')
const cropPreviewLoading = ref(false)
const cropPreviewEl = ref<HTMLElement | null>(null)
const cropImgEl = ref<HTMLImageElement | null>(null)
// 拖拽状态
const cropDragging = ref(false)
const cropDragType = ref<'move' | 'tl' | 'tr' | 'bl' | 'br' | null>(null)
const cropDragStartX = ref(0)
const cropDragStartY = ref(0)
const cropDragStartRegion = ref({ x: 0, y: 0, w: 0, h: 0 })
// 图片显示比例（显示宽度 / 实际视频宽度）
const cropDisplayRatio = ref(1)

// 裁剪框叠加层样式（基于显示坐标）
const cropOverlayStyle = computed(() => {
  if (!cropRegionVideoInfo.value) return null
  const r = cropDisplayRatio.value
  return {
    left: Math.round(cropRegionX.value * r) + 'px',
    top: Math.round(cropRegionY.value * r) + 'px',
    width: Math.round(cropRegionW.value * r) + 'px',
    height: Math.round(cropRegionH.value * r) + 'px',
  }
})

// ============ 裁剪计算属性 ============
const segmentDuration = computed(() => endTime.value - startTime.value)
const isRangeValid = computed(() => startTime.value < endTime.value && segmentDuration.value >= 0.1)

// ============ 合并计算属性 ============
const allMergeSameFormat = computed(() => {
  if (mergeFiles.value.length < 2) return true
  const firstExt = mergeFiles.value[0].name.split('.').pop()?.toLowerCase()
  return firstExt === mergeFormat.value && mergeFiles.value.every(f => f.name.split('.').pop()?.toLowerCase() === firstExt)
})

// ============ Canvas 时间轴 ============
const canvasRef = ref<HTMLCanvasElement | null>(null)
const thumbnailCanvasRef = ref<HTMLCanvasElement | null>(null)
const timelineContainer = ref<HTMLDivElement | null>(null)
const thumbnailImages = ref<HTMLImageElement[]>([])

// ponytail: 绘制缩略图到底层 canvas（只在加载视频时调用一次）
function drawThumbnails() {
  const thumbCanvas = thumbnailCanvasRef.value
  if (!thumbCanvas || !videoInfo.value) return

  const dpr = window.devicePixelRatio || 1
  const rect = thumbCanvas.getBoundingClientRect()
  thumbCanvas.width = rect.width * dpr
  thumbCanvas.height = rect.height * dpr

  const ctx = thumbCanvas.getContext('2d')
  if (!ctx) return

  ctx.scale(dpr, dpr)
  const width = rect.width
  const height = rect.height

  const style = getComputedStyle(document.documentElement)
  const bgColor = style.getPropertyValue('--bg-input').trim() || '#0d1520'
  const secondaryColor = style.getPropertyValue('--text-secondary').trim() || '#94a3b8'
  ctx.fillStyle = bgColor
  ctx.fillRect(0, 0, width, height)

  if (thumbnails.value.images.length > 0) {
    const n = thumbnails.value.images.length
    const thumbWidth = width / n

    if (thumbnailImages.value.length === 0) {
      const imgPromises: Promise<void>[] = []
      for (let i = 0; i < n; i++) {
        const img = new Image()
        const promise = new Promise<void>((resolve) => {
          img.onload = () => { resolve() }
          img.onerror = () => resolve()
        })
        img.src = 'data:image/jpeg;base64,' + thumbnails.value.images[i]
        thumbnailImages.value.push(img)
        imgPromises.push(promise)
      }
      Promise.all(imgPromises).then(() => {
        for (let i = 0; i < n; i++) {
          const img = thumbnailImages.value[i]
          if (img.complete && img.naturalWidth > 0) {
            const x = i * thumbWidth
            const aspectRatio = img.width / img.height
            const drawWidth = height * aspectRatio
            const drawX = x + (thumbWidth - drawWidth) / 2
            ctx.drawImage(img, drawX, 0, drawWidth, height)
          }
        }
      })
    } else {
      for (let i = 0; i < n; i++) {
        const img = thumbnailImages.value[i]
        if (img.complete && img.naturalWidth > 0) {
          const x = i * thumbWidth
          const aspectRatio = img.width / img.height
          const drawWidth = height * aspectRatio
          const drawX = x + (thumbWidth - drawWidth) / 2
          ctx.drawImage(img, drawX, 0, drawWidth, height)
        }
      }
    }
  } else {
    // 无缩略图时绘制占位
    ctx.strokeStyle = secondaryColor + '66'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()

    const tickCount = 10
    for (let i = 0; i <= tickCount; i++) {
      const x = (i / tickCount) * width
      const tickHeight = i % 2 === 0 ? 12 : 6
      ctx.strokeStyle = secondaryColor + '88'
      ctx.beginPath()
      ctx.moveTo(x, height / 2 - tickHeight)
      ctx.lineTo(x, height / 2 + tickHeight)
      ctx.stroke()
    }

    ctx.fillStyle = secondaryColor
    ctx.font = '14px sans-serif'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText('未检测到 ffmpeg，无法显示缩略图', width / 2, height / 2 - 20)
  }
}

// ponytail: 绘制选中区域到顶层 canvas（拖拽时调用）
function drawSelection() {
  const canvas = canvasRef.value
  if (!canvas || !videoInfo.value) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  const width = rect.width
  const height = rect.height
  const dur = videoInfo.value.duration

  if (canvas.width !== width * dpr || canvas.height !== height * dpr) {
    canvas.width = width * dpr
    canvas.height = height * dpr
    ctx.scale(dpr, dpr)
  }

  const style = getComputedStyle(document.documentElement)
  const primaryColor = style.getPropertyValue('--accent-cyan').trim() || '#00d4ff'

  const startX = (startTime.value / dur) * width
  const endX = (endTime.value / dur) * width

  ctx.clearRect(0, 0, width, height)
  drawSelectionOverlay(ctx, startX, endX, width, height, primaryColor)
}

function drawTimeline() {
  drawThumbnails()
  drawSelection()
}

function drawSelectionOverlay(
  ctx: CanvasRenderingContext2D,
  startX: number, endX: number, _width: number, height: number, primaryColor: string
) {
  ctx.fillStyle = primaryColor + '1A'
  ctx.fillRect(startX, 0, endX - startX, height)
  ctx.strokeStyle = primaryColor
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(startX, 0)
  ctx.lineTo(startX, height)
  ctx.stroke()
  ctx.beginPath()
  ctx.moveTo(endX, 0)
  ctx.lineTo(endX, height)
  ctx.stroke()
}

function timeToPercent(time: number): number {
  if (!videoInfo.value || videoInfo.value.duration <= 0) return 0
  return (time / videoInfo.value.duration) * 100
}

function percentToTime(percent: number): number {
  if (!videoInfo.value) return 0
  return Math.round((percent / 100) * videoInfo.value.duration * 10) / 10
}

// ============ 滑块拖拽 ============
let draggingSlider: 'start' | 'end' | null = null

function onSliderMouseDown(_e: MouseEvent, slider: 'start' | 'end') {
  draggingSlider = slider
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!draggingSlider || !timelineContainer.value || !videoInfo.value) return
  const rect = timelineContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))
  if (draggingSlider === 'start') {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawTimeline()
}

function onMouseUp() {
  draggingSlider = null
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown(e: MouseEvent) {
  if (!timelineContainer.value || !videoInfo.value) return
  const rect = timelineContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))
  const startDist = Math.abs(time - startTime.value)
  const endDist = Math.abs(time - endTime.value)
  if (startDist <= endDist) {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawTimeline()
}

// ============ 通用文件操作 ============
async function openFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: useFfmpeg.value ? ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] : ['mp4', 'm4v'] }],
      multiple: false,
    })
    if (!selected) return
    resetForm() // 选定文件后重置状态，清空旧缩略图等内容
    filePath.value = selected as string
    fileName.value = (selected as string).split(/[/\\]/).pop() || ''
    isLoadingInfo.value = true
    actualRange.value = null
    const info: VideoInfo = await invoke('get_video_info', { path: filePath.value, useFfmpeg: useFfmpeg.value })
    videoInfo.value = info
    startTime.value = 0
    endTime.value = info.duration
    const result: ThumbnailResult = await invoke('extract_thumbnails', { path: filePath.value, count: 12 })
    thumbnails.value = result
    await nextTick()
    drawTimeline()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetForm()
  } finally {
    isLoadingInfo.value = false
  }
}

async function cropVideo() {
  if (!isRangeValid.value) { ElMessage.warning('请设置有效的裁剪区间'); return }
  try {
    error.value = ''
    isProcessing.value = true
    cropProgress.value = 0
    actualRange.value = null
    const unlisten = await listen<{ progress: number }>('video-crop-progress', (event) => {
      cropProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!saveToSamePath.value) {
      const defaultName = fileName.value.replace(/\.[^.]+$/, '') + '_cropped.mp4'
      outputPath = await save({ defaultPath: defaultName, filters: [{ name: 'MP4 视频', extensions: ['mp4'] }] })
      if (!outputPath) { unlisten(); isProcessing.value = false; return }
    }
    const result: CropResult = await invoke('video_crop', {
      path: filePath.value,
      options: { start_time: startTime.value, end_time: endTime.value, use_ffmpeg: useFfmpeg.value, output_path: outputPath },
    })
    unlisten()
    cropProgress.value = 100
    if (result.actual_start != null && result.actual_end != null) {
      actualRange.value = { start: result.actual_start, end: result.actual_end }
    }
    ElMessage.success(`裁剪完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '裁剪',
      inputPreview: `${fileName.value} [${formatDuration(startTime.value)} - ${formatDuration(endTime.value)}]`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: filePath.value,
      outputFull: result.output_path,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '裁剪失败'
  } finally { isProcessing.value = false }
}

function resetForm() {
  filePath.value = ''; fileName.value = ''; videoInfo.value = null
  thumbnails.value = { images: [], timestamps: [] }
  thumbnailImages.value = []
  startTime.value = 0; endTime.value = 0; error.value = ''; actualRange.value = null
}

// ============ 转码操作 ============
async function openTranscodeFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] }],
      multiple: false,
    })
    if (!selected) return
    transcodeFilePath.value = selected as string
    transcodeFileName.value = (selected as string).split(/[/\\]/).pop() || ''
    isLoadingInfo.value = true
    transcodeResult.value = null
    const info: VideoInfo = await invoke('get_video_info', { path: transcodeFilePath.value, useFfmpeg: true })
    transcodeVideoInfo.value = info
    transcodeWidth.value = info.width
    transcodeHeight.value = info.height
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
  } finally { isLoadingInfo.value = false }
}

async function doTranscode() {
  try {
    error.value = ''
    transcodeProcessing.value = true
    transcodeProgress.value = 0
    transcodeResult.value = null
    const unlisten = await listen<{ progress: number }>('video-transcode-progress', (event) => {
      transcodeProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!transcodeSaveToSamePath.value) {
      outputPath = await save({
        defaultPath: transcodeFileName.value.replace(/\.[^.]+$/, '') + '_transcoded.' + transcodeFormat.value,
        filters: [{ name: '视频', extensions: [transcodeFormat.value] }],
      })
      if (!outputPath) { unlisten(); transcodeProcessing.value = false; return }
    }
    const result: TranscodeResult = await invoke('video_transcode', {
      path: transcodeFilePath.value,
      options: {
        output_format: transcodeFormat.value,
        video_codec: transcodeVideoCodec.value,
        audio_codec: transcodeAudioCodec.value,
        width: transcodeWidth.value > 0 ? transcodeWidth.value : null,
        height: transcodeHeight.value > 0 ? transcodeHeight.value : null,
        fps: transcodeFps.value > 0 ? transcodeFps.value : null,
        video_bitrate: transcodeVideoBitrate.value || null,
        audio_bitrate: transcodeAudioBitrate.value || null,
        output_path: outputPath,
      },
    })
    unlisten()
    transcodeProgress.value = 100
    transcodeResult.value = result
    ElMessage.success(`转码完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '转码',
      inputPreview: `${transcodeFileName.value} → ${transcodeFormat.value.toUpperCase()}`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: transcodeFilePath.value,
      outputFull: result.output_path,
      options: {
        output_format: transcodeFormat.value,
        video_codec: transcodeVideoCodec.value,
        audio_codec: transcodeAudioCodec.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '转码失败'
  } finally { transcodeProcessing.value = false }
}

function resetTranscode() {
  transcodeFilePath.value = ''; transcodeFileName.value = ''; transcodeVideoInfo.value = null
  transcodeResult.value = null; error.value = ''
}

// ============ 音频提取操作 ============
async function openAudioExtractFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] }],
      multiple: false,
    })
    if (!selected) return
    audioExtractFilePath.value = selected as string
    audioExtractFileName.value = (selected as string).split(/[/\\]/).pop() || ''
    isLoadingInfo.value = true
    audioExtractResult.value = null
    const info: VideoInfo = await invoke('get_video_info', { path: audioExtractFilePath.value, useFfmpeg: true })
    audioExtractVideoInfo.value = info
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
  } finally { isLoadingInfo.value = false }
}

function onAudioFormatChange(fmt: string) {
  const map: Record<string, string> = { mp3: 'libmp3lame', aac: 'aac', wav: 'pcm_s16le', flac: 'flac', ogg: 'libopus' }
  if (map[fmt]) audioExtractCodec.value = map[fmt]
}

async function doAudioExtract() {
  try {
    error.value = ''
    audioExtractProcessing.value = true
    audioExtractProgress.value = 0
    audioExtractResult.value = null
    const unlisten = await listen<{ progress: number }>('audio-extract-progress', (event) => {
      audioExtractProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!audioExtractSaveToSamePath.value) {
      outputPath = await save({
        defaultPath: audioExtractFileName.value.replace(/\.[^.]+$/, '') + '_audio.' + audioExtractFormat.value,
        filters: [{ name: '音频', extensions: [audioExtractFormat.value] }],
      })
      if (!outputPath) { unlisten(); audioExtractProcessing.value = false; return }
    }
    const result: AudioExtractResult = await invoke('audio_extract', {
      path: audioExtractFilePath.value,
      options: {
        output_format: audioExtractFormat.value,
        audio_codec: audioExtractCodec.value,
        bitrate: audioExtractBitrate.value || null,
        quality: null,
        output_path: outputPath,
      },
    })
    unlisten()
    audioExtractProgress.value = 100
    audioExtractResult.value = result
    ElMessage.success(`音频提取完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '音频提取',
      inputPreview: `${audioExtractFileName.value} → ${audioExtractFormat.value.toUpperCase()}`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: audioExtractFilePath.value,
      outputFull: result.output_path,
      options: {
        output_format: audioExtractFormat.value,
        audio_codec: audioExtractCodec.value,
        bitrate: audioExtractBitrate.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '音频提取失败'
  } finally { audioExtractProcessing.value = false }
}

function resetAudioExtract() {
  audioExtractFilePath.value = ''; audioExtractFileName.value = ''; audioExtractVideoInfo.value = null
  audioExtractResult.value = null; error.value = ''
}

// ============ 压缩操作 ============
async function openCompressFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] }],
      multiple: false,
    })
    if (!selected) return
    compressFilePath.value = selected as string
    compressFileName.value = (selected as string).split(/[/\\]/).pop() || ''
    isLoadingInfo.value = true
    compressResult.value = null
    const info: VideoInfo = await invoke('get_video_info', { path: compressFilePath.value, useFfmpeg: true })
    compressVideoInfo.value = info
    compressWidth.value = info.width
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
  } finally { isLoadingInfo.value = false }
}

async function doCompress() {
  try {
    error.value = ''
    compressProcessing.value = true
    compressProgress.value = 0
    compressResult.value = null
    const unlisten = await listen<{ progress: number }>('video-compress-progress', (event) => {
      compressProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!compressSaveToSamePath.value) {
      outputPath = await save({
        defaultPath: compressFileName.value.replace(/\.[^.]+$/, '') + '_compressed.mp4',
        filters: [{ name: 'MP4 视频', extensions: ['mp4'] }],
      })
      if (!outputPath) { unlisten(); compressProcessing.value = false; return }
    }
    const result: CompressResult = await invoke('video_compress', {
      path: compressFilePath.value,
      options: {
        crf: compressCrf.value,
        preset: compressPreset.value,
        video_codec: compressVideoCodec.value,
        audio_codec: compressAudioCodec.value,
        width: compressWidth.value > 0 ? compressWidth.value : null,
        keep_resolution: compressKeepResolution.value,
        output_path: outputPath,
      },
    })
    unlisten()
    compressProgress.value = 100
    compressResult.value = result
    ElMessage.success(`压缩完成，已保存到: ${result.output_path}（压缩率 ${result.compression_ratio}%）`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '压缩',
      inputPreview: `${compressFileName.value} (${formatFileSize(result.input_size)})`,
      outputPreview: `${formatFileSize(result.output_size)} (压缩率 ${result.compression_ratio}%)`,
      inputFull: compressFilePath.value,
      outputFull: result.output_path,
      options: {
        crf: compressCrf.value,
        preset: compressPreset.value,
        video_codec: compressVideoCodec.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '压缩失败'
  } finally { compressProcessing.value = false }
}

function resetCompress() {
  compressFilePath.value = ''; compressFileName.value = ''; compressVideoInfo.value = null
  compressResult.value = null; error.value = ''
}

// ============ 合并操作 ============
async function openMergeFiles() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] }],
      multiple: true,
    })
    if (!selected) return
    const paths = Array.isArray(selected) ? selected : [selected]
    for (const p of paths) {
      const pathStr = p as string
      if (!mergeFiles.value.find(f => f.path === pathStr)) {
        const fsMeta = await invoke('get_file_info', { path: pathStr }).catch(() => ({ size: 0 }))
        mergeFiles.value.push({ path: pathStr, name: pathStr.split(/[/\\]/).pop() || '', size: (fsMeta as any).size || 0 })
      }
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '添加文件失败'
  }
}

function removeMergeFile(idx: number) {
  mergeFiles.value.splice(idx, 1)
}

function clearMergeFiles() {
  mergeFiles.value = []
}

async function doMerge() {
  if (mergeFiles.value.length < 2) { ElMessage.warning('至少需要 2 个视频文件'); return }
  try {
    error.value = ''
    mergeProcessing.value = true
    mergeProgress.value = 0
    mergeResult.value = null
    const unlisten = await listen<{ progress: number }>('video-merge-progress', (event) => {
      mergeProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!mergeSaveToSamePath.value) {
      outputPath = await save({
        defaultPath: 'merged.' + mergeFormat.value,
        filters: [{ name: '视频', extensions: [mergeFormat.value] }],
      })
      if (!outputPath) { unlisten(); mergeProcessing.value = false; return }
    }
    const result: MergeResult = await invoke('video_merge', {
      options: {
        paths: mergeFiles.value.map(f => f.path),
        output_format: mergeFormat.value,
        video_codec: mergeVideoCodec.value,
        audio_codec: mergeAudioCodec.value,
        output_path: outputPath,
      },
    })
    unlisten()
    mergeProgress.value = 100
    mergeResult.value = result
    ElMessage.success(`合并完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '合并',
      inputPreview: `${mergeFiles.value.length} 个文件 → ${mergeFormat.value.toUpperCase()}`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: mergeFiles.value.map(f => f.path).join('\n'),
      outputFull: result.output_path,
      options: {
        file_count: mergeFiles.value.length,
        output_format: mergeFormat.value,
        video_codec: mergeVideoCodec.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '合并失败'
  } finally { mergeProcessing.value = false }
}

function resetMerge() {
  mergeFiles.value = []; mergeResult.value = null; error.value = ''
}

// ============ 截图提取方法 (F24) ============
async function openFrameExtractFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] }],
      multiple: false,
    })
    if (!selected) return
    frameExtractFilePath.value = selected as string
    frameExtractFileName.value = (selected as string).split(/[/\\]/).pop() || ''
    isLoadingInfo.value = true
    frameExtractResult.value = null
    frameExtractResultSrc.value = ''
    framePreviewSrc.value = ''
    const info: VideoInfo = await invoke('get_video_info', { path: frameExtractFilePath.value, useFfmpeg: true })
    frameExtractVideoInfo.value = info
    frameExtractTime.value = 0
    // 加载初始帧预览
    loadFramePreview()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
  } finally { isLoadingInfo.value = false }
}

function onFrameExtractTimeChange() {
  // 300ms 防抖
  if (framePreviewTimer) clearTimeout(framePreviewTimer)
  framePreviewTimer = setTimeout(() => loadFramePreview(), 300)
}

async function loadFramePreview() {
  if (!frameExtractFilePath.value) return
  framePreviewLoading.value = true
  try {
    const base64: string = await invoke('video_preview_frame', {
      path: frameExtractFilePath.value,
      timePoint: frameExtractTime.value,
      maxWidth: 480,
    })
    framePreviewSrc.value = `data:image/jpeg;base64,${base64}`
  } catch (e: any) {
    // 预览失败显示错误，方便调试
    const msg = typeof e === 'string' ? e : e.message || '预览失败'
    console.error('loadFramePreview error:', msg)
    framePreviewSrc.value = ''
  } finally { framePreviewLoading.value = false }
}

async function doFrameExtract() {
  try {
    error.value = ''
    frameExtractProcessing.value = true
    frameExtractProgress.value = 0
    frameExtractResult.value = null
    frameExtractResultSrc.value = ''
    const unlisten = await listen<{ progress: number }>('video-extract-frame-progress', (event) => {
      frameExtractProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!frameExtractSaveToSamePath.value) {
      const ext = frameExtractFormat.value
      outputPath = await save({
        defaultPath: frameExtractFileName.value.replace(/\.[^.]+$/, '') + `_frame_${frameExtractTime.value.toFixed(1)}s.` + ext,
        filters: [{ name: ext.toUpperCase() + ' 图片', extensions: [ext] }],
      })
      if (!outputPath) { unlisten(); frameExtractProcessing.value = false; return }
    }
    const result: FrameExtractResult = await invoke('video_extract_frame', {
      path: frameExtractFilePath.value,
      options: {
        time_point: frameExtractTime.value,
        output_format: frameExtractFormat.value,
        quality: frameExtractFormat.value === 'jpg' ? frameExtractQuality.value : null,
        output_path: outputPath,
      },
    })
    unlisten()
    frameExtractProgress.value = 100
    frameExtractResult.value = result
    frameExtractResultSrc.value = await invoke('read_file_base64', { path: result.output_path })
    frameExtractResultSrc.value = `data:image/${frameExtractFormat.value};base64,${frameExtractResultSrc.value.replace(/^data:.*?;base64,/, '')}`
    ElMessage.success(`截图提取完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '截图提取',
      inputPreview: `${frameExtractFileName.value} @ ${frameExtractTime.value.toFixed(1)}s`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: frameExtractFilePath.value,
      outputFull: result.output_path,
      options: {
        time_point: frameExtractTime.value,
        output_format: frameExtractFormat.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '截图失败'
  } finally { frameExtractProcessing.value = false }
}

async function saveFrameExtractResult() {
  if (!frameExtractResult.value) return
  try {
    const ext = frameExtractFormat.value
    const savePath = await save({
      defaultPath: frameExtractFileName.value.replace(/\.[^.]+$/, '') + `_frame_${frameExtractTime.value.toFixed(1)}s.` + ext,
      filters: [{ name: ext.toUpperCase() + ' 图片', extensions: [ext] }],
    })
    if (!savePath) return
    await invoke('copy_file', { from: frameExtractResult.value.output_path, to: savePath })
    ElMessage.success(`已保存到: ${savePath}`)
  } catch (e: any) {
    ElMessage.error('保存失败: ' + (typeof e === 'string' ? e : e.message))
  }
}

async function copyFrameExtractResult() {
  if (!frameExtractResult.value || !frameExtractResultSrc.value) return
  try {
    // 从 base64 提取纯数据部分
    const base64Data = frameExtractResultSrc.value.replace(/^data:image\/\w+;base64,/, '')
    const mime = frameExtractFormat.value === 'png' ? 'image/png' : 'image/jpeg'
    await invoke('write_clipboard_image', { base64Data, mime })
    ElMessage.success('已复制到剪贴板')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '复制失败'
  }
}

function resetFrameExtract() {
  frameExtractFilePath.value = ''; frameExtractFileName.value = ''; frameExtractVideoInfo.value = null
  frameExtractResult.value = null; frameExtractResultSrc.value = ''; framePreviewSrc.value = ''
  error.value = ''
  if (framePreviewTimer) { clearTimeout(framePreviewTimer); framePreviewTimer = null }
}

// ============ 画面裁剪方法 (F25) ============
async function openCropRegionFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '视频文件', extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] }],
      multiple: false,
    })
    if (!selected) return
    cropRegionFilePath.value = selected as string
    cropRegionFileName.value = (selected as string).split(/[/\\]/).pop() || ''
    isLoadingInfo.value = true
    cropRegionResult.value = null
    cropPreviewSrc.value = ''
    const info: VideoInfo = await invoke('get_video_info', { path: cropRegionFilePath.value, useFfmpeg: true })
    cropRegionVideoInfo.value = info
    resetCropToFull()
    // 加载预览帧
    await loadCropPreview()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
  } finally { isLoadingInfo.value = false }
}

async function loadCropPreview() {
  if (!cropRegionFilePath.value) return
  // 超出时长自动钳制到末尾，不弹提示，直接预览最后一帧
  const dur = cropRegionVideoInfo.value?.duration ?? 0
  if (cropRegionPreviewTime.value > dur) {
    cropRegionPreviewTime.value = Number(dur.toFixed(1))
  }
  cropPreviewLoading.value = true
  try {
    const base64: string = await invoke('video_preview_frame', {
      path: cropRegionFilePath.value,
      timePoint: cropRegionPreviewTime.value,
      maxWidth: 480,
    })
    cropPreviewSrc.value = `data:image/jpeg;base64,${base64}`
  } catch {
    cropPreviewSrc.value = ''
    ElMessage.error('预览加载失败，请检查时间点或视频文件')
  } finally { cropPreviewLoading.value = false }
}

function onCropPreviewTimeChange() {
  loadCropPreview()
}

function onCropImgLoad() {
  if (!cropImgEl.value || !cropRegionVideoInfo.value) return
  cropDisplayRatio.value = cropImgEl.value.clientWidth / cropRegionVideoInfo.value.width
}

function resetCropToFull() {
  if (!cropRegionVideoInfo.value) return
  cropRegionX.value = 0
  cropRegionY.value = 0
  cropRegionW.value = cropRegionVideoInfo.value.width - (cropRegionVideoInfo.value.width % 2)
  cropRegionH.value = cropRegionVideoInfo.value.height - (cropRegionVideoInfo.value.height % 2)
  cropRegionPreset.value = ''
}

async function onCropPresetChange(preset: string) {
  if (!preset || !cropRegionVideoInfo.value) return
  try {
    const result: CropPresetResult = await invoke('calc_crop_preset', {
      origW: cropRegionVideoInfo.value.width,
      origH: cropRegionVideoInfo.value.height,
      preset,
    })
    cropRegionX.value = result.x
    cropRegionY.value = result.y
    cropRegionW.value = result.width
    cropRegionH.value = result.height
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '预设计算失败'
  }
}

// 裁剪框拖拽逻辑
function screenToVideo(screenX: number, screenY: number): { vx: number; vy: number } {
  if (!cropImgEl.value) return { vx: 0, vy: 0 }
  const rect = cropImgEl.value.getBoundingClientRect()
  const r = cropDisplayRatio.value
  return {
    vx: Math.round((screenX - rect.left) / r),
    vy: Math.round((screenY - rect.top) / r),
  }
}

function clampRegion(info: VideoInfo) {
  cropRegionX.value = Math.max(0, Math.min(cropRegionX.value, info.width - 2))
  cropRegionY.value = Math.max(0, Math.min(cropRegionY.value, info.height - 2))
  cropRegionW.value = Math.max(2, Math.min(cropRegionW.value, info.width - cropRegionX.value))
  cropRegionH.value = Math.max(2, Math.min(cropRegionH.value, info.height - cropRegionY.value))
  // 对齐到偶数
  cropRegionW.value = cropRegionW.value - (cropRegionW.value % 2)
  cropRegionH.value = cropRegionH.value - (cropRegionH.value % 2)
}

function onCropMouseDown(e: MouseEvent) {
  if (!cropRegionVideoInfo.value) return
  const { vx, vy } = screenToVideo(e.clientX, e.clientY)
  // 检查是否在裁剪区域内（用于移动）
  if (vx >= cropRegionX.value && vx <= cropRegionX.value + cropRegionW.value &&
      vy >= cropRegionY.value && vy <= cropRegionY.value + cropRegionH.value) {
    cropDragging.value = true
    cropDragType.value = 'move'
    cropDragStartX.value = vx
    cropDragStartY.value = vy
    cropDragStartRegion.value = {
      x: cropRegionX.value, y: cropRegionY.value,
      w: cropRegionW.value, h: cropRegionH.value,
    }
    e.preventDefault()
  }
}

function onCropMouseMove(e: MouseEvent) {
  if (!cropDragging.value || !cropRegionVideoInfo.value) return
  const { vx, vy } = screenToVideo(e.clientX, e.clientY)
  const info = cropRegionVideoInfo.value
  const dx = vx - cropDragStartX.value
  const dy = vy - cropDragStartY.value

  if (cropDragType.value === 'move') {
    cropRegionX.value = Math.max(0, Math.min(cropDragStartRegion.value.x + dx, info.width - cropDragStartRegion.value.w))
    cropRegionY.value = Math.max(0, Math.min(cropDragStartRegion.value.y + dy, info.height - cropDragStartRegion.value.h))
    clampRegion(info)
  } else {
    // 手柄拖拽
    const r = cropDragStartRegion.value
    switch (cropDragType.value) {
      case 'tl':
        cropRegionX.value = Math.max(0, Math.min(r.x + dx, r.x + r.w - 2))
        cropRegionY.value = Math.max(0, Math.min(r.y + dy, r.y + r.h - 2))
        cropRegionW.value = r.w - (cropRegionX.value - r.x)
        cropRegionH.value = r.h - (cropRegionY.value - r.y)
        break
      case 'tr':
        cropRegionY.value = Math.max(0, Math.min(r.y + dy, r.y + r.h - 2))
        cropRegionW.value = Math.max(2, Math.min(r.w + dx, info.width - r.x))
        cropRegionH.value = r.h - (cropRegionY.value - r.y)
        break
      case 'bl':
        cropRegionX.value = Math.max(0, Math.min(r.x + dx, r.x + r.w - 2))
        cropRegionW.value = r.w - (cropRegionX.value - r.x)
        cropRegionH.value = Math.max(2, Math.min(r.h + dy, info.height - r.y))
        break
      case 'br':
        cropRegionW.value = Math.max(2, Math.min(r.w + dx, info.width - r.x))
        cropRegionH.value = Math.max(2, Math.min(r.h + dy, info.height - r.y))
        break
    }
    clampRegion(info)
  }
}

function onCropMouseUp() {
  cropDragging.value = false
  cropDragType.value = null
}

function onCropHandleMouseDown(e: MouseEvent, handle: 'tl' | 'tr' | 'bl' | 'br') {
  if (!cropRegionVideoInfo.value) return
  const { vx, vy } = screenToVideo(e.clientX, e.clientY)
  cropDragging.value = true
  cropDragType.value = handle
  cropDragStartX.value = vx
  cropDragStartY.value = vy
  cropDragStartRegion.value = {
    x: cropRegionX.value, y: cropRegionY.value,
    w: cropRegionW.value, h: cropRegionH.value,
  }
  e.preventDefault()
  e.stopPropagation()
}

async function doCropRegion() {
  if (cropRegionW.value < 2 || cropRegionH.value < 2) {
    ElMessage.warning('裁剪宽高至少为 2')
    return
  }
  try {
    error.value = ''
    cropRegionProcessing.value = true
    cropRegionProgress.value = 0
    cropRegionResult.value = null
    const unlisten = await listen<{ progress: number }>('video-crop-region-progress', (event) => {
      cropRegionProgress.value = Math.round(event.payload.progress)
    })
    let outputPath: string | null = null
    if (!cropRegionSaveToSamePath.value) {
      outputPath = await save({
        defaultPath: cropRegionFileName.value.replace(/\.[^.]+$/, '') + '_cropped.mp4',
        filters: [{ name: 'MP4 视频', extensions: ['mp4'] }],
      })
      if (!outputPath) { unlisten(); cropRegionProcessing.value = false; return }
    }
    const result: CropRegionResult = await invoke('video_crop_region', {
      path: cropRegionFilePath.value,
      options: {
        x: cropRegionX.value,
        y: cropRegionY.value,
        width: cropRegionW.value,
        height: cropRegionH.value,
        output_path: outputPath,
      },
    })
    unlisten()
    cropRegionProgress.value = 100
    cropRegionResult.value = result
    ElMessage.success(`画面裁剪完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'videoTool',
      action: '画面裁剪',
      inputPreview: `${cropRegionFileName.value} [${cropRegionX},${cropRegionY} ${cropRegionW}x${cropRegionH}]`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: cropRegionFilePath.value,
      outputFull: result.output_path,
      options: {
        x: cropRegionX.value,
        y: cropRegionY.value,
        width: cropRegionW.value,
        height: cropRegionH.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '画面裁剪失败'
  } finally { cropRegionProcessing.value = false }
}

function resetCropRegion() {
  cropRegionFilePath.value = ''; cropRegionFileName.value = ''; cropRegionVideoInfo.value = null
  cropRegionResult.value = null; cropPreviewSrc.value = ''; error.value = ''
  cropRegionPreset.value = ''
}

// ============ 格式化 ============
function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = (seconds % 60).toFixed(1)
  return `${String(m).padStart(2, '0')}:${String(parseFloat(s)).padStart(4, '0')}`
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

// ============ 响应式 ============
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  try { useFfmpeg.value = await invoke('check_ffmpeg') } catch { /* 忽略 */ }
  ffmpegChecked.value = true
  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => drawTimeline())
    resizeObserver.observe(canvasRef.value)
  }
})

// ponytail: KeepAlive 缓存后重新激活时重绘 canvas（onMounted 不会再次触发）
onActivated(() => {
  if (videoInfo.value && activeTab.value === 'crop') {
    nextTick(() => drawTimeline())
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

onActivated(() => {
  // 从历史记录恢复（视频工具均为文件操作，文件路径可能已失效，不还原文件，仅切换 Tab）
  const restore = store.pendingHistoryRestore
  if (!restore) return
  if (restore.tool === 'videoTool') {
    // 根据 action 切换到对应 Tab
    const actionTabMap: Record<string, string> = {
      '裁剪': 'crop',
      '转码': 'transcode',
      '音频提取': 'audioExtract',
      '压缩': 'compress',
      '合并': 'merge',
      '截图提取': 'frameExtract',
      '画面裁剪': 'cropRegion',
    }
    if (restore.action) {
      const tab = actionTabMap[restore.action]
      if (tab) {
        activeTab.value = tab
        ElMessage.success(`已跳转到${restore.action}，请重新选择文件`)
      }
    }
  }
  store.clearHistoryRestore()
})

// ponytail: 拖拽时只重绘选中区域，不重绘缩略图，避免闪烁
watch([startTime, endTime], () => drawSelection())
// ponytail: 缩略图变化时重绘整个时间轴
watch(thumbnails, () => nextTick(() => drawTimeline()), { deep: true })
// 切换 Tab 时清理共享的 error 状态
watch(activeTab, () => { error.value = '' })
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.video-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .video-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.video-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.video-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.video-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.video-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.video-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 页面特有样式 ===== */
.video-file-info {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-name {
  color: var(--accent-cyan);
  font-size: 14px;
  font-weight: 500;
}

.file-detail {
  color: var(--text-secondary);
  font-size: 12px;
}

.timeline-container {
  position: relative;
  width: 100%;
  height: 180px;
  cursor: pointer;
}

.timeline-container:active, .slider-handle:active {
  cursor: col-resize;
}

.timeline-canvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border-radius: 4px;
}

.thumbnail-layer {
  z-index: 1;
}

.selection-layer {
  z-index: 2;
}

.slider-handle {
  position: absolute;
  top: 0;
  width: 12px;
  height: 100%;
  transform: translateX(-50%);
  cursor: col-resize;
  z-index: 10;
}

.slider-handle::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent-cyan);
  border: 2px solid var(--bg-primary);
  box-shadow: 0 0 4px rgba(0, 0, 0, 0.4);
}

.timeline-labels {
  display: flex;
  justify-content: space-between;
  color: var(--text-secondary);
  font-size: 12px;
  margin-top: 4px;
  padding: 0 6px;
}

.unit-text {
  color: var(--text-secondary);
  font-size: 12px;
  margin-left: 4px;
}

.segment-info {
  margin-top: 8px;
  color: var(--accent-cyan);
  font-size: 13px;
}

.keyframe-hint {
  margin-top: 4px;
  color: var(--accent-orange);
  font-size: 12px;
}

.result-info {
  margin-top: 12px;
  color: var(--accent-green);
  font-size: 13px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.result-sep {
  color: var(--text-secondary);
}

/* ffmpeg 状态横幅 */
.ffmpeg-banner {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 12px;
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

.ffmpeg-icon {
  font-size: 16px;
}

.ffmpeg-tip {
  margin-left: 8px;
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

.ffmpeg-required {
  color: var(--accent-orange);
  font-size: 14px;
  text-align: center;
  padding: 16px;
}

/* 合并文件列表 */
.merge-file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.merge-file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.merge-file-idx {
  color: var(--accent-cyan);
  font-weight: 600;
  min-width: 24px;
}

.merge-file-name {
  color: var(--text-primary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.merge-file-size {
  color: var(--text-secondary);
  font-size: 12px;
  min-width: 60px;
  text-align: right;
}

.merge-hint {
  margin-top: 12px;
  color: var(--accent-green);
  font-size: 12px;
}

.merge-hint-warn {
  margin-top: 12px;
  color: var(--accent-orange);
  font-size: 12px;
}

.empty-hint {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

.crf-slider {
  display: flex;
  align-items: center;
}

/* 画面裁剪区域 */
.crop-region-grid {
  display: flex;
  gap: 16px;
  margin-top: 12px;
  flex-wrap: wrap;
}

.crop-region-input {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.crop-region-info {
  margin-top: 12px;
  color: var(--text-secondary);
  font-size: 13px;
}

/* 裁剪预览 */
.crop-preview-wrapper {
  position: relative;
  display: inline-block;
  max-width: 100%;
  border-radius: 6px;
  overflow: hidden;
  background: #000;
  cursor: crosshair;
}

.crop-preview-img {
  display: block;
  max-width: 100%;
  height: auto;
}

.crop-preview-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 480px;
  height: 270px;
  color: var(--text-secondary);
  font-size: 14px;
  background: var(--bg-input);
}

/* 加载时整体压暗，复用 F24 的 is-loading 模式 */
.crop-preview-wrapper.is-loading .crop-preview-img {
  opacity: 0.5;
}

.crop-preview-mask {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.35);
  pointer-events: none;
  z-index: 5;
}

.crop-preview-mask-text {
  padding: 6px 14px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  font-size: 13px;
  letter-spacing: 1px;
}

.crop-overlay-box {
  position: absolute;
  pointer-events: none;
}

.crop-mask {
  position: absolute;
  background: rgba(0, 0, 0, 0.5);
}

.crop-mask-top {
  top: 0;
  left: 0;
  right: 0;
}

.crop-mask-bottom {
  left: 0;
  right: 0;
  bottom: 0;
}

.crop-mask-left { }
.crop-mask-right { right: 0; }

.crop-border {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  border: 2px dashed var(--accent-cyan);
  box-sizing: border-box;
}

.crop-handle {
  position: absolute;
  width: 12px;
  height: 12px;
  background: var(--accent-cyan);
  border: 2px solid #000;
  border-radius: 2px;
  pointer-events: auto;
  z-index: 10;
}

.crop-handle-tl { top: -6px; left: -6px; cursor: nw-resize; }
.crop-handle-tr { top: -6px; right: -6px; cursor: ne-resize; }
.crop-handle-bl { bottom: -6px; left: -6px; cursor: sw-resize; }
.crop-handle-br { bottom: -6px; right: -6px; cursor: se-resize; }

.crop-dim-hint {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 截图预览 */
.frame-preview-container {
  margin-top: 16px;
}

.frame-preview-label {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.frame-preview-box {
  border-radius: 6px;
  overflow: hidden;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
  width: 100%;
  max-width: 480px;
}

.frame-preview-box.is-loading {
  opacity: 0.6;
}

.frame-preview-img {
  display: block;
  max-width: 100%;
  height: auto;
}

.frame-preview-hint {
  color: var(--text-secondary);
  font-size: 13px;
  padding: 20px;
}

.frame-result-container {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}
</style>