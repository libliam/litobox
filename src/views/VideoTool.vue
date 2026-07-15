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
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '合并失败'
  } finally { mergeProcessing.value = false }
}

function resetMerge() {
  mergeFiles.value = []; mergeResult.value = null; error.value = ''
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
</style>