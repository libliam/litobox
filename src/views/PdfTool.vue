<template>
  <div class="tool-container">
    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="pdf-tabs">
        <el-tab-pane label="PDF转图片" name="pdfToImages" />
        <el-tab-pane label="图片转PDF" name="imagesToPdf" />
        <el-tab-pane label="PDF文本提取" name="textExtract" />
        <el-tab-pane label="PDF转Markdown" name="pdfToMarkdown" />
        <el-tab-pane label="PDF合并/拆分" name="mergeSplit" />
        <el-tab-pane label="PDF压缩" name="compress" />
        <el-tab-pane label="提取图片" name="extractImages" />
      </el-tabs>
    </div>

    <!-- Tab 1: PDF转图片 -->
    <div v-if="activeTab === 'pdfToImages'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">PDF 输入</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>上传 PDF 文件，逐页导出为 PNG 图片</p>
                <p>DPI 越高，图片越清晰但速度越慢</p>
                <p>72 DPI: 快速预览 | 150 DPI: 标准 | 300 DPI: 高清</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerPdfInput">上传 PDF</el-button>
          <el-button v-if="pdfFile" size="small" @click="handleClearPdf">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="pdfInputRef"
          type="file"
          accept=".pdf"
          style="display: none"
          @change="handlePdfFileSelect"
        />
        <div v-if="pdfFile" class="file-info">
          <span class="file-name">{{ pdfFile.name }}</span>
          <span class="file-size">{{ formatFileSize(pdfFile.size) }}</span>
          <span v-if="pdfPageCount" class="file-pages">{{ pdfPageCount }} 页</span>
        </div>
        <div v-else class="upload-hint">点击「上传 PDF」选择文件</div>
      </div>
    </div>

    <div v-if="activeTab === 'pdfToImages'" class="tool-card">
      <div class="card-header">
        <span class="card-title">转换设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">DPI</div>
            <el-select v-model="dpi" size="small" style="width: 120px">
              <el-option label="72 (快速)" :value="72" />
              <el-option label="150 (标准)" :value="150" />
              <el-option label="300 (高清)" :value="300" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :disabled="!pdfFile" @click="handlePdfToImages">
                开始转换
              </el-button>
              <el-button size="small" :disabled="!imageBlobs.length" @click="handleDownloadAllImages">
                全部下载
              </el-button>
              <el-button
                size="small"
                type="success"
                :disabled="!imageBlobs.length || isOcrRunning"
                :loading="isOcrRunning"
                @click="handleOcrAll"
              >
                OCR 识别
              </el-button>
              <el-button
                size="small"
                :disabled="!imageBlobs.length"
                @click="handleJumpToOcr"
              >
                跳转到OCR
              </el-button>
            </div>
          </div>
        </div>
        <div v-if="imageBlobs.length" class="result-info">
          <span>共 {{ imageBlobs.length }} 页</span>
          <span>总大小: {{ formatFileSize(imageBlobs.reduce((sum, b) => sum + b.size, 0)) }}</span>
        </div>
        <div v-if="imageBlobs.length" class="image-preview-grid">
          <div v-for="(blob, idx) in imageBlobs" :key="idx" class="image-preview-item">
            <img :src="getImageUrl(blob)" :alt="`第 ${idx + 1} 页`" />
            <div class="image-label">第 {{ idx + 1 }} 页</div>
            <el-button size="small" @click="handleDownloadSingleImage(blob, idx + 1)">下载</el-button>
          </div>
        </div>

        <!-- OCR 结果 -->
        <div v-if="ocrResults.length > 0" class="ocr-result-section">
          <el-divider />
          <div class="ocr-result-header">
            <span class="ocr-result-title">OCR 识别结果 ({{ ocrResults.length }} 页)</span>
            <div class="ocr-actions">
              <el-button size="small" @click="handleCopyOcrResult">复制全部</el-button>
              <el-button size="small" @click="handleExportOcrResult">导出 .txt</el-button>
              <el-button size="small" type="success" :disabled="!ocrEditableText" @click="handleSaveOcrEdit">保存修改</el-button>
              <el-button size="small" @click="handleClearOcrResult">清除</el-button>
            </div>
          </div>
          <el-input
            v-model="ocrEditableText"
            type="textarea"
            :rows="10"
            class="ocr-textarea"
          />
        </div>

        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 2: 图片转PDF -->
    <div v-if="activeTab === 'imagesToPdf'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerImageInput">添加图片</el-button>
          <el-button v-if="imageFiles.length" size="small" @click="handleClearImages">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="imageInputRef"
          type="file"
          accept="image/png,image/jpeg"
          multiple
          style="display: none"
          @change="handleImageFileSelect"
        />
        <div v-if="imageFiles.length" class="image-list">
          <div v-for="(file, idx) in imageFiles" :key="idx" class="image-list-item">
            <span class="image-list-index">{{ idx + 1 }}</span>
            <span class="image-list-name">{{ file.name }}</span>
            <span class="image-list-size">{{ formatFileSize(file.size) }}</span>
            <el-button size="small" type="danger" link @click="handleRemoveImage(idx)">删除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">点击「添加图片」选择图片（支持多选）</div>
      </div>
    </div>

    <div v-if="activeTab === 'imagesToPdf'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">页面尺寸</div>
            <el-select v-model="imageToPdfOptions.pageSize" size="small" style="width: 120px">
              <el-option label="跟随图片" value="auto" />
              <el-option label="A4" value="a4" />
              <el-option label="A3" value="a3" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">方向</div>
            <el-select v-model="imageToPdfOptions.orientation" size="small" style="width: 120px">
              <el-option label="自动" value="auto" />
              <el-option label="纵向" value="portrait" />
              <el-option label="横向" value="landscape" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :disabled="!imageFiles.length" @click="handleImagesToPdf">
                生成 PDF
              </el-button>
            </div>
          </div>
        </div>
        <div v-if="generatedPdfBlob" class="result-info">
          <span>PDF 大小: {{ formatFileSize(generatedPdfBlob.size) }}</span>
          <el-button size="small" @click="handleDownloadGeneratedPdf">下载 PDF</el-button>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 3: PDF文本提取 -->
    <div v-if="activeTab === 'textExtract'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerExtractInput">上传 PDF</el-button>
          <el-button v-if="extractPdfFile" size="small" @click="handleClearExtractPdf">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="extractInputRef"
          type="file"
          accept=".pdf"
          style="display: none"
          @change="handleExtractPdfSelect"
        />
        <div v-if="extractPdfFile" class="file-info">
          <span class="file-name">{{ extractPdfFile.name }}</span>
          <span class="file-size">{{ formatFileSize(extractPdfFile.size) }}</span>
        </div>
        <div v-else class="upload-hint">点击「上传 PDF」选择文件</div>
      </div>
    </div>

    <div v-if="activeTab === 'textExtract'" class="tool-card">
      <div class="card-header">
        <span class="card-title">提取结果</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!extractedText" @click="handleCopyExtractedText">复制</el-button>
          <el-button size="small" :disabled="!extractedText" @click="handleDownloadExtractedText">下载 .txt</el-button>
          <el-button size="small" type="success" :disabled="!extractedText" @click="handleSaveExtractedEdit">保存修改</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" size="small" :disabled="!extractPdfFile" @click="handleExtractText">
              提取文本
            </el-button>
          </div>
        </div>
        <div v-if="extractedText" class="text-info">
          <span>字符数: {{ extractedText.length }}</span>
        </div>
        <el-input
          v-model="extractedText"
          type="textarea"
          :rows="12"
          placeholder="提取的文本将在此处显示..."
        />
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 5: PDF转Markdown -->
    <div v-if="activeTab === 'pdfToMarkdown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerMarkdownInput">上传 PDF</el-button>
          <el-button v-if="markdownPdfFile" size="small" @click="handleClearMarkdownPdf">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="markdownInputRef"
          type="file"
          accept=".pdf"
          style="display: none"
          @change="handleMarkdownPdfSelect"
        />
        <div v-if="markdownPdfFile" class="file-info">
          <span class="file-name">{{ markdownPdfFile.name }}</span>
          <span class="file-size">{{ formatFileSize(markdownPdfFile.size) }}</span>
          <span v-if="markdownPageCount" class="file-pages">{{ markdownPageCount }} 页</span>
        </div>
        <div v-else class="upload-hint">点击「上传 PDF」选择文件</div>
      </div>
    </div>

    <div v-if="activeTab === 'pdfToMarkdown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">转换设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">转换模式</div>
            <el-select v-model="markdownMode" size="small" style="width: 160px">
              <el-option label="自动检测" value="auto" />
              <el-option label="文本提取（快速）" value="text" />
              <el-option label="OCR 识别（精确）" value="ocr" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">OCR DPI</div>
            <el-select v-model="markdownDpi" size="small" style="width: 120px">
              <el-option label="150 (标准)" :value="150" />
              <el-option label="200 (清晰)" :value="200" />
              <el-option label="300 (高清)" :value="300" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!markdownPdfFile || isMarkdownConverting"
                :loading="isMarkdownConverting"
                @click="handlePdfToMarkdown"
              >
                开始转换
              </el-button>
            </div>
          </div>
        </div>
        <div v-if="markdownMode === 'ocr'" class="ocr-hint">
          <el-icon><Warning /></el-icon>
          <span>OCR 模式会逐页识别，速度较慢，适合扫描版 PDF</span>
        </div>
        <div v-if="markdownError" class="error-message">{{ markdownError }}</div>
      </div>
    </div>

    <div v-if="activeTab === 'pdfToMarkdown' && markdownOutput" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Markdown 结果</span>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleCopyMarkdownOutput">复制</el-button>
          <el-button size="small" @click="handleExportMarkdownOutput">导出 .md</el-button>
          <el-button size="small" type="success" @click="handleSaveMarkdownEdit">保存修改</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="markdown-output-grid">
          <div class="markdown-source-panel">
            <div class="panel-label">源码（可编辑）</div>
            <el-input
              v-model="markdownOutput"
              type="textarea"
              :rows="20"
              resize="vertical"
              class="markdown-textarea"
            />
          </div>
          <div class="markdown-preview-panel">
            <div class="panel-label">预览</div>
            <div class="markdown-preview" v-html="markdownHtmlPreview"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 4: PDF合并/拆分 -->
    <div v-if="activeTab === 'mergeSplit'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 文件列表</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerMergeInput">添加 PDF</el-button>
          <el-button v-if="mergePdfFiles.length" size="small" @click="handleClearMergePdfs">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="mergeInputRef"
          type="file"
          accept=".pdf"
          multiple
          style="display: none"
          @change="handleMergePdfSelect"
        />
        <div v-if="mergePdfFiles.length" class="merge-file-list">
          <div v-for="(file, idx) in mergePdfFiles" :key="idx" class="merge-file-item">
            <span class="merge-file-index">{{ idx + 1 }}</span>
            <span class="merge-file-name">{{ file.name }}</span>
            <span class="merge-file-size">{{ formatFileSize(file.size) }}</span>
            <el-input
              v-model="mergePageRanges[idx]"
              size="small"
              placeholder="页码范围 (如: 1-3,5)"
              style="width: 160px"
            />
            <el-button size="small" type="danger" link @click="handleRemoveMergePdf(idx)">删除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">点击「添加 PDF」选择文件（支持多选）</div>
      </div>
    </div>

    <div v-if="activeTab === 'mergeSplit'" class="tool-card">
      <div class="card-header">
        <span class="card-title">快捷操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">快捷</div>
            <div class="group-buttons">
              <el-button size="small" :disabled="!mergePdfFiles.length" @click="handleQuickMerge('all')">
                合并所有
              </el-button>
              <el-button size="small" :disabled="!mergePdfFiles.length" @click="handleQuickMerge('odd')">
                提取奇数页
              </el-button>
              <el-button size="small" :disabled="!mergePdfFiles.length" @click="handleQuickMerge('even')">
                提取偶数页
              </el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">自定义页码</div>
            <el-input
              v-model="customPageRange"
              size="small"
              placeholder="如: 1-3,5,8-10"
              style="width: 200px"
              clearable
            />
            <el-button type="primary" size="small" :disabled="!mergePdfFiles.length || !customPageRange" @click="handleCustomMerge">
              生成 PDF
            </el-button>
          </div>
        </div>
        <div class="page-range-hint">
          支持格式: <code>1-3,5,8-10</code>（连续范围用 <code>-</code>，多个用 <code>,</code> 分隔）
          <span v-if="mergePdfFiles.length === 1 && singleFilePageCount > 0">
            · 当前文件共 {{ singleFilePageCount }} 页
          </span>
        </div>
        <div v-if="mergedPdfBlob" class="result-info">
          <span>PDF 大小: {{ formatFileSize(mergedPdfBlob.size) }}</span>
          <el-button size="small" @click="handleDownloadMergedPdf">下载 PDF</el-button>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 6: PDF压缩 -->
    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">PDF 输入</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>支持拖入多个 PDF 文件批量压缩</p>
                <p>单文件最大 100MB</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerCompressInput">添加 PDF</el-button>
          <el-button v-if="compressFiles.length" size="small" @click="handleClearCompressFiles">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="compressInputRef"
          type="file"
          accept=".pdf"
          multiple
          style="display: none"
          @change="handleCompressFileSelect"
        />
        <div
          class="compress-drop-zone"
          @dragover.prevent="isDragOver = true"
          @dragenter.prevent="handleCompressDragEnter"
          @dragleave.prevent="handleCompressDragLeave"
          @drop.prevent="handleCompressDrop"
          :class="{ 'drag-over': isDragOver }"
        >
          <div v-if="compressFiles.length" class="compress-file-list">
            <div v-for="(file, idx) in compressFiles" :key="idx" class="compress-file-item">
              <span class="file-index">{{ idx + 1 }}</span>
              <span class="file-name">{{ file.name }}</span>
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
              <el-button size="small" type="danger" link @click="handleRemoveCompressFile(idx)">移除</el-button>
            </div>
          </div>
          <div v-else class="upload-hint">点击「添加 PDF」或拖入 PDF 文件</div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header">
        <span class="card-title">压缩设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">压缩级别</div>
            <el-radio-group v-model="compressLevel" size="small">
              <el-radio-button :value="1">快速压缩</el-radio-button>
              <el-radio-button :value="2">标准压缩</el-radio-button>
              <el-radio-button :value="3">极限压缩</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!compressFiles.length || isCompressing"
                :loading="isCompressing"
                @click="handleCompress"
              >
                开始压缩
              </el-button>
            </div>
          </div>
        </div>
        <div class="compress-level-hint">
          {{ compressLevelHint }}
        </div>
        <div class="compress-custom-toggle">
          <el-switch v-model="useCustomSettings" size="small" />
          <span class="custom-toggle-label">自定义参数</span>
        </div>
        <div v-if="useCustomSettings" class="compress-custom-params">
          <div class="custom-param">
            <span class="param-label">图片 DPI</span>
            <el-slider
              v-model="customDpi"
              :min="36"
              :max="300"
              :step="6"
              size="small"
              show-input
              :format-tooltip="(v: number) => v + ' DPI'"
              style="width: 220px"
            />
          </div>
          <div class="custom-param">
            <span class="param-label">JPEG 质量</span>
            <el-slider
              v-model="customQuality"
              :min="10"
              :max="100"
              :step="5"
              size="small"
              show-input
              :format-tooltip="(v: number) => v + '%'"
              style="width: 220px"
            />
          </div>
        </div>
        <div v-if="gsAvailable" class="gs-hint">
          <span>已检测到 Ghostscript，「极限压缩」将获得更佳效果</span>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'compress' && compressResults.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">压缩结果</span>
        <div class="card-actions">
          <el-button size="small" @click="handleSaveAllCompressed">全部保存</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="compressResults" stripe size="small" class="compress-table">
          <el-table-column prop="fileName" label="文件" min-width="200" />
          <el-table-column label="原始大小" width="110">
            <template #default="{ row }">
              <span>{{ formatFileSize(row.originalSize) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="压缩后" width="110">
            <template #default="{ row }">
              <span>{{ formatFileSize(row.compressedSize) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="压缩率" width="90">
            <template #default="{ row }">
              <span :class="row.ratio > 0 ? 'ratio-positive' : 'ratio-negative'">
                {{ row.ratio > 0 ? `-${row.ratio}%` : `+${Math.abs(row.ratio)}%` }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="90">
            <template #default="{ $index }">
              <el-button size="small" @click="handleSaveSingleCompressed($index)">保存</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div class="compress-summary">
          <span>合计：{{ formatFileSize(totalOriginalSize) }} → {{ formatFileSize(totalCompressedSize) }}，</span>
          <span :class="totalRatio > 0 ? 'ratio-positive' : 'ratio-negative'">
            缩小 {{ totalRatio }}%
          </span>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'compress' && compressError" class="error-message">{{ compressError }}</div>

    <!-- Tab 7: 提取图片（输入卡片） -->
    <div v-if="activeTab === 'extractImages'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">PDF 输入</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>从 PDF 中提取<strong>原始内嵌图片</strong>（XObject 资源）</p>
                <p>与「PDF 转图片」的区别：<strong>不做整页栅格化</strong>，只提取真正嵌入 PDF 的位图资源</p>
                <p>支持：JPEG(DCTDecode) / PNG(FlateDecode) / JPEG2000 / TIFF(CCITT) / Raw</p>
                <p>跨页复用的图片资源仅提取一次（自动去重）</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerExtractImgInput">上传 PDF</el-button>
          <el-button v-if="extractImgFile" size="small" @click="handleClearExtractImgFile">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="extractImgInputRef"
          type="file"
          accept=".pdf"
          style="display: none"
          @change="handleExtractImgFileSelect"
        />
        <div v-if="extractImgFile" class="file-info">
          <span class="file-name">{{ extractImgFile.name }}</span>
          <span class="file-size">{{ formatFileSize(extractImgFile.size) }}</span>
          <span v-if="extractImgPageCount" class="file-pages">{{ extractImgPageCount }} 页</span>
        </div>
        <div v-else class="upload-hint">点击「上传 PDF」选择文件</div>
      </div>
    </div>

    <!-- Tab 7: 提取图片（操作卡片） -->
    <div v-if="activeTab === 'extractImages'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片提取</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!extractedImages.length" @click="handleSaveAllExtractedImagesZip">
            📦 打包下载 ZIP
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!extractImgFile || isExtractingImages"
                :loading="isExtractingImages"
                @click="handleExtractImages"
              >
                扫描并提取
              </el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">说明</div>
            <span class="group-label">
              支持 JPEG / PNG / JPEG 2000 / TIFF Fax / 原始字节，部分罕见格式仅保存原始数据，浏览器无法预览
            </span>
          </div>
        </div>
        <div v-if="extractedImages.length" class="result-info">
          <span>共找到 {{ extractedImages.length }} 张内嵌图片</span>
          <span>总大小: {{ formatFileSize(extractedImages.reduce((s, i) => s + i.size, 0)) }}</span>
          <span>JPEG {{ imgFormatCounts.jpeg }} 张 · PNG {{ imgFormatCounts.png }} 张 · 其他 {{ imgFormatCounts.other }} 张</span>
        </div>
        <div v-if="extractedImages.length" class="extract-preview-grid">
          <div v-for="(img, idx) in extractedImages" :key="idx" class="extract-preview-item">
            <div class="extract-preview-thumb">
              <img :src="img.previewDataUrl" :alt="img.xObjectName" loading="lazy" />
            </div>
            <div class="extract-preview-meta">
              <div class="preview-title">#{{ idx + 1 }} · P{{ img.pageIndex }}</div>
              <div class="preview-dims">{{ img.width }} × {{ img.height }}</div>
              <div class="preview-tags">
                <span class="tag tag-format" :class="'tag-' + img.format">{{ img.format.toUpperCase() }}</span>
                <span class="tag">{{ formatFileSize(img.size) }}</span>
                <span v-if="img.colorSpace" class="tag tag-cs">{{ img.colorSpace }}</span>
              </div>
              <div class="preview-xobj" :title="img.xObjectName">资源: {{ img.xObjectName }}</div>
            </div>
            <div class="extract-preview-actions">
              <el-button size="small" @click="handleSaveSingleExtractedImage(img, idx + 1)">保存</el-button>
            </div>
          </div>
        </div>
        <div v-if="!extractedImages.length && extractImgScanDone" class="empty-hint">
          本 PDF 未发现内嵌图片资源（可能是纯文本 PDF，或图片以其他方式嵌入）
        </div>
        <div v-if="extractImagesError" class="error-message">{{ extractImagesError }}</div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElLoading } from 'element-plus'
import { QuestionFilled, Warning } from '@element-plus/icons-vue'
import {
  pdfToImages,
  extractPdfText,
  imagesToPdf,
  mergePdf,
  loadPdfDocument,
  saveFileWithDialog,
  formatFileSize,
  extractEmbeddedImages,
  type ImageToPdfOptions,
  type ExtractedImage
} from '@/utils/pdfUtils'
import { recognizeImage, recognizeMarkdown } from '@/utils/ocrUtils'
import { useToolboxStore } from '@/store'
import { invoke } from '@tauri-apps/api/core'

const store = useToolboxStore()
const activeTab = ref('pdfToImages')
const error = ref('')

// ============ Tab 6: PDF压缩 ============
const compressInputRef = ref<HTMLInputElement | null>(null)
const compressFiles = ref<File[]>([])
const compressLevel = ref(2)
const useCustomSettings = ref(false)
const customDpi = ref(72)
const customQuality = ref(50)
const isCompressing = ref(false)
const isDragOver = ref(false)
const compressDragCounter = ref(0)
const compressError = ref('')
const gsAvailable = ref(false)

interface CompressResultItem {
  fileName: string
  originalSize: number
  compressedSize: number
  ratio: number
  outputPath: string
}

const compressResults = ref<CompressResultItem[]>([])

const compressLevelHint = computed(() => {
  switch (compressLevel.value) {
    case 1: return '快速压缩：图片150DPI/85%质量，清除XMP元数据，速度最快'
    case 2: return '标准压缩：图片150DPI/70%质量，清除全部元数据，平衡体积与质量'
    case 3: return '极限压缩：图片72DPI/50%质量，清除全部元数据，最小体积（有Ghostscript效果更佳）'
    default: return ''
  }
})

const totalOriginalSize = computed(() =>
  compressResults.value.reduce((sum, r) => sum + r.originalSize, 0)
)

const totalCompressedSize = computed(() =>
  compressResults.value.reduce((sum, r) => sum + r.compressedSize, 0)
)

const totalRatio = computed(() => {
  const orig = totalOriginalSize.value
  const comp = totalCompressedSize.value
  if (orig === 0) return 0
  return Math.round((1 - comp / orig) * 100)
})

const triggerCompressInput = () => compressInputRef.value?.click()

const handleCompressFileSelect = (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files) return
  addCompressFiles(Array.from(files))
  input.value = ''
}

const handleCompressDragEnter = () => {
  compressDragCounter.value++
  isDragOver.value = true
}

const handleCompressDragLeave = () => {
  compressDragCounter.value--
  if (compressDragCounter.value <= 0) {
    compressDragCounter.value = 0
    isDragOver.value = false
  }
}

const handleCompressDrop = (e: DragEvent) => {
  isDragOver.value = false
  compressDragCounter.value = 0
  const files = e.dataTransfer?.files
  if (!files) return
  addCompressFiles(Array.from(files))
}

const addCompressFiles = (files: File[]) => {
  compressError.value = ''
  const pdfFiles = files.filter(f => f.type === 'application/pdf' || f.name.toLowerCase().endsWith('.pdf'))
  if (pdfFiles.length === 0) {
    compressError.value = '请选择 PDF 文件'
    return
  }
  const maxSize = 100 * 1024 * 1024
  const oversize = pdfFiles.find(f => f.size > maxSize)
  if (oversize) {
    compressError.value = `文件 "${oversize.name}" 超过 100MB 限制`
    return
  }
  compressFiles.value = [...compressFiles.value, ...pdfFiles]
  compressResults.value = []
}

const handleRemoveCompressFile = (idx: number) => {
  compressFiles.value.splice(idx, 1)
  compressResults.value = []
}

const handleClearCompressFiles = () => {
  compressFiles.value = []
  compressResults.value = []
  compressError.value = ''
  if (compressInputRef.value) compressInputRef.value.value = ''
}

/**
 * 分块 base64 编码，避免大文件栈溢出
 */
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  const chunkSize = 0x8000 // 32KB
  const chunks: string[] = []
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.subarray(i, i + chunkSize)
    chunks.push(String.fromCharCode(...chunk))
  }
  return btoa(chunks.join(''))
}

const handleCompress = async () => {
  if (!compressFiles.value.length) return
  compressError.value = ''
  compressResults.value = []
  isCompressing.value = true

  try {
    for (let i = 0; i < compressFiles.value.length; i++) {
      const file = compressFiles.value[i]
      const buffer = await file.arrayBuffer()
      const base64 = arrayBufferToBase64(buffer)
      const tempPath: string = await invoke('save_temp_file', { data: base64, filename: file.name })

      const result: { output_path: string; original_size: number; compressed_size: number } =
        await invoke('compress_pdf', {
          filePath: tempPath,
          level: compressLevel.value,
          gsAvailable: gsAvailable.value,
          targetDpi: useCustomSettings.value ? customDpi.value : null,
          jpegQuality: useCustomSettings.value ? customQuality.value : null,
        })

      const originalSize = result.original_size
      const compressedSize = result.compressed_size
      const ratio = originalSize > 0
        ? Math.round((1 - compressedSize / originalSize) * 100)
        : 0

      compressResults.value.push({
        fileName: file.name,
        originalSize,
        compressedSize,
        ratio,
        outputPath: result.output_path,
      })
    }
    ElMessage.success(`压缩完成，共 ${compressResults.value.length} 个文件`)
    store.addHistory({
      tool: 'pdf',
      action: `PDF压缩 (${compressLevel.value === 1 ? '快速' : compressLevel.value === 2 ? '标准' : '极限'})`,
      inputPreview: `${compressFiles.value.length} 个文件`,
      outputPreview: `缩小 ${totalRatio.value}%`,
      inputFull: compressFiles.value.map(f => f.name).join('\n'),
      outputFull: `${formatFileSize(totalOriginalSize.value)} → ${formatFileSize(totalCompressedSize.value)}，缩小 ${totalRatio.value}%`,
    })
  } catch (e: any) {
    compressError.value = typeof e === 'string' ? e : (e.message || '压缩失败')
  } finally {
    isCompressing.value = false
  }
}

const handleSaveSingleCompressed = async (idx: number) => {
  const result = compressResults.value[idx]
  if (!result) return
  try {
    const base64: string = await invoke('read_file_base64', { filePath: result.outputPath })
    const originalName = result.fileName
    const baseName = originalName.replace(/\.pdf$/i, '')
    await saveFileWithDialog(
      new Blob([Uint8Array.from(atob(base64), c => c.charCodeAt(0))], { type: 'application/pdf' }),
      `${baseName}_compressed.pdf`,
      'pdf'
    )
    ElMessage.success('已保存')
  } catch (e: any) {
    ElMessage.error(typeof e === 'string' ? e : '保存失败')
  }
}

const handleSaveAllCompressed = async () => {
  for (let i = 0; i < compressResults.value.length; i++) {
    await handleSaveSingleCompressed(i)
  }
}

// ============ Tab 7: 提取图片 ============
const extractImgInputRef = ref<HTMLInputElement | null>(null)
const extractImgFile = ref<File | null>(null)
const extractImgPageCount = ref(0)
const extractedImages = ref<ExtractedImage[]>([])
const isExtractingImages = ref(false)
const extractImagesError = ref('')
const extractImgScanDone = ref(false)

const imgFormatCounts = computed(() => {
  const counts = { jpeg: 0, png: 0, other: 0 }
  for (const img of extractedImages.value) {
    if (img.format === 'jpeg') counts.jpeg++
    else if (img.format === 'png') counts.png++
    else counts.other++
  }
  return counts
})

function extensionFor(img: ExtractedImage): string {
  switch (img.format) {
    case 'jpeg': return 'jpg'
    case 'png':  return 'png'
    case 'jp2':  return 'jp2'
    case 'tiff': return 'tiff'
    default:     return 'bin'
  }
}

function sanitizeFilename(name: string): string {
  // ponytail: 去掉 Windows 非法字符，保留中英文、数字、下划线、连字符、点
  return String(name).replace(/[\\/:*?"<>|]+/g, '_').slice(0, 80) || 'unnamed'
}

const triggerExtractImgInput = () => extractImgInputRef.value?.click()

const handleExtractImgFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  extractImagesError.value = ''
  extractedImages.value = []
  extractImgScanDone.value = false
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    extractImagesError.value = 'PDF 过大，建议小于 100MB'
    return
  }
  extractImgFile.value = file
  try {
    const buffer = await file.arrayBuffer()
    const doc = await loadPdfDocument(new Uint8Array(buffer))
    extractImgPageCount.value = doc.numPages
  } catch (e: any) {
    extractImagesError.value = e.message || 'PDF 加载失败'
    extractImgPageCount.value = 0
  }
  input.value = ''
}

const handleClearExtractImgFile = () => {
  extractImgFile.value = null
  extractImgPageCount.value = 0
  extractedImages.value = []
  extractImagesError.value = ''
  extractImgScanDone.value = false
  if (extractImgInputRef.value) extractImgInputRef.value.value = ''
}

const handleExtractImages = async () => {
  if (!extractImgFile.value) return
  extractImagesError.value = ''
  extractImgScanDone.value = false
  isExtractingImages.value = true
  const loading = ElLoading.service({
    lock: true,
    text: '正在扫描 PDF 内嵌图片...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    const imgs = await extractEmbeddedImages(extractImgFile.value)
    extractedImages.value = imgs
    extractImgScanDone.value = true
    if (imgs.length === 0) {
      ElMessage.warning('未发现内嵌图片资源')
    } else {
      ElMessage.success(`提取完成，共 ${imgs.length} 张图片`)
    }
    const jpeg = imgFormatCounts.value.jpeg
    const png = imgFormatCounts.value.png
    const other = imgs.length - jpeg - png
    store.addHistory({
      tool: 'pdf',
      action: 'PDF提取内嵌图片',
      inputPreview: extractImgFile.value.name.slice(0, 50),
      outputPreview: `${imgs.length} 张 (JPEG ${jpeg}, PNG ${png}, 其他 ${other})`,
      inputFull: extractImgFile.value.name,
      outputFull: extractedImages.value.map(i =>
        `#${i.xObjectName}  P${i.pageIndex}  ${i.width}×${i.height}  ${i.format.toUpperCase()}  ${formatFileSize(i.size)}`
      ).join('\n')
    })
  } catch (e: any) {
    extractImagesError.value = e.message || '提取失败'
  } finally {
    isExtractingImages.value = false
    loading.close()
  }
}

const handleSaveSingleExtractedImage = async (img: ExtractedImage, seqNo: number) => {
  try {
    const ext = extensionFor(img)
    const baseName = extractImgFile.value?.name.replace(/\.pdf$/i, '') || 'pdf'
    // 命名：文件基础名 + 页码 + 序号 + 原始资源名 + 扩展名
    const fileName = sanitizeFilename(
      `${baseName}_p${img.pageIndex}_${seqNo.toString().padStart(3, '0')}_${img.xObjectName}.${ext}`
    )
    await saveFileWithDialog(img.blob, fileName, ext === 'bin' ? 'zip' : ext)
    ElMessage.success('已保存')
  } catch (e: any) {
    ElMessage.error(typeof e === 'string' ? e : '保存失败')
  }
}

const handleSaveAllExtractedImagesZip = async () => {
  const imgs = extractedImages.value
  if (!imgs.length) return
  try {
    // 动态 import 避免无此功能的页面也被强依赖 jszip
    const JSZipModule = await import('jszip')
    const JSZip = JSZipModule.default || JSZipModule
    const zip = new JSZip()
    const baseName = (extractImgFile.value?.name.replace(/\.pdf$/i, '') || 'pdf-images')
    const dirName = sanitizeFilename(baseName)

    for (let idx = 0; idx < imgs.length; idx++) {
      const img = imgs[idx]
      const ext = extensionFor(img)
      const seqNo = (idx + 1).toString().padStart(3, '0')
      const fileName = sanitizeFilename(
        `p${img.pageIndex}_${seqNo}_${img.xObjectName}.${ext}`
      )
      zip.file(`${dirName}/${fileName}`, img.blob)
    }

    // 附加一份清单 manifest.csv（中文 GBK 兼容性不好，用 UTF-8 + BOM，Excel 可识别）
    const manifestHeader = ['序号', '页码', '资源名', '格式', '宽度', '高度', '大小(字节)', '大小(可读)', '颜色空间', '过滤器']
    const manifestRows = imgs.map((img, i) => [
      String(i + 1),
      String(img.pageIndex),
      `"${img.xObjectName.replace(/"/g, '""')}"`,
      img.format.toUpperCase(),
      String(img.width),
      String(img.height),
      String(img.size),
      formatFileSize(img.size),
      img.colorSpace,
      img.primaryFilter
    ])
    const manifestCsv = '\uFEFF' +
      [manifestHeader.join(','), ...manifestRows.map(r => r.join(','))].join('\r\n')
    zip.file(`${dirName}/_manifest.csv`, manifestCsv)

    const zipBlob = await zip.generateAsync({ type: 'blob', compression: 'STORE' })

    const now = new Date()
    const pad = (n: number) => n.toString().padStart(2, '0')
    const ts = `${now.getFullYear()}-${pad(now.getMonth()+1)}-${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`
    const zipFileName = sanitizeFilename(`${dirName}_images_${ts}.zip`)
    await saveFileWithDialog(zipBlob, zipFileName, 'zip')
    ElMessage.success(`已打包 ${imgs.length} 张图片`)
  } catch (e: any) {
    ElMessage.error('打包失败: ' + (typeof e === 'string' ? e : (e.message || String(e))))
  }
}

// 检测 Ghostscript
onMounted(async () => {
  try {
    gsAvailable.value = await invoke('detect_ghostscript')
  } catch {
    gsAvailable.value = false
  }
})

// ============ OCR 识别 ============
const ocrResults = ref<string[]>([])
const isOcrRunning = ref(false)
const ocrEditableText = ref('')
const ocrFullText = computed(() =>
  ocrResults.value.map((text, idx) => `--- 第 ${idx + 1} 页 ---\n${text}`).join('\n\n')
)

// ============ Tab 1: PDF转图片 ============
const pdfInputRef = ref<HTMLInputElement | null>(null)
const pdfFile = ref<File | null>(null)
const pdfPageCount = ref(0)
const dpi = ref(150)
const imageBlobs = ref<Blob[]>([])

const triggerPdfInput = () => pdfInputRef.value?.click()

const handlePdfFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await loadPdfFile(file)
  input.value = ''
}

const loadPdfFile = async (file: File) => {
  error.value = ''
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    error.value = 'PDF 过大，建议小于 100MB'
    return
  }
  pdfFile.value = file
  imageBlobs.value = []

  try {
    const buffer = await file.arrayBuffer()
    const doc = await loadPdfDocument(new Uint8Array(buffer))
    pdfPageCount.value = doc.numPages
  } catch (e: any) {
    error.value = e.message || 'PDF 加载失败'
  }
}

const handleClearPdf = () => {
  pdfFile.value = null
  pdfPageCount.value = 0
  imageBlobs.value = []
  error.value = ''
  if (pdfInputRef.value) pdfInputRef.value.value = ''
}

const handlePdfToImages = async () => {
  if (!pdfFile.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在转换 PDF（${pdfPageCount.value} 页），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    imageBlobs.value = await pdfToImages(pdfFile.value, dpi.value)
    ElMessage.success(`转换完成，共 ${imageBlobs.value.length} 页`)
    store.addHistory({
      tool: 'pdf',
      action: `PDF转图片 (${dpi.value}DPI)`,
      inputPreview: pdfFile.value.name.slice(0, 50),
      outputPreview: `${imageBlobs.value.length} 页`,
      inputFull: pdfFile.value.name,
      outputFull: `${imageBlobs.value.length} 页`,
    })
  } catch (e: any) {
    error.value = e.message || '转换失败'
  } finally {
    loading.close()
  }
}

const getImageUrl = (blob: Blob) => URL.createObjectURL(blob)

const handleDownloadSingleImage = async (blob: Blob, pageNum: number) => {
  await saveFileWithDialog(blob, `page_${pageNum}.png`, 'png')
}

const handleDownloadAllImages = async () => {
  for (let idx = 0; idx < imageBlobs.value.length; idx++) {
    const blob = imageBlobs.value[idx]
    await saveFileWithDialog(blob, `page_${idx + 1}.png`, 'png')
  }
}

// ============ OCR 识别 ============
const handleOcrAll = async () => {
  if (imageBlobs.value.length === 0) return
  error.value = ''
  ocrResults.value = []
  isOcrRunning.value = true

  const loading = ElLoading.service({
    lock: true,
    text: `正在 OCR 识别 ${imageBlobs.value.length} 页...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })

  try {
    for (let idx = 0; idx < imageBlobs.value.length; idx++) {
      const blob = imageBlobs.value[idx]
      const blobFile = new File([blob], `page_${idx + 1}.png`, { type: 'image/png' })
      const text = await recognizeImage(blobFile)
      ocrResults.value.push(text)
    }
    ocrEditableText.value = ocrFullText.value
    ElMessage.success(`OCR 识别完成，共 ${ocrResults.value.length} 页`)
    store.addHistory({
      tool: 'pdf',
      action: 'PDF转图片+OCR',
      inputPreview: pdfFile.value?.name.slice(0, 50) || '',
      outputPreview: ocrEditableText.value.slice(0, 50),
      inputFull: pdfFile.value?.name || '',
      outputFull: ocrEditableText.value,
    })
  } catch (e: any) {
    error.value = `OCR 识别失败: ${e.message}`
  } finally {
    isOcrRunning.value = false
    loading.close()
  }
}

const handleCopyOcrResult = async () => {
  if (!ocrEditableText.value) return
  try {
    await navigator.clipboard.writeText(ocrEditableText.value)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportOcrResult = async () => {
  if (!ocrEditableText.value) return
  const blob = new Blob([ocrEditableText.value], { type: 'text/plain' })
  await saveFileWithDialog(blob, 'pdf-ocr-result.txt', 'txt')
}

const handleSaveOcrEdit = () => {
  if (!ocrEditableText.value) return
  store.addHistory({
    tool: 'pdf',
    action: 'PDF转图片+OCR(已编辑)',
    inputPreview: pdfFile.value?.name.slice(0, 50) || '',
    outputPreview: ocrEditableText.value.slice(0, 50),
    inputFull: pdfFile.value?.name || '',
    outputFull: ocrEditableText.value,
  })
  ElMessage.success('修改已保存')
}

const handleClearOcrResult = () => {
  ocrResults.value = []
  ocrEditableText.value = ''
}

const handleJumpToOcr = () => {
  if (imageBlobs.value.length === 0) return
  ;(window as any).__pendingOcrBlobs = imageBlobs.value.slice()
  store.activeTool = 'ocr'
}

// ============ Tab 2: 图片转PDF ============
const imageInputRef = ref<HTMLInputElement | null>(null)
const imageFiles = ref<File[]>([])
const imageToPdfOptions = reactive<ImageToPdfOptions>({
  pageSize: 'auto',
  orientation: 'auto',
  quality: 0.92
})
const generatedPdfBlob = ref<Blob | null>(null)

const triggerImageInput = () => imageInputRef.value?.click()

const handleImageFileSelect = (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files) return
  imageFiles.value = [...imageFiles.value, ...Array.from(files)]
  generatedPdfBlob.value = null
  input.value = ''
}

const handleRemoveImage = (idx: number) => {
  imageFiles.value.splice(idx, 1)
  generatedPdfBlob.value = null
}

const handleClearImages = () => {
  imageFiles.value = []
  generatedPdfBlob.value = null
  error.value = ''
  if (imageInputRef.value) imageInputRef.value.value = ''
}

const handleImagesToPdf = async () => {
  if (!imageFiles.value.length) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在生成 PDF（${imageFiles.value.length} 张图片），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    generatedPdfBlob.value = await imagesToPdf(imageFiles.value, imageToPdfOptions)
    ElMessage.success('PDF 生成完成')
    store.addHistory({
      tool: 'pdf',
      action: '图片转PDF',
      inputPreview: `${imageFiles.value.length} 张图片`,
      outputPreview: formatFileSize(generatedPdfBlob.value.size),
      inputFull: imageFiles.value.map(f => f.name).join('\n'),
      outputFull: formatFileSize(generatedPdfBlob.value.size),
    })
  } catch (e: any) {
    error.value = e.message || '生成失败'
  } finally {
    loading.close()
  }
}

const handleDownloadGeneratedPdf = async () => {
  if (!generatedPdfBlob.value) return
  await saveFileWithDialog(generatedPdfBlob.value, 'output.pdf', 'pdf')
}

// ============ Tab 3: PDF文本提取 ============
const extractInputRef = ref<HTMLInputElement | null>(null)
const extractPdfFile = ref<File | null>(null)
const extractedText = ref('')

const triggerExtractInput = () => extractInputRef.value?.click()

const handleExtractPdfSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  error.value = ''
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    error.value = 'PDF 过大，建议小于 100MB'
    return
  }
  extractPdfFile.value = file
  extractedText.value = ''
  input.value = ''
}

const handleClearExtractPdf = () => {
  extractPdfFile.value = null
  extractedText.value = ''
  error.value = ''
  if (extractInputRef.value) extractInputRef.value.value = ''
}

const handleExtractText = async () => {
  if (!extractPdfFile.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: '正在提取 PDF 文本，请稍候...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    extractedText.value = await extractPdfText(extractPdfFile.value)
    ElMessage.success('文本提取完成')
    store.addHistory({
      tool: 'pdf',
      action: 'PDF文本提取',
      inputPreview: extractPdfFile.value.name.slice(0, 50),
      outputPreview: extractedText.value.slice(0, 50),
      inputFull: extractPdfFile.value.name,
      outputFull: extractedText.value,
    })
  } catch (e: any) {
    error.value = e.message || '提取失败'
  } finally {
    loading.close()
  }
}

const handleCopyExtractedText = async () => {
  if (!extractedText.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(extractedText.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleDownloadExtractedText = async () => {
  if (!extractedText.value) return
  const blob = new Blob([extractedText.value], { type: 'text/plain' })
  await saveFileWithDialog(blob, 'extracted-text.txt', 'txt')
}

const handleSaveExtractedEdit = () => {
  if (!extractedText.value) return
  store.addHistory({
    tool: 'pdf',
    action: 'PDF文本提取(已编辑)',
    inputPreview: extractPdfFile.value?.name.slice(0, 50) || '',
    outputPreview: extractedText.value.slice(0, 50),
    inputFull: extractPdfFile.value?.name || '',
    outputFull: extractedText.value,
  })
  ElMessage.success('修改已保存')
}

// ============ Tab 5: PDF转Markdown ============
const markdownInputRef = ref<HTMLInputElement | null>(null)
const markdownPdfFile = ref<File | null>(null)
const markdownPageCount = ref(0)
const markdownMode = ref<'auto' | 'text' | 'ocr'>('auto')
const markdownDpi = ref(150)
const markdownOutput = ref('')
const markdownError = ref('')
const isMarkdownConverting = ref(false)

const markdownHtmlPreview = computed(() => {
  if (!markdownOutput.value) return ''
  return markdownOutput.value
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^#### (.+)$/gm, '<h4>$1</h4>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br>')
    .replace(/^(?!<[h1-6])/gm, '<p>')
    .replace(/(?<!<\/[h1-6]>)$/gm, '</p>')
    .replace(/<p><\/p>/g, '')
})

const triggerMarkdownInput = () => markdownInputRef.value?.click()

const handleMarkdownPdfSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  markdownError.value = ''
  markdownOutput.value = ''
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    markdownError.value = 'PDF 过大，建议小于 100MB'
    return
  }
  markdownPdfFile.value = file

  try {
    const buffer = await file.arrayBuffer()
    const doc = await loadPdfDocument(new Uint8Array(buffer))
    markdownPageCount.value = doc.numPages
  } catch (e: any) {
    markdownError.value = e.message || 'PDF 加载失败'
  }
  input.value = ''
}

const handleClearMarkdownPdf = () => {
  markdownPdfFile.value = null
  markdownPageCount.value = 0
  markdownOutput.value = ''
  markdownError.value = ''
  if (markdownInputRef.value) markdownInputRef.value.value = ''
}

const detectPdfType = async (file: File): Promise<'text' | 'image'> => {
  const buffer = await file.arrayBuffer()
  const doc = await loadPdfDocument(new Uint8Array(buffer))
  let totalTextItems = 0
  const samplePages = Math.min(doc.numPages, 5)
  for (let i = 1; i <= samplePages; i++) {
    const page = await doc.getPage(i)
    const content = await page.getTextContent()
    totalTextItems += content.items.length
  }
  const avgTextItems = totalTextItems / samplePages
  return avgTextItems > 10 ? 'text' : 'image'
}

const extractTextToMarkdown = async (file: File): Promise<string> => {
  const buffer = await file.arrayBuffer()
  const doc = await loadPdfDocument(new Uint8Array(buffer))
  const totalPages = doc.numPages

  const allLines: string[] = []

  for (let i = 1; i <= totalPages; i++) {
    const page = await doc.getPage(i)
    const content = await page.getTextContent()

    const items: Array<{ str: string; y: number; height: number; x: number }> = []
    for (const item of content.items as any[]) {
      if (!item.str || !item.str.trim()) continue
      const transform = item.transform || []
      const y = transform[5] || 0
      const height = transform[3] || 12
      const x = transform[4] || 0
      items.push({ str: item.str, y, height, x })
    }

    if (items.length === 0) continue

    const avgHeight = items.reduce((sum, it) => sum + it.height, 0) / items.length

    const rowTolerance = avgHeight * 0.5
    const rows: Array<Array<{ str: string; y: number; height: number; x: number }>> = []
    const sortedByY = [...items].sort((a, b) => b.y - a.y)

    for (const item of sortedByY) {
      const existingRow = rows.find(row => Math.abs(row[0].y - item.y) <= rowTolerance)
      if (existingRow) {
        existingRow.push(item)
      } else {
        rows.push([item])
      }
    }

    const avgLineHeight = avgHeight * 1.5
    const pageLines: string[] = []
    let prevY: number | null = null

    for (const row of rows) {
      const rowY = row[0].y
      const rowHeight = row.reduce((sum, r) => sum + r.height, 0) / row.length
      const sortedByX = [...row].sort((a, b) => a.x - b.x)
      const lineText = sortedByX.map(r => r.str).join('')

      if (prevY !== null && Math.abs(prevY - rowY) > avgLineHeight * 2) {
        pageLines.push('')
      }

      let prefix = ''
      if (rowHeight > avgHeight * 1.6) prefix = '# '
      else if (rowHeight > avgHeight * 1.3) prefix = '## '
      else if (rowHeight > avgHeight * 1.1) prefix = '### '

      pageLines.push(`${prefix}${lineText}`)
      prevY = rowY
    }

    if (allLines.length > 0) allLines.push('', `--- 第 ${i} 页 ---`, '')
    allLines.push(...pageLines)
  }

  return allLines.join('\n')
}

const ocrToMarkdown = async (file: File, dpi: number): Promise<string> => {
  const images = await pdfToImages(file, dpi)
  const results: string[] = []

  for (let i = 0; i < images.length; i++) {
    const blob = images[i]
    const mdText = await recognizeMarkdown(blob)
    results.push(`--- 第 ${i + 1} 页 ---\n\n${mdText}`)
  }

  return results.join('\n\n')
}

const handlePdfToMarkdown = async () => {
  if (!markdownPdfFile.value) return
  markdownError.value = ''
  markdownOutput.value = ''
  isMarkdownConverting.value = true

  const loading = ElLoading.service({
    lock: true,
    text: '正在转换为 Markdown，请稍候...',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  try {
    let mode: 'text' | 'ocr' = markdownMode.value === 'auto' ? 'text' : markdownMode.value
    if (markdownMode.value === 'auto') {
      const detected = await detectPdfType(markdownPdfFile.value)
      mode = detected === 'text' ? 'text' : 'ocr'
    }

    if (mode === 'text') {
      markdownOutput.value = await extractTextToMarkdown(markdownPdfFile.value)
    } else {
      markdownOutput.value = await ocrToMarkdown(markdownPdfFile.value, markdownDpi.value)
    }

    const modeLabel = mode === 'text' ? '文本提取' : 'OCR识别'
    ElMessage.success(`转换完成（${modeLabel}模式）`)
    store.addHistory({
      tool: 'pdf',
      action: `PDF转Markdown (${modeLabel})`,
      inputPreview: markdownPdfFile.value.name.slice(0, 50),
      outputPreview: markdownOutput.value.slice(0, 50),
      inputFull: markdownPdfFile.value.name,
      outputFull: markdownOutput.value,
    })
  } catch (e: any) {
    markdownError.value = e.message || '转换失败'
  } finally {
    isMarkdownConverting.value = false
    loading.close()
  }
}

const handleCopyMarkdownOutput = async () => {
  if (!markdownOutput.value) return
  try {
    await navigator.clipboard.writeText(markdownOutput.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportMarkdownOutput = async () => {
  if (!markdownOutput.value) return
  const blob = new Blob([markdownOutput.value], { type: 'text/markdown;charset=utf-8' })
  await saveFileWithDialog(blob, 'pdf-to-markdown.md', 'md')
}

const handleSaveMarkdownEdit = () => {
  if (!markdownOutput.value) return
  store.addHistory({
    tool: 'pdf',
    action: 'PDF转Markdown(已编辑)',
    inputPreview: markdownPdfFile.value?.name.slice(0, 50) || '',
    outputPreview: markdownOutput.value.slice(0, 50),
    inputFull: markdownPdfFile.value?.name || '',
    outputFull: markdownOutput.value,
  })
  ElMessage.success('修改已保存')
}

// ============ Tab 4: PDF合并/拆分 ============
const mergeInputRef = ref<HTMLInputElement | null>(null)
const mergePdfFiles = ref<File[]>([])
const mergePageRanges = ref<string[]>([])
const customPageRange = ref('')
const mergedPdfBlob = ref<Blob | null>(null)
const singleFilePageCount = ref(0)

const triggerMergeInput = () => mergeInputRef.value?.click()

const handleMergePdfSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files) return
  const newFiles = Array.from(files)
  mergePdfFiles.value = [...mergePdfFiles.value, ...newFiles]
  mergePageRanges.value = [...mergePageRanges.value, ...newFiles.map(() => 'all')]
  mergedPdfBlob.value = null
  input.value = ''

  // 单文件时检测总页数，方便用户参考
  if (mergePdfFiles.value.length === 1) {
    try {
      const buffer = await mergePdfFiles.value[0].arrayBuffer()
      const doc = await loadPdfDocument(new Uint8Array(buffer))
      singleFilePageCount.value = doc.numPages
    } catch {
      singleFilePageCount.value = 0
    }
  } else {
    singleFilePageCount.value = 0
  }
}

const handleRemoveMergePdf = async (idx: number) => {
  mergePdfFiles.value.splice(idx, 1)
  mergePageRanges.value.splice(idx, 1)
  mergedPdfBlob.value = null

  // 删除后如果只剩1个文件，重新检测页数
  if (mergePdfFiles.value.length === 1) {
    try {
      const buffer = await mergePdfFiles.value[0].arrayBuffer()
      const doc = await loadPdfDocument(new Uint8Array(buffer))
      singleFilePageCount.value = doc.numPages
    } catch {
      singleFilePageCount.value = 0
    }
  } else {
    singleFilePageCount.value = 0
  }
}

const handleClearMergePdfs = () => {
  mergePdfFiles.value = []
  mergePageRanges.value = []
  customPageRange.value = ''
  mergedPdfBlob.value = null
  singleFilePageCount.value = 0
  error.value = ''
  if (mergeInputRef.value) mergeInputRef.value.value = ''
}

const handleQuickMerge = async (range: string) => {
  if (!mergePdfFiles.value.length) return
  error.value = ''
  customPageRange.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在合并 PDF（${mergePdfFiles.value.length} 个文件），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    mergedPdfBlob.value = await mergePdf(mergePdfFiles.value, mergePdfFiles.value.map(() => range))
    ElMessage.success('PDF 合并完成')
    store.addHistory({
      tool: 'pdf',
      action: `PDF合并 (${range})`,
      inputPreview: `${mergePdfFiles.value.length} 个文件`,
      outputPreview: formatFileSize(mergedPdfBlob.value.size),
      inputFull: mergePdfFiles.value.map(f => f.name).join('\n'),
      outputFull: formatFileSize(mergedPdfBlob.value.size),
    })
  } catch (e: any) {
    error.value = e.message || '合并失败'
  } finally {
    loading.close()
  }
}

const handleCustomMerge = async () => {
  if (!mergePdfFiles.value.length || !customPageRange.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在生成 PDF（${mergePdfFiles.value.length} 个文件），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    // 自定义页码对所有文件统一应用
    const ranges = mergePdfFiles.value.map(() => customPageRange.value)
    mergedPdfBlob.value = await mergePdf(mergePdfFiles.value, ranges)
    ElMessage.success('PDF 生成完成')
    store.addHistory({
      tool: 'pdf',
      action: `PDF自定义提取 [${customPageRange.value}]`,
      inputPreview: `${mergePdfFiles.value.length} 个文件`,
      outputPreview: formatFileSize(mergedPdfBlob.value.size),
      inputFull: mergePdfFiles.value.map((f) => `${f.name} [${customPageRange.value}]`).join('\n'),
      outputFull: formatFileSize(mergedPdfBlob.value.size),
    })
  } catch (e: any) {
    error.value = e.message || '合并失败'
  } finally {
    loading.close()
  }
}

const handleDownloadMergedPdf = async () => {
  if (!mergedPdfBlob.value) return
  await saveFileWithDialog(mergedPdfBlob.value, 'merged.pdf', 'pdf')
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.pdf-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .pdf-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.pdf-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.pdf-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.pdf-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.pdf-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.pdf-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 工具卡片 ===== */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.group-buttons {
  display: flex;
  gap: 6px;
}

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

/* 文件信息 */
.file-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.file-name {
  color: var(--text-primary);
  font-weight: 500;
}

.upload-hint {
  color: var(--text-muted);
  font-size: 13px;
  text-align: center;
  padding: 20px;
}

/* 结果信息 */
.result-info {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

/* 图片预览网格 */
.image-preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.image-preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px;
  background: var(--bg-input);
  border-radius: 6px;
}

.image-preview-item img {
  width: 100%;
  height: auto;
  border-radius: 4px;
  max-height: 200px;
  object-fit: contain;
}

.image-label {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 图片列表 */
.image-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.image-list-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.image-list-index {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.image-list-name {
  flex: 1;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-list-size {
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 页码范围提示 */
.page-range-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-muted);
}

.page-range-hint code {
  background: var(--bg-input);
  padding: 1px 5px;
  border-radius: 3px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

/* 合并文件列表 */
.merge-file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.merge-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.merge-file-index {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.merge-file-name {
  flex: 1;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.merge-file-size {
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 文本信息 */
.text-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

/* 错误提示 */
.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}

/* ===== OCR 结果 ===== */
.ocr-result-section {
  margin-top: 8px;
}

.ocr-result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.ocr-result-title {
  font-size: 13px;
  color: var(--accent-cyan);
  font-weight: 500;
}

.ocr-actions {
  display: flex;
  gap: 6px;
}

.ocr-textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* ===== OCR 提示 ===== */
.ocr-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  padding: 8px 12px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: 4px;
  font-size: 12px;
  color: #f59e0b;
}

.ocr-hint .el-icon {
  flex-shrink: 0;
}

/* ===== Markdown 输出 ===== */
.markdown-output-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.markdown-source-panel,
.markdown-preview-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.panel-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.markdown-textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  width: 100%;
}

.markdown-preview {
  padding: 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  min-height: 300px;
  max-height: 500px;
  overflow-y: auto;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary);
}

.markdown-preview h1,
.markdown-preview h2,
.markdown-preview h3,
.markdown-preview h4 {
  color: var(--accent-cyan);
  margin-top: 16px;
  margin-bottom: 8px;
  font-weight: 600;
}

.markdown-preview h1 { font-size: 22px; }
.markdown-preview h2 { font-size: 18px; }
.markdown-preview h3 { font-size: 16px; }
.markdown-preview h4 { font-size: 14px; }

.markdown-preview p {
  margin: 8px 0;
}

@media (max-width: 900px) {
  .markdown-output-grid {
    grid-template-columns: 1fr;
  }
}

/* ===== PDF压缩 ===== */
.compress-drop-zone {
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  padding: 16px;
  transition: border-color 0.3s, background-color 0.3s;
  min-height: 60px;
  display: flex;
  align-items: center;
}

.compress-drop-zone.drag-over {
  border-color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.05);
}

.compress-file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
  max-height: 300px;
  overflow-y: auto;
}

.compress-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.file-index {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.compress-level-hint {
  margin-top: 12px;
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  line-height: 1.5;
}

.compress-custom-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
}

.custom-toggle-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.compress-custom-params {
  margin-top: 12px;
  padding: 12px 16px;
  background: var(--bg-input);
  border-radius: 4px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.custom-param {
  display: flex;
  align-items: center;
  gap: 12px;
}

.param-label {
  font-size: 13px;
  color: var(--text-secondary);
  width: 80px;
  flex-shrink: 0;
}

.gs-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  font-size: 12px;
  color: var(--accent-green);
  padding: 6px 12px;
  background: rgba(16, 185, 129, 0.08);
  border-radius: 4px;
}

.compress-table {
  width: 100%;
  --el-table-bg-color: transparent;
  --el-table-tr-bg-color: transparent;
  --el-table-header-bg-color: rgba(0, 0, 0, 0.2);
  --el-table-row-hover-bg-color: rgba(0, 212, 255, 0.05);
  --el-table-border-color: var(--border-color);
  --el-table-text-color: var(--text-primary);
  --el-table-header-text-color: var(--text-secondary);
}

:deep(.compress-table .el-table__header th) {
  background: rgba(0, 0, 0, 0.2);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-color);
}

:deep(.compress-table .el-table__body td) {
  background: transparent;
  color: var(--text-primary);
  border-bottom: 1px solid rgba(30, 58, 95, 0.3);
}

:deep(.compress-table .el-table__body tr:hover > td) {
  background: rgba(0, 212, 255, 0.05) !important;
}

html.light :deep(.compress-table .el-table__header th) {
  background: #f8fafc;
}

html.light :deep(.compress-table .el-table__body td) {
  border-bottom: 1px solid rgba(226, 232, 240, 0.5);
}

.ratio-positive {
  color: var(--accent-green);
  font-weight: 600;
}

.ratio-negative {
  color: var(--accent-orange);
  font-weight: 600;
}

.compress-summary {
  margin-top: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-secondary);
  text-align: right;
}

/* ========== 提取图片 Tab ========== */
.extract-preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
  margin-top: 14px;
}

.extract-preview-item {
  display: flex;
  flex-direction: column;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  transition: all 0.2s ease;
}
.extract-preview-item:hover {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 0 1px var(--accent-cyan), 0 4px 14px rgba(0, 212, 255, 0.12);
  transform: translateY(-1px);
}

.extract-preview-thumb {
  height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0d1520 0%, #1a2332 100%);
  padding: 8px;
  border-bottom: 1px solid var(--border-color);
  overflow: hidden;
}
html.light .extract-preview-thumb {
  background: linear-gradient(135deg, #f0f4f8 0%, #e2e8f0 100%);
}

.extract-preview-thumb img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 4px;
}

.extract-preview-meta {
  padding: 10px 12px 6px;
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 12px;
}

.preview-title {
  font-weight: 600;
  color: var(--accent-cyan);
  font-size: 13px;
  letter-spacing: 0.5px;
}

.preview-dims {
  color: var(--text-primary);
  font-family: 'Consolas', 'Courier New', monospace;
  font-weight: 500;
}

.preview-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 2px;
}

.tag {
  display: inline-block;
  padding: 1px 8px;
  background: var(--bg-active);
  color: var(--text-secondary);
  border-radius: 10px;
  font-size: 11px;
  line-height: 18px;
  border: 1px solid var(--border-color);
  font-family: 'Consolas', 'Courier New', monospace;
}

.tag-format {
  color: var(--text-primary);
  font-weight: 700;
  letter-spacing: 0.4px;
}
.tag-format.tag-jpeg { border-color: var(--accent-orange); color: var(--accent-orange); background: rgba(245,158,11,0.08); }
.tag-format.tag-png  { border-color: var(--accent-green);  color: var(--accent-green);  background: rgba(16,185,129,0.08); }
.tag-format.tag-jp2  { border-color: var(--accent-blue);   color: var(--accent-blue);   background: rgba(59,130,246,0.08); }
.tag-format.tag-tiff { border-color: #a78bfa;              color: #a78bfa;              background: rgba(167,139,250,0.08); }
.tag-format.tag-raw  { border-color: var(--text-secondary);color: var(--text-secondary);background: var(--bg-input); }

.tag-cs {
  font-style: italic;
}

.preview-xobj {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

.extract-preview-actions {
  margin-top: auto;
  padding: 8px 12px 12px;
  display: flex;
  justify-content: flex-end;
}

.empty-hint {
  margin-top: 16px;
  padding: 24px;
  background: var(--bg-input);
  border: 1px dashed var(--border-color);
  border-radius: 6px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}
</style>
