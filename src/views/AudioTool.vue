<template>
  <div class="tool-container">
    <!-- ffmpeg 状态提示 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': useFfmpeg, 'ffmpeg-missing': !useFfmpeg }" v-if="ffmpegChecked">
      <template v-if="useFfmpeg">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，处理速度更快、音频信息更准确
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        未检测到 ffmpeg，当前使用内置引擎（功能完整，速度较慢）。
        <span class="ffmpeg-tip">
          安装 ffmpeg 可加速处理：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
          <a href="https://www.wikihow.com/Install-FFmpeg-on-Windows" target="_blank" class="ffmpeg-link">详细教程</a>
        </span>
      </template>
    </div>

    <!-- Tab 栏（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="audio-tool-tabs">
        <el-tab-pane label="音频裁剪" name="crop" />
        <el-tab-pane label="格式转换" name="convert" />
        <el-tab-pane label="音频压缩" name="compress" />
        <el-tab-pane label="音频合并" name="merge" />
        <el-tab-pane label="变速变调" name="speed" />
        <el-tab-pane label="文字转语音" name="tts" />
      </el-tabs>
    </div>

    <!-- ====== Tab: 音频裁剪 ====== -->
    <template v-if="activeTab === 'crop'">
      <!-- 文件选择 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择音频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openFile" :loading="isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="filePath" class="audio-file-info">
            <span class="file-name">{{ fileName }}</span>
            <span class="file-detail" v-if="audioInfo">
              {{ formatDuration(audioInfo.duration) }} | {{ audioInfo.format.toUpperCase() }} |
              {{ audioInfo.sample_rate }}Hz | {{ audioInfo.channels === 2 ? '立体声' : '单声道' }} |
              {{ audioInfo.bitrate }}kbps
            </span>
          </div>
        </div>
      </div>

      <!-- 波形预览 -->
      <div v-if="waveformData.points.length > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">波形预览</span>
        </div>
        <div class="card-body">
          <div class="waveform-container" ref="waveformContainer" @contextmenu.prevent>
            <canvas ref="canvasRef" class="waveform-canvas" @mousedown="onCanvasMouseDown" @contextmenu.prevent></canvas>
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
          <div class="waveform-labels">
            <span>{{ formatTime(startTime) }}</span>
            <span>{{ formatTime(endTime) }}</span>
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-button size="small" @click="togglePreview" :type="isPreviewing ? 'danger' : 'default'" :loading="isPreviewLoading">
                {{ isPreviewing ? (isPreviewLoading ? '加载中…' : '⏹ 停止') : '▶ 预览选中区域' }}
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 裁剪设置 -->
      <div v-if="waveformData.points.length > 0" class="tool-card">
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
                :max="waveformData.duration"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 140px"
              />
              <span class="unit-text">秒</span>
            </div>
            <div class="action-group">
              <div class="group-label">输出格式</div>
              <el-select v-model="outputFormat" size="small" style="width: 100px">
                <el-option label="MP3" value="mp3" />
                <el-option label="WAV" value="wav" />
              </el-select>
            </div>
            <div class="action-group" v-if="outputFormat === 'mp3'">
              <div class="group-label">比特率</div>
              <el-select v-model="mp3Bitrate" size="small" style="width: 120px">
                <el-option label="128 kbps" :value="128" />
                <el-option label="192 kbps" :value="192" />
                <el-option label="256 kbps" :value="256" />
                <el-option label="320 kbps" :value="320" />
              </el-select>
            </div>
          </div>
          <div class="segment-info" v-if="audioInfo">
            片段时长: {{ formatDuration(segmentDuration) }}
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

      <!-- 操作 -->
      <div v-if="waveformData.points.length > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="cropAudio" :loading="isProcessing" :disabled="!isRangeValid">
                裁剪并导出
              </el-button>
              <el-button size="small" @click="resetForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="isProcessing" :percentage="cropProgress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- ====== Tab: 格式转换 ====== -->
    <template v-if="activeTab === 'convert'">
      <!-- 文件选择 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择音频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openConvertFile" :loading="convertState.isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="convertState.filePath" class="audio-file-info">
            <span class="file-name">{{ convertState.fileName }}</span>
            <span class="file-detail" v-if="convertState.audioInfo">
              {{ formatDuration(convertState.audioInfo.duration) }} | {{ convertState.audioInfo.format.toUpperCase() }} |
              {{ convertState.audioInfo.sample_rate }}Hz | {{ convertState.audioInfo.channels === 2 ? '立体声' : '单声道' }} |
              {{ convertState.audioInfo.bitrate }}kbps
            </span>
          </div>
        </div>
      </div>

      <!-- 转换设置 -->
      <div v-if="convertState.filePath" class="tool-card">
        <div class="card-header">
          <span class="card-title">转换设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">输出格式</div>
              <el-select v-model="convertState.outputFormat" size="small" style="width: 120px">
                <el-option label="MP3" value="mp3" />
                <el-option label="WAV" value="wav" />
                <el-option label="M4A/AAC" value="m4a" />
                <el-option label="FLAC" value="flac" />
                <el-option label="OGG" value="ogg" />
              </el-select>
            </div>
            <div class="action-group" v-if="['mp3', 'm4a', 'ogg'].includes(convertState.outputFormat)">
              <div class="group-label">比特率</div>
              <el-select v-model="convertState.bitrate" size="small" style="width: 120px">
                <el-option label="128 kbps" :value="128" />
                <el-option label="192 kbps" :value="192" />
                <el-option label="256 kbps" :value="256" />
                <el-option label="320 kbps" :value="320" />
              </el-select>
            </div>
            <div class="action-group">
              <div class="group-label">采样率</div>
              <el-select v-model="convertState.sampleRate" size="small" style="width: 120px">
                <el-option label="原始" :value="null" />
                <el-option label="44100 Hz" :value="44100" />
                <el-option label="48000 Hz" :value="48000" />
                <el-option label="96000 Hz" :value="96000" />
              </el-select>
            </div>
            <div class="action-group">
              <div class="group-label">声道</div>
              <el-select v-model="convertState.channels" size="small" style="width: 120px">
                <el-option label="原始" :value="null" />
                <el-option label="单声道" :value="1" />
                <el-option label="立体声" :value="2" />
              </el-select>
            </div>
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="convertState.saveToSamePath" size="small">
                与源文件相同路径
              </el-checkbox>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作 -->
      <div v-if="convertState.filePath" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="convertAudio" :loading="convertState.isProcessing">
                转换并导出
              </el-button>
              <el-button size="small" @click="resetConvertForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="convertState.isProcessing" :percentage="convertState.progress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- ====== Tab: 音频压缩 ====== -->
    <template v-if="activeTab === 'compress'">
      <!-- 文件选择 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择音频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openCompressFile" :loading="compressState.isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="compressState.filePath" class="audio-file-info">
            <span class="file-name">{{ compressState.fileName }}</span>
            <span class="file-detail" v-if="compressState.audioInfo">
              {{ formatDuration(compressState.audioInfo.duration) }} | {{ compressState.audioInfo.format.toUpperCase() }} |
              {{ (compressState.audioInfo.file_size / 1024 / 1024).toFixed(2) }} MB |
              {{ compressState.audioInfo.bitrate }}kbps
            </span>
          </div>
        </div>
      </div>

      <!-- 压缩设置 -->
      <div v-if="compressState.filePath" class="tool-card">
        <div class="card-header">
          <span class="card-title">压缩设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">压缩模式</div>
              <el-select v-model="compressState.mode" size="small" style="width: 140px">
                <el-option label="目标比特率" value="bitrate" />
                <el-option label="质量等级" value="quality" />
              </el-select>
            </div>
            <div class="action-group" v-if="compressState.mode === 'bitrate'">
              <div class="group-label">目标比特率</div>
              <el-select v-model="compressState.bitrate" size="small" style="width: 120px">
                <el-option label="64 kbps" :value="64" />
                <el-option label="96 kbps" :value="96" />
                <el-option label="128 kbps" :value="128" />
                <el-option label="192 kbps" :value="192" />
              </el-select>
            </div>
            <div class="action-group" v-if="compressState.mode === 'quality'">
              <div class="group-label">质量等级</div>
              <el-select v-model="compressState.quality" size="small" style="width: 120px">
                <el-option label="低 (64kbps)" value="low" />
                <el-option label="中 (128kbps)" value="medium" />
                <el-option label="高 (192kbps)" value="high" />
              </el-select>
            </div>
            <div class="action-group">
              <div class="group-label">采样率</div>
              <el-select v-model="compressState.sampleRate" size="small" style="width: 120px">
                <el-option label="原始" :value="null" />
                <el-option label="44100 Hz" :value="44100" />
                <el-option label="22050 Hz" :value="22050" />
              </el-select>
            </div>
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="compressState.saveToSamePath" size="small">
                与源文件相同路径
              </el-checkbox>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作 -->
      <div v-if="compressState.filePath" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="compressAudio" :loading="compressState.isProcessing">
                压缩并导出
              </el-button>
              <el-button size="small" @click="resetCompressForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="compressState.isProcessing" :percentage="compressState.progress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- ====== Tab: 音频合并 ====== -->
    <template v-if="activeTab === 'merge'">
      <!-- 文件列表 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">添加音频文件</span>
          <div class="card-actions">
            <el-button size="small" @click="addMergeFiles">添加文件</el-button>
            <el-button size="small" @click="clearMergeFiles" :disabled="mergeState.files.length === 0">清空列表</el-button>
          </div>
        </div>
        <div class="card-body">
          <div v-if="mergeState.files.length === 0" class="merge-empty">
            点击"添加文件"选择多个音频文件
          </div>
          <div v-else class="merge-file-list">
            <div
              v-for="(file, index) in mergeState.files"
              :key="file.path"
              class="merge-file-item"
              draggable="true"
              @dragstart="onDragStart($event, index)"
              @dragover.prevent="onDragOver($event, index)"
              @drop="onDrop($event, index)"
              :class="{ 'dragging': mergeState.dragIndex === index }"
            >
              <span class="merge-file-index">{{ index + 1 }}</span>
              <span class="merge-file-name">{{ file.name }}</span>
              <span class="merge-file-duration" v-if="file.duration">{{ formatDuration(file.duration) }}</span>
              <el-button size="small" type="danger" text @click="removeMergeFile(index)">删除</el-button>
            </div>
          </div>
          <div v-if="mergeState.files.length > 0" class="merge-total-info">
            共 {{ mergeState.files.length }} 个文件，总时长: {{ formatDuration(mergeTotalDuration) }}
          </div>
        </div>
      </div>

      <!-- 合并设置 -->
      <div v-if="mergeState.files.length >= 2" class="tool-card">
        <div class="card-header">
          <span class="card-title">合并设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">输出格式</div>
              <el-select v-model="mergeState.outputFormat" size="small" style="width: 120px">
                <el-option label="MP3" value="mp3" />
                <el-option label="WAV" value="wav" />
                <el-option label="M4A/AAC" value="m4a" />
              </el-select>
            </div>
            <div class="action-group" v-if="['mp3', 'm4a'].includes(mergeState.outputFormat)">
              <div class="group-label">比特率</div>
              <el-select v-model="mergeState.bitrate" size="small" style="width: 120px">
                <el-option label="128 kbps" :value="128" />
                <el-option label="192 kbps" :value="192" />
                <el-option label="256 kbps" :value="256" />
                <el-option label="320 kbps" :value="320" />
              </el-select>
            </div>
            <div class="action-group">
              <div class="group-label">合并模式</div>
              <el-select v-model="mergeState.mode" size="small" style="width: 140px">
                <el-option label="自动 (推荐)" value="auto" />
                <el-option label="强制转码" value="force_transcode" />
              </el-select>
            </div>
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="mergeState.saveToSamePath" size="small">
                与第一个文件相同路径
              </el-checkbox>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作 -->
      <div v-if="mergeState.files.length >= 2" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="mergeAudio" :loading="mergeState.isProcessing">
                合并并导出
              </el-button>
              <el-button size="small" @click="resetMergeForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="mergeState.isProcessing" :percentage="mergeState.progress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- ====== Tab: 变速变调 ====== -->
    <template v-if="activeTab === 'speed'">
      <!-- 文件选择 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择音频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openSpeedFile" :loading="speedState.isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="speedState.filePath" class="audio-file-info">
            <span class="file-name">{{ speedState.fileName }}</span>
            <span class="file-detail" v-if="speedState.audioInfo">
              {{ formatDuration(speedState.audioInfo.duration) }} | {{ speedState.audioInfo.format.toUpperCase() }} |
              {{ speedState.audioInfo.sample_rate }}Hz | {{ speedState.audioInfo.channels === 2 ? '立体声' : '单声道' }} |
              {{ speedState.audioInfo.bitrate }}kbps
            </span>
          </div>
        </div>
      </div>

      <!-- 变速设置 -->
      <div v-if="speedState.filePath" class="tool-card">
        <div class="card-header">
          <span class="card-title">变速设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">播放速度</div>
              <el-slider
                v-model="speedState.speed"
                :min="0.5"
                :max="4.0"
                :step="0.1"
                :format-tooltip="(val: number) => val.toFixed(1) + 'x'"
                style="width: 200px"
              />
              <el-input-number
                v-model="speedState.speed"
                :min="0.5"
                :max="4.0"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 120px; margin-left: 12px"
              />
              <span class="unit-text">x</span>
            </div>
            <div class="action-group">
              <el-checkbox v-model="speedState.keepPitch" size="small">
                保持音调 (推荐)
              </el-checkbox>
            </div>
          </div>
          <div class="speed-info" v-if="speedState.audioInfo">
            输出时长: {{ formatDuration(speedState.audioInfo.duration) }} → {{ formatDuration(speedState.audioInfo.duration / speedState.speed) }}
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <div class="group-label">输出格式</div>
              <el-select v-model="speedState.outputFormat" size="small" style="width: 120px">
                <el-option label="MP3" value="mp3" />
                <el-option label="WAV" value="wav" />
                <el-option label="M4A/AAC" value="m4a" />
              </el-select>
            </div>
            <div class="action-group" v-if="['mp3', 'm4a'].includes(speedState.outputFormat)">
              <div class="group-label">比特率</div>
              <el-select v-model="speedState.bitrate" size="small" style="width: 120px">
                <el-option label="128 kbps" :value="128" />
                <el-option label="192 kbps" :value="192" />
                <el-option label="256 kbps" :value="256" />
                <el-option label="320 kbps" :value="320" />
              </el-select>
            </div>
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="speedState.saveToSamePath" size="small">
                与源文件相同路径
              </el-checkbox>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作 -->
      <div v-if="speedState.filePath" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="changeSpeed" :loading="speedState.isProcessing">
                导出并保存
              </el-button>
              <el-button size="small" @click="resetSpeedForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="speedState.isProcessing" :percentage="speedState.progress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- ====== Tab: 文字转语音 ====== -->
    <template v-if="activeTab === 'tts'">
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">输入文字</span>
          <div class="card-actions">
            <el-button size="small" @click="tsState.text = ''" :disabled="!tsState.text">清空</el-button>
            <el-button size="small" @click="pasteTtsText" :disabled="tsState.isProcessing">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="tsState.text"
            type="textarea"
            :rows="6"
            placeholder="输入要转换为语音的文字..."
            resize="vertical"
            :disabled="tsState.isProcessing"
          />
          <div class="ts-char-count" v-if="tsState.text.length">
            {{ tsState.text.length }} 字 · 预计 {{ Math.max(1, Math.ceil(tsState.text.length / 4)) }} 秒
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">语音设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">引擎</div>
              <el-select
                v-model="tsState.engine"
                size="small"
                style="width: 140px"
                :disabled="tsState.isProcessing"
                @change="onEngineChange"
              >
                <el-option label="SAPI 经典" value="sapi" />
                <el-option label="WinRT 神经语音" value="winrt" />
              </el-select>
            </div>
            <div class="action-group">
              <div class="group-label">语音</div>
              <el-select
                v-model="tsState.voiceName"
                size="small"
                style="width: 220px"
                placeholder="默认语音"
                clearable
                :disabled="tsState.isProcessing"
              >
                <el-option
                  v-for="v in filteredVoices"
                  :key="v.name + v.engine"
                  :label="`${v.name} (${v.language})`"
                  :value="v.name"
                />
              </el-select>
            </div>
            <div class="action-group">
              <div class="group-label">语速: {{ tsState.rate > 0 ? '+' : '' }}{{ tsState.rate }}</div>
              <el-slider
                v-model="tsState.rate"
                :min="-10"
                :max="10"
                :step="1"
                :disabled="tsState.isProcessing"
                :format-tooltip="(val: number) => (val > 0 ? '+' : '') + val"
                style="width: 160px"
              />
            </div>
            <div class="action-group">
              <div class="group-label">音量: {{ tsState.volume }}</div>
              <el-slider
                v-model="tsState.volume"
                :min="0"
                :max="100"
                :step="5"
                :disabled="tsState.isProcessing"
                style="width: 160px"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button
                type="primary"
                size="small"
                @click="generateTts"
                :loading="tsState.isProcessing"
                :disabled="!tsState.text.trim()"
              >
                生成语音
              </el-button>
              <el-button
                size="small"
                @click="previewTts"
                :disabled="!tsState.resultPath || tsState.isProcessing"
              >
                试听
              </el-button>
              <el-button
                size="small"
                @click="locateTtsFile"
                :disabled="!tsState.resultPath || tsState.isProcessing"
              >
                在文件夹中打开
              </el-button>
              <el-button
                size="small"
                type="primary"
                @click="saveAsTtsFile"
                :disabled="!tsState.resultPath || tsState.isProcessing"
              >
                另存为
              </el-button>
            </div>
          </div>
          <el-progress
            v-if="tsState.isProcessing"
            :percentage="tsState.progress"
            :stroke-width="6"
            style="margin-top: 12px"
          />
        </div>
      </div>

      <div v-if="tsState.resultPath" class="tool-card">
        <div class="card-header">
          <span class="card-title">生成结果</span>
        </div>
        <div class="card-body">
          <div class="ts-result-info">
            <span class="ts-result-label">文件路径:</span>
            <span class="ts-result-path">{{ tsState.resultPath }}</span>
          </div>
          <div class="ts-result-info">
            <span class="ts-result-label">文件大小:</span>
            <span class="ts-result-path">{{ formatBytes(tsState.resultSize) }}</span>
          </div>
        </div>
      </div>

      <!-- 试听弹窗 -->
      <el-dialog v-model="tsState.showPreview" title="试听" width="400px" :close-on-click-modal="true" @close="stopTtsPreview">
        <div class="ts-preview-container">
          <audio
            ref="ttsAudioRef"
            v-if="tsState.audioBase64"
            :src="'data:audio/wav;base64,' + tsState.audioBase64"
            controls
            autoplay
            class="ts-audio-player"
          />
        </div>
      </el-dialog>
    </template>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, onActivated, nextTick, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ 类型定义 ============
interface AudioInfo {
  duration: number
  sample_rate: number
  channels: number
  format: string
  bitrate: number
  file_size: number
}

interface WaveformData {
  points: number[]
  duration: number
  sample_rate: number
}

interface CropResult {
  output_path: string
  output_size: number
  duration: number
}

interface ConvertResult {
  output_path: string
  output_size: number
}

interface CompressResult {
  output_path: string
  output_size: number
  original_size: number
}

interface MergeResult {
  output_path: string
  output_size: number
  duration: number
}

interface SpeedChangeResult {
  output_path: string
  output_size: number
  duration: number
}

interface MergeFile {
  path: string
  name: string
  duration: number
}

interface TtsVoice {
  name: string
  language: string
  engine: string
}

interface TtsResult {
  output_path: string
  output_size: number
}

// ============ Tab 状态 ============
const activeTab = ref('crop')

// ============ 格式转换状态 ============
const convertState = reactive({
  filePath: '',
  fileName: '',
  audioInfo: null as AudioInfo | null,
  outputFormat: 'mp3',
  bitrate: 192,
  sampleRate: null as number | null,
  channels: null as number | null,
  saveToSamePath: true,
  isProcessing: false,
  isLoadingInfo: false,
  progress: 0,
})

// ============ 音频压缩状态 ============
const compressState = reactive({
  filePath: '',
  fileName: '',
  audioInfo: null as AudioInfo | null,
  mode: 'bitrate',
  bitrate: 128,
  quality: 'medium',
  sampleRate: null as number | null,
  saveToSamePath: true,
  isProcessing: false,
  isLoadingInfo: false,
  progress: 0,
})

// ============ 音频合并状态 ============
const mergeState = reactive({
  files: [] as MergeFile[],
  outputFormat: 'mp3',
  bitrate: 192,
  mode: 'auto',
  saveToSamePath: true,
  isProcessing: false,
  progress: 0,
  dragIndex: -1,
})

const mergeTotalDuration = computed(() => {
  return mergeState.files.reduce((sum, f) => sum + f.duration, 0)
})

// ============ 变速变调状态 ============
const speedState = reactive({
  filePath: '',
  fileName: '',
  audioInfo: null as AudioInfo | null,
  speed: 1.0,
  keepPitch: true,
  outputFormat: 'mp3',
  bitrate: 192,
  saveToSamePath: true,
  isProcessing: false,
  isLoadingInfo: false,
  progress: 0,
})

// ============ TTS 状态 ============
const tsState = reactive({
  text: '',
  voiceName: '',
  voices: [] as TtsVoice[],
  engine: 'sapi' as 'sapi' | 'winrt',
  rate: 0,
  volume: 100,
  isProcessing: false,
  progress: 0,
  resultPath: '',
  resultSize: 0,
  audioBase64: '',
  showPreview: false,
})

const filteredVoices = computed(() => {
  return tsState.voices.filter(v => v.engine === tsState.engine)
})

// ============ 状态 ============
const filePath = ref('')
const fileName = ref('')
const audioInfo = ref<AudioInfo | null>(null)
const waveformData = ref<WaveformData>({ points: [], duration: 0, sample_rate: 44100 })
const startTime = ref(0)
const endTime = ref(0)
const outputFormat = ref<'mp3' | 'wav'>('mp3')
const mp3Bitrate = ref(192)
const isProcessing = ref(false)
const isLoadingInfo = ref(false)
const isPreviewing = ref(false)
const saveToSamePath = ref(true)
const cropProgress = ref(0)
const useFfmpeg = ref(false)
const ffmpegChecked = ref(false)
const error = ref('')

// ============ 计算属性 ============
const segmentDuration = computed(() => endTime.value - startTime.value)
const isRangeValid = computed(() => startTime.value < endTime.value && segmentDuration.value >= 0.1)

// ============ Canvas ============
const canvasRef = ref<HTMLCanvasElement | null>(null)
const waveformContainer = ref<HTMLDivElement | null>(null)

function drawWaveform() {
  const canvas = canvasRef.value
  if (!canvas || !waveformData.value.points.length) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.scale(dpr, dpr)
  const width = rect.width
  const height = rect.height
  const data = waveformData.value.points
  const n = data.length
  const dur = waveformData.value.duration

  // 背景
  const style = getComputedStyle(document.documentElement)
  const bgColor = style.getPropertyValue('--bg-input').trim() || '#0d1520'
  const primaryColor = style.getPropertyValue('--accent-cyan').trim() || '#00d4ff'
  const secondaryColor = style.getPropertyValue('--text-secondary').trim() || '#94a3b8'

  ctx.fillStyle = bgColor
  ctx.fillRect(0, 0, width, height)

  const startX = (startTime.value / dur) * width
  const endX = (endTime.value / dur) * width
  const barWidth = width / n
  const midY = height / 2

  for (let i = 0; i < n; i++) {
    const x = i * barWidth
    const barHeight = data[i] * midY * 0.9

    if (x >= startX && x <= endX) {
      ctx.fillStyle = primaryColor
    } else {
      ctx.fillStyle = secondaryColor + '66'
    }

    ctx.fillRect(x, midY - barHeight / 2, Math.max(barWidth, 1), barHeight || 1)
  }

  // 选中区域高亮覆盖
  ctx.fillStyle = primaryColor + '1A'
  ctx.fillRect(startX, 0, endX - startX, height)
}

function timeToPercent(time: number): number {
  if (waveformData.value.duration <= 0) return 0
  return (time / waveformData.value.duration) * 100
}

function percentToTime(percent: number): number {
  return Math.round((percent / 100) * waveformData.value.duration * 10) / 10
}

// ============ 滑块拖拽 ============
let draggingSlider: 'start' | 'end' | null = null

function onSliderMouseDown(_e: MouseEvent, slider: 'start' | 'end') {
  draggingSlider = slider
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!draggingSlider || !waveformContainer.value) return
  const rect = waveformContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  if (draggingSlider === 'start') {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawWaveform()
}

function onMouseUp() {
  draggingSlider = null
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown(e: MouseEvent) {
  if (!waveformContainer.value) return
  const rect = waveformContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  // 点击靠近哪个滑块就移动哪个
  const startDist = Math.abs(time - startTime.value)
  const endDist = Math.abs(time - endTime.value)

  if (startDist <= endDist) {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawWaveform()
}

// ============ 音频预览 ============
let audioCtx: AudioContext | null = null
let audioSource: AudioBufferSourceNode | null = null
let previewAbortFlag = false
const isPreviewLoading = ref(false)

async function togglePreview() {
  if (isPreviewing.value) {
    stopPreview()
    return
  }
  previewAudio()
}

async function previewAudio() {
  // 立即标记为预览中，按钮变为"停止"
  isPreviewing.value = true
  isPreviewLoading.value = true
  previewAbortFlag = false
  error.value = ''
  // 停止之前的播放
  if (audioSource) {
    try { audioSource.stop() } catch (_) { /* 忽略 */ }
    audioSource.disconnect()
    audioSource = null
  }

  try {
    const base64Wav: string = await invoke('get_audio_preview', {
      path: filePath.value,
      start: startTime.value,
      end: endTime.value,
    })

    // 加载期间被取消
    if (previewAbortFlag) return

    const binaryStr = atob(base64Wav)
    const bytes = new Uint8Array(binaryStr.length)
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i)
    }

    if (!audioCtx) {
      audioCtx = new AudioContext()
    }
    await audioCtx.resume()

    if (previewAbortFlag) return

    const audioBuffer = await audioCtx.decodeAudioData(bytes.buffer.slice(0))
    audioSource = audioCtx.createBufferSource()
    audioSource.buffer = audioBuffer
    audioSource.connect(audioCtx.destination)
    audioSource.onended = () => { isPreviewing.value = false }
    audioSource.start()
    isPreviewLoading.value = false
  } catch (e: any) {
    if (!previewAbortFlag) {
      error.value = '预览播放失败: ' + (typeof e === 'string' ? e : e.message || e)
    }
    isPreviewing.value = false
    isPreviewLoading.value = false
  }
}

function stopPreview() {
  previewAbortFlag = true
  if (audioSource) {
    try { audioSource.stop() } catch (_) { /* 忽略已停止错误 */ }
    audioSource.disconnect()
    audioSource = null
  }
  isPreviewing.value = false
  isPreviewLoading.value = false
}

// ============ 文件操作 ============
async function openFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a'] }],
      multiple: false,
    })
    if (!selected) return

    filePath.value = selected as string
    fileName.value = (selected as string).split(/[/\\]/).pop() || ''

    isLoadingInfo.value = true
    const info: AudioInfo = await invoke('get_audio_info', { path: filePath.value, useFfmpeg: useFfmpeg.value })
    audioInfo.value = info

    const wf: WaveformData = await invoke('generate_waveform', { path: filePath.value })
    waveformData.value = wf
    // 用实际解码时长更新
    audioInfo.value.duration = wf.duration

    startTime.value = 0
    endTime.value = wf.duration
    await nextTick()
    drawWaveform()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetForm()
  } finally {
    isLoadingInfo.value = false
  }
}

async function cropAudio() {
  if (!isRangeValid.value) {
    ElMessage.warning('请设置有效的裁剪区间')
    return
  }

  try {
    error.value = ''
    isProcessing.value = true
    cropProgress.value = 0

    // 监听进度事件
    const unlisten = await listen<{ progress: number }>('audio-crop-progress', (event) => {
      cropProgress.value = Math.round(event.payload.progress)
    })

    // 确定输出路径
    let outputPath: string | null = null
    if (!saveToSamePath.value) {
      const defaultName = fileName.value.replace(/\.[^.]+$/, '') + '_cropped.' + outputFormat.value
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: '音频文件', extensions: [outputFormat.value] }],
      })
      if (!outputPath) {
        unlisten()
        isProcessing.value = false
        return // 用户取消
      }
    }

    const result: CropResult = await invoke('audio_crop', {
      path: filePath.value,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
        output_format: outputFormat.value,
        mp3_bitrate: mp3Bitrate.value,
        output_path: outputPath,
        use_ffmpeg: useFfmpeg.value,
      },
    })

    unlisten()
    cropProgress.value = 100
    ElMessage.success(`裁剪完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'audioTool',
      action: '裁剪',
      inputPreview: `${fileName.value} [${formatDuration(startTime.value)} - ${formatDuration(endTime.value)}]`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: filePath.value,
      outputFull: result.output_path,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
        output_format: outputFormat.value,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '裁剪失败'
  } finally {
    isProcessing.value = false
  }
}

function resetForm() {
  filePath.value = ''
  fileName.value = ''
  audioInfo.value = null
  waveformData.value = { points: [], duration: 0, sample_rate: 44100 }
  startTime.value = 0
  endTime.value = 0
  error.value = ''
  stopPreview()
}

// ============ 格式转换操作 ============
async function openConvertFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'] }],
      multiple: false,
    })
    if (!selected) return

    convertState.filePath = selected as string
    convertState.fileName = (selected as string).split(/[/\\]/).pop() || ''

    convertState.isLoadingInfo = true
    const info: AudioInfo = await invoke('get_audio_info', { path: convertState.filePath, useFfmpeg: useFfmpeg.value })
    convertState.audioInfo = info
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetConvertForm()
  } finally {
    convertState.isLoadingInfo = false
  }
}

async function convertAudio() {
  if (!convertState.filePath) {
    ElMessage.warning('请先选择音频文件')
    return
  }

  try {
    error.value = ''
    convertState.isProcessing = true
    convertState.progress = 0

    const unlisten = await listen<{ progress: number }>('audio-convert-progress', (event) => {
      convertState.progress = Math.round(event.payload.progress)
    })

    let outputPath: string | null = null
    if (!convertState.saveToSamePath) {
      const defaultName = convertState.fileName.replace(/\.[^.]+$/, '') + '_converted.' + convertState.outputFormat
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: '音频文件', extensions: [convertState.outputFormat] }],
      })
      if (!outputPath) {
        unlisten()
        convertState.isProcessing = false
        return
      }
    }

    const result: ConvertResult = await invoke('audio_convert', {
      path: convertState.filePath,
      options: {
        output_format: convertState.outputFormat,
        bitrate: convertState.bitrate,
        sample_rate: convertState.sampleRate,
        channels: convertState.channels,
        output_path: outputPath,
      },
    })

    unlisten()
    convertState.progress = 100
    ElMessage.success(`转换完成，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'audioTool',
      action: '格式转换',
      inputPreview: `${convertState.fileName} → ${convertState.outputFormat.toUpperCase()}`,
      outputPreview: result.output_path.split(/[/\\]/).pop() || '',
      inputFull: convertState.filePath,
      outputFull: result.output_path,
      options: {
        output_format: convertState.outputFormat,
        bitrate: convertState.bitrate,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '转换失败'
  } finally {
    convertState.isProcessing = false
  }
}

function resetConvertForm() {
  convertState.filePath = ''
  convertState.fileName = ''
  convertState.audioInfo = null
  convertState.outputFormat = 'mp3'
  convertState.bitrate = 192
  convertState.sampleRate = null
  convertState.channels = null
  convertState.saveToSamePath = true
  convertState.progress = 0
  error.value = ''
}

// ============ 音频压缩操作 ============
async function openCompressFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'] }],
      multiple: false,
    })
    if (!selected) return

    compressState.filePath = selected as string
    compressState.fileName = (selected as string).split(/[/\\]/).pop() || ''

    compressState.isLoadingInfo = true
    const info: AudioInfo = await invoke('get_audio_info', { path: compressState.filePath, useFfmpeg: useFfmpeg.value })
    compressState.audioInfo = info
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetCompressForm()
  } finally {
    compressState.isLoadingInfo = false
  }
}

async function compressAudio() {
  if (!compressState.filePath) {
    ElMessage.warning('请先选择音频文件')
    return
  }

  try {
    error.value = ''
    compressState.isProcessing = true
    compressState.progress = 0

    const unlisten = await listen<{ progress: number }>('audio-compress-progress', (event) => {
      compressState.progress = Math.round(event.payload.progress)
    })

    let outputPath: string | null = null
    if (!compressState.saveToSamePath) {
      const ext = compressState.audioInfo?.format || 'mp3'
      const defaultName = compressState.fileName.replace(/\.[^.]+$/, '') + '_compressed.' + ext
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: '音频文件', extensions: [ext] }],
      })
      if (!outputPath) {
        unlisten()
        compressState.isProcessing = false
        return
      }
    }

    const result: CompressResult = await invoke('audio_compress', {
      path: compressState.filePath,
      options: {
        mode: compressState.mode,
        bitrate: compressState.mode === 'bitrate' ? compressState.bitrate : null,
        quality: compressState.mode === 'quality' ? compressState.quality : null,
        sample_rate: compressState.sampleRate,
        output_path: outputPath,
      },
    })

    unlisten()
    compressState.progress = 100
    const saved = result.original_size - result.output_size
    ElMessage.success(`压缩完成，节省 ${(saved / 1024 / 1024).toFixed(2)} MB，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'audioTool',
      action: '压缩',
      inputPreview: `${compressState.fileName} (${formatBytes(result.original_size)})`,
      outputPreview: `${formatBytes(result.output_size)} (节省 ${(saved / 1024 / 1024).toFixed(2)} MB)`,
      inputFull: compressState.filePath,
      outputFull: result.output_path,
      options: {
        mode: compressState.mode,
        bitrate: compressState.bitrate,
        quality: compressState.quality,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '压缩失败'
  } finally {
    compressState.isProcessing = false
  }
}

function resetCompressForm() {
  compressState.filePath = ''
  compressState.fileName = ''
  compressState.audioInfo = null
  compressState.mode = 'bitrate'
  compressState.bitrate = 128
  compressState.quality = 'medium'
  compressState.sampleRate = null
  compressState.saveToSamePath = true
  compressState.progress = 0
  error.value = ''
}

// ============ 音频合并操作 ============
async function addMergeFiles() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'] }],
      multiple: true,
    })
    if (!selected || !Array.isArray(selected)) return

    for (const path of selected) {
      const name = path.split(/[/\\]/).pop() || ''
      // 获取音频时长
      let duration = 0
      try {
        const info: AudioInfo = await invoke('get_audio_info', { path, useFfmpeg: useFfmpeg.value })
        duration = info.duration
      } catch {
        // 忽略获取时长失败
      }
      mergeState.files.push({ path, name, duration })
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '添加文件失败'
  }
}

function clearMergeFiles() {
  mergeState.files = []
  mergeState.progress = 0
  error.value = ''
}

function removeMergeFile(index: number) {
  mergeState.files.splice(index, 1)
}

function onDragStart(e: DragEvent, index: number) {
  mergeState.dragIndex = index
  e.dataTransfer!.effectAllowed = 'move'
}

function onDragOver(e: DragEvent, _index: number) {
  e.dataTransfer!.dropEffect = 'move'
}

function onDrop(e: DragEvent, index: number) {
  e.preventDefault()
  const dragIndex = mergeState.dragIndex
  if (dragIndex === index || dragIndex === -1) return

  const item = mergeState.files.splice(dragIndex, 1)[0]
  mergeState.files.splice(index, 0, item)
  mergeState.dragIndex = -1
}

async function mergeAudio() {
  if (mergeState.files.length < 2) {
    ElMessage.warning('至少需要两个音频文件')
    return
  }

  try {
    error.value = ''
    mergeState.isProcessing = true
    mergeState.progress = 0

    const unlisten = await listen<{ progress: number }>('audio-merge-progress', (event) => {
      mergeState.progress = Math.round(event.payload.progress)
    })

    let outputPath: string | null = null
    if (!mergeState.saveToSamePath) {
      const firstName = mergeState.files[0].name.replace(/\.[^.]+$/, '')
      const defaultName = firstName + '_merged.' + mergeState.outputFormat
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: '音频文件', extensions: [mergeState.outputFormat] }],
      })
      if (!outputPath) {
        unlisten()
        mergeState.isProcessing = false
        return
      }
    }

    const result: MergeResult = await invoke('audio_merge', {
      options: {
        input_paths: mergeState.files.map(f => f.path),
        output_format: mergeState.outputFormat,
        bitrate: mergeState.bitrate,
        mode: mergeState.mode,
        output_path: outputPath,
      },
    })

    unlisten()
    mergeState.progress = 100
    ElMessage.success(`合并完成，总时长 ${formatDuration(result.duration)}，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'audioTool',
      action: '合并',
      inputPreview: `${mergeState.files.length} 个文件 → ${mergeState.outputFormat.toUpperCase()}`,
      outputPreview: `${formatDuration(result.duration)} | ${result.output_path.split(/[/\\]/).pop() || ''}`,
      inputFull: mergeState.files.map(f => f.path).join('\n'),
      outputFull: result.output_path,
      options: {
        file_count: mergeState.files.length,
        output_format: mergeState.outputFormat,
        bitrate: mergeState.bitrate,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '合并失败'
  } finally {
    mergeState.isProcessing = false
  }
}

function resetMergeForm() {
  mergeState.files = []
  mergeState.outputFormat = 'mp3'
  mergeState.bitrate = 192
  mergeState.mode = 'auto'
  mergeState.saveToSamePath = true
  mergeState.progress = 0
  mergeState.dragIndex = -1
  error.value = ''
}

// ============ 变速变调操作 ============
async function openSpeedFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'] }],
      multiple: false,
    })
    if (!selected) return

    speedState.filePath = selected as string
    speedState.fileName = (selected as string).split(/[/\\]/).pop() || ''

    speedState.isLoadingInfo = true
    const info: AudioInfo = await invoke('get_audio_info', { path: speedState.filePath, useFfmpeg: useFfmpeg.value })
    speedState.audioInfo = info
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetSpeedForm()
  } finally {
    speedState.isLoadingInfo = false
  }
}

async function changeSpeed() {
  if (!speedState.filePath) {
    ElMessage.warning('请先选择音频文件')
    return
  }

  if (speedState.speed < 0.5 || speedState.speed > 4.0) {
    ElMessage.warning('速度必须在 0.5x 到 4.0x 之间')
    return
  }

  try {
    error.value = ''
    speedState.isProcessing = true
    speedState.progress = 0

    const unlisten = await listen<{ progress: number }>('audio-speed-progress', (event) => {
      speedState.progress = Math.round(event.payload.progress)
    })

    let outputPath: string | null = null
    if (!speedState.saveToSamePath) {
      const defaultName = speedState.fileName.replace(/\.[^.]+$/, '') + `_${speedState.speed}x.` + speedState.outputFormat
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: '音频文件', extensions: [speedState.outputFormat] }],
      })
      if (!outputPath) {
        unlisten()
        speedState.isProcessing = false
        return
      }
    }

    const result: SpeedChangeResult = await invoke('audio_speed_change', {
      path: speedState.filePath,
      options: {
        speed: speedState.speed,
        keep_pitch: speedState.keepPitch,
        output_format: speedState.outputFormat,
        bitrate: speedState.bitrate,
        output_path: outputPath,
      },
    })

    unlisten()
    speedState.progress = 100
    ElMessage.success(`变速完成，新时长 ${formatDuration(result.duration)}，已保存到: ${result.output_path}`)
    
    // 添加历史记录
    store.addHistory({
      tool: 'audio',
      action: '变速变调',
      inputPreview: `${speedState.fileName} @ ${speedState.speed}x`,
      outputPreview: `${formatDuration(result.duration)} | ${result.output_path.split(/[/\\]/).pop() || ''}`,
      inputFull: speedState.filePath,
      outputFull: result.output_path,
      options: {
        speed: speedState.speed,
        keep_pitch: speedState.keepPitch,
        output_format: speedState.outputFormat,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '变速失败'
  } finally {
    speedState.isProcessing = false
  }
}

function resetSpeedForm() {
  speedState.filePath = ''
  speedState.fileName = ''
  speedState.audioInfo = null
  speedState.speed = 1.0
  speedState.keepPitch = true
  speedState.outputFormat = 'mp3'
  speedState.bitrate = 192
  speedState.saveToSamePath = true
  speedState.progress = 0
  error.value = ''
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

// ============ 响应式 ============
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  // 检测 ffmpeg
  try {
    useFfmpeg.value = await invoke('check_ffmpeg')
  } catch { /* 忽略 */ }
  ffmpegChecked.value = true

  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => drawWaveform())
    resizeObserver.observe(canvasRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  stopPreview()
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

onActivated(() => {
  // 从历史记录恢复（仅支持文字转语音的文字内容）
  const restore = store.pendingHistoryRestore
  if (!restore) return
  if (restore.tool !== 'audioTool') return
  if (restore.action === '文字转语音' && restore.input) {
    tsState.text = restore.input
    activeTab.value = 'tts'
    if (restore.options?.voice_name) tsState.voiceName = restore.options.voice_name
    if (restore.options?.engine) tsState.engine = restore.options.engine
    if (restore.options?.rate !== undefined) tsState.rate = restore.options.rate
    if (restore.options?.volume !== undefined) tsState.volume = restore.options.volume
    ElMessage.success('已恢复文字转语音的历史记录')
  }
  store.clearHistoryRestore()
})

watch([startTime, endTime], () => drawWaveform())

// ============ TTS 操作 ============

async function loadTtsVoices() {
  try {
    const voices: TtsVoice[] = await invoke('list_tts_voices')
    tsState.voices = voices
  } catch (e: any) {
    // 静默失败，语音列表为空时使用默认语音
    console.warn('加载语音列表失败:', e)
  }
}

function onEngineChange() {
  tsState.voiceName = ''
}

async function pasteTtsText() {
  try {
    const text = await navigator.clipboard.readText()
    if (text) {
      tsState.text = text
    }
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

async function generateTts() {
  if (!tsState.text.trim()) {
    ElMessage.warning('请输入要转换的文字')
    return
  }

  try {
    error.value = ''
    tsState.isProcessing = true
    tsState.progress = 0
    tsState.resultPath = ''
    tsState.resultSize = 0
    tsState.audioBase64 = ''

    const result: TtsResult = await invoke('tts_generate', {
      options: {
        text: tsState.text,
        voice_name: tsState.voiceName || null,
        rate: tsState.rate,
        volume: tsState.volume,
        output_path: null,
        engine: tsState.engine,
      },
    })

    tsState.progress = 100
    tsState.resultPath = result.output_path
    tsState.resultSize = result.output_size

    ElMessage.success('语音生成完成')
    
    // 添加历史记录
    store.addHistory({
      tool: 'audioTool',
      action: '文字转语音',
      inputPreview: `${tsState.text.slice(0, 30)}${tsState.text.length > 30 ? '...' : ''}`,
      outputPreview: `${tsState.voiceName || '默认'} | ${formatBytes(result.output_size)}`,
      inputFull: tsState.text,
      outputFull: result.output_path,
      options: {
        voice_name: tsState.voiceName,
        engine: tsState.engine,
        rate: tsState.rate,
        volume: tsState.volume,
      },
    })
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '生成失败'
  } finally {
    tsState.isProcessing = false
  }
}

async function previewTts() {
  if (!tsState.resultPath) return
  try {
    const base64: string = await invoke('read_file_base64', { filePath: tsState.resultPath })
    tsState.audioBase64 = base64
    tsState.showPreview = true
  } catch (e: any) {
    ElMessage.error('无法加载音频文件')
  }
}

const ttsAudioRef = ref<HTMLAudioElement | null>(null)

function stopTtsPreview() {
  if (ttsAudioRef.value) {
    ttsAudioRef.value.pause()
    ttsAudioRef.value.currentTime = 0
  }
}

async function locateTtsFile() {
  if (!tsState.resultPath) return
  try {
    await invoke('disk_locate_in_explorer', { path: tsState.resultPath })
  } catch (e: any) {
    ElMessage.error('无法打开文件所在位置')
  }
}

async function saveAsTtsFile() {
  if (!tsState.resultPath) return
  try {
    const filePath: string | null = await save({
      title: '另存为',
      defaultPath: 'tts_output.wav',
      filters: [{ name: 'WAV 音频', extensions: ['wav'] }],
    })
    if (!filePath) return

    // 通过后端复制文件到目标路径
    await invoke('copy_file', { from: tsState.resultPath, to: filePath })
    ElMessage.success('已保存到: ' + filePath)
  } catch (e: any) {
    ElMessage.error('保存失败: ' + (e?.message || e))
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
}

// 页面加载时获取语音列表
onMounted(() => {
  loadTtsVoices()
})
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.audio-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .audio-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.audio-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.audio-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.audio-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.audio-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.audio-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 页面特有样式 ===== */
.audio-file-info {
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

.waveform-container {
  position: relative;
  width: 100%;
  height: 200px;
  cursor: pointer;
}

.waveform-canvas {
  width: 100%;
  height: 100%;
  border-radius: 4px;
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

.waveform-labels {
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

.ffmpeg-link {
  color: var(--accent-cyan);
  margin-left: 4px;
}

.ffmpeg-link:hover {
  text-decoration: underline;
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

/* ===== 音频合并样式 ===== */
.merge-empty {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 20px;
}

.merge-file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.merge-file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: move;
  transition: all 0.2s;
}

.merge-file-item:hover {
  border-color: var(--accent-cyan);
}

.merge-file-item.dragging {
  opacity: 0.5;
}

.merge-file-index {
  color: var(--accent-cyan);
  font-weight: 600;
  min-width: 20px;
}

.merge-file-name {
  flex: 1;
  color: var(--text-primary);
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.merge-file-duration {
  color: var(--text-secondary);
  font-size: 12px;
}

.merge-total-info {
  margin-top: 12px;
  color: var(--accent-cyan);
  font-size: 13px;
}

/* ===== 变速变调样式 ===== */
.speed-info {
  margin-top: 8px;
  color: var(--accent-cyan);
  font-size: 13px;
}

/* ===== TTS 文字转语音样式 ===== */
.ts-char-count {
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 13px;
}

.ts-result-info {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 13px;
  align-items: flex-start;
}

.ts-result-label {
  color: var(--text-secondary);
  white-space: nowrap;
  min-width: 60px;
}

.ts-result-path {
  color: var(--text-primary);
  word-break: break-all;
}

.ts-preview-container {
  display: flex;
  justify-content: center;
  padding: 16px 0;
}

.ts-audio-player {
  width: 100%;
  max-width: 320px;
  outline: none;
}
</style>