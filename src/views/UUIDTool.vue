<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">ID 生成器</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>生成各类唯一标识 ID</p>
                <p>• 每张卡片可独立配置并生成</p>
                <p>• 支持全部生成 / 清除全部</p>
                <p>• 自增序列在同一会话内持续递增</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <el-button size="small" @click="handleClearAll">清除全部</el-button>
          <el-button size="small" type="primary" @click="handleGenerateAll">全部生成</el-button>
        </div>
      </div>
    </div>

    <div class="data-grid">
      <!-- UUID v4 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">UUID v4</span>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('uuid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('uuid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.uuid.count" :min="1" :max="100" size="small" style="width: 80px" />
            <el-select v-model="options.uuid.format" size="small" style="width: 100px">
              <el-option label="标准" value="standard" />
              <el-option label="无横杠" value="no-dash" />
              <el-option label="大写" value="upper" />
              <el-option label="大写无横杠" value="upper-no-dash" />
            </el-select>
          </div>
          <div v-if="results.uuid.length" class="result-list">
            <div v-for="(item, idx) in results.uuid" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 雪花算法ID -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">雪花算法ID</span>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('snowflake')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('snowflake')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.snowflake.count" :min="1" :max="100" size="small" style="width: 80px" />
            <span class="option-label">机器ID</span>
            <el-input-number v-model="options.snowflake.machineId" :min="0" :max="1023" size="small" style="width: 80px" />
          </div>
          <div v-if="results.snowflake.length" class="result-list">
            <div v-for="(item, idx) in results.snowflake" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- MongoDB ObjectId -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">MongoDB ObjectId</span>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('objectid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('objectid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.objectid.count" :min="1" :max="100" size="small" style="width: 80px" />
          </div>
          <div v-if="results.objectid.length" class="result-list">
            <div v-for="(item, idx) in results.objectid" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 自增序列 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">自增序列</span>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('sequence')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('sequence')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row wrap">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.sequence.count" :min="1" :max="100" size="small" style="width: 80px" />
            <span class="option-label">前缀</span>
            <el-input v-model="options.sequence.prefix" size="small" style="width: 90px" placeholder="如 ORD-" />
            <span class="option-label">起始</span>
            <el-input-number v-model="options.sequence.start" :min="0" size="small" style="width: 90px" />
            <span class="option-label">步长</span>
            <el-input-number v-model="options.sequence.step" :min="1" size="small" style="width: 80px" />
            <span class="option-label">补零</span>
            <el-input-number v-model="options.sequence.padLength" :min="0" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.sequence.length" class="result-list">
            <div v-for="(item, idx) in results.sequence" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- NanoID -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">NanoID</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>URL 安全短 ID，长度可调，体积小</p>
                  <p>适合：短期 token、短链接、日志追踪 ID、临时会话 ID</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('nanoid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('nanoid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.nanoid.count" :min="1" :max="100" size="small" style="width: 80px" />
            <span class="option-label">长度</span>
            <el-input-number v-model="options.nanoid.size" :min="4" :max="64" size="small" style="width: 80px" />
          </div>
          <div v-if="results.nanoid.length" class="result-list">
            <div v-for="(item, idx) in results.nanoid" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- ULID -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">ULID</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>26 字符，按生成时间排序（时间有序）</p>
                  <p>适合：分布式主键、事件流/日志 ID、按时间倒序查询的场景，比雪花短且可读</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('ulid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('ulid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.ulid.count" :min="1" :max="100" size="small" style="width: 80px" />
          </div>
          <div v-if="results.ulid.length" class="result-list">
            <div v-for="(item, idx) in results.ulid" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- UUID v7 -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">UUID v7</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>时间有序版 UUID（RFC 9562 新标准）</p>
                  <p>适合：新项目数据库主键，兼顾 UUID 格式与按时间排序，索引友好</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('uuidv7')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('uuidv7')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.uuidv7.count" :min="1" :max="100" size="small" style="width: 80px" />
          </div>
          <div v-if="results.uuidv7.length" class="result-list">
            <div v-for="(item, idx) in results.uuidv7" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- UUID v1 -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">UUID v1</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>时间戳 + 时钟序列 + 节点，可反解生成时间</p>
                  <p>适合：需从 ID 反推创建时间的场景（如数据审计）；含节点信息，隐私敏感环境慎用</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('uuidv1')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('uuidv1')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.uuidv1.count" :min="1" :max="100" size="small" style="width: 80px" />
          </div>
          <div v-if="results.uuidv1.length" class="result-list">
            <div v-for="(item, idx) in results.uuidv1" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- UUID v5 -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">UUID v5</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>命名空间 + 名字 SHA-1 哈希，确定性：同一输入永远得到同一 ID</p>
                  <p>区别于 MD5：输出为合规 UUID 格式，可直接用于数据库 UUID 字段 / API 资源 ID</p>
                  <p>适合：为 URL/邮箱/文件路径等资源生成稳定主键；命名空间留空时使用 DNS 默认值</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('uuidv5')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('uuidv5')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">名称</span>
            <el-input v-model="options.uuidv5.name" placeholder="如 www.example.com" size="small" style="width: 160px" />
            <span class="option-label">命名空间</span>
            <el-input v-model="options.uuidv5.namespace" placeholder="留空默认 DNS" size="small" style="width: 200px" />
          </div>
          <div v-if="results.uuidv5.length" class="result-list">
            <div v-for="(item, idx) in results.uuidv5" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- KSUID -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">KSUID</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>27 字符 Base62，秒级时间戳 + 高强度随机，时间有序</p>
                  <p>适合：日志/事件系统主键、需要按时间排序且防猜测的 ID</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('ksuid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('ksuid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.ksuid.count" :min="1" :max="100" size="small" style="width: 80px" />
          </div>
          <div v-if="results.ksuid.length" class="result-list">
            <div v-for="(item, idx) in results.ksuid" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- CUID2 -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">CUID2</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>24 字符 base36 短 ID：时间 + 计数器 + 随机，支持自定义前缀</p>
                  <p>适合：前端生成主键，前缀可区分来源（如 user_xxx / order_xxx）</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('cuid2')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('cuid2')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.cuid2.count" :min="1" :max="100" size="small" style="width: 80px" />
            <span class="option-label">前缀</span>
            <el-input v-model="options.cuid2.prefix" placeholder="如 user_" size="small" style="width: 100px" />
          </div>
          <div v-if="results.cuid2.length" class="result-list">
            <div v-for="(item, idx) in results.cuid2" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- XID -->
      <div class="tool-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">XID</span>
            <el-tooltip placement="bottom" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>20 字符 Base32hex：秒级时间戳 + 机器ID + 进程ID + 计数器，时间可排序</p>
                  <p>适合：Go 生态项目主键，比 ObjectId 更短、可读性好</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleGenerate('xid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('xid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.xid.count" :min="1" :max="100" size="small" style="width: 80px" />
          </div>
          <div v-if="results.xid.length" class="result-list">
            <div v-for="(item, idx) in results.xid" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { generateIds, ID_TYPE_LABELS } from '@/utils/uuidUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const UUID_FORMAT_MAP: Record<string, { uppercase: boolean; removeDashes: boolean }> = {
  standard: { uppercase: false, removeDashes: false },
  'no-dash': { uppercase: false, removeDashes: true },
  upper: { uppercase: true, removeDashes: false },
  'upper-no-dash': { uppercase: true, removeDashes: true },
}

// 每类 ID 独立配置
const options = reactive({
  uuid: { count: 1, format: 'standard' as 'standard' | 'no-dash' | 'upper' | 'upper-no-dash' },
  uuidv1: { count: 1 },
  uuidv5: { name: 'www.example.com', namespace: '' },
  uuidv7: { count: 1 },
  snowflake: { count: 1, machineId: 0 },
  objectid: { count: 1 },
  sequence: { count: 1, prefix: '', start: 1, step: 1, padLength: 0 },
  nanoid: { count: 1, size: 21 },
  ulid: { count: 1 },
  ksuid: { count: 1 },
  cuid2: { count: 1, prefix: '' },
  xid: { count: 1 },
})

// 各类型结果独立存储
const results = reactive<Record<string, string[]>>({
  uuid: [],
  uuidv1: [],
  uuidv5: [],
  uuidv7: [],
  snowflake: [],
  objectid: [],
  sequence: [],
  nanoid: [],
  ulid: [],
  ksuid: [],
  cuid2: [],
  xid: [],
})

// 自增序列会话内递增：记录下次生成时的起始值（刷新页面后重置）
const seqNextStart = ref(options.sequence.start)

// 调整自增序列任意参数（数量/前缀/起始值/步长/补零位数）时，重新开始计数
watch(
  () => [options.sequence.count, options.sequence.prefix, options.sequence.start, options.sequence.step, options.sequence.padLength],
  () => {
    seqNextStart.value = options.sequence.start
  }
)

// 生成单个类型
const handleGenerate = async (type: string) => {
  if (type === 'uuid') {
    const f = UUID_FORMAT_MAP[options.uuid.format]
    results.uuid = await generateIds({ type: 'uuid', count: options.uuid.count, uppercase: f.uppercase, removeDashes: f.removeDashes })
  } else if (type === 'snowflake') {
    results.snowflake = await generateIds({ type: 'snowflake', count: options.snowflake.count, machineId: options.snowflake.machineId })
  } else if (type === 'objectid') {
    results.objectid = await generateIds({ type: 'objectid', count: options.objectid.count })
  } else if (type === 'sequence') {
    results.sequence = await generateIds({
      type: 'sequence',
      count: options.sequence.count,
      prefix: options.sequence.prefix,
      start: seqNextStart.value,
      step: options.sequence.step,
      padLength: options.sequence.padLength,
    })
    seqNextStart.value = seqNextStart.value + options.sequence.step * options.sequence.count
  } else if (type === 'nanoid') {
    results.nanoid = await generateIds({ type: 'nanoid', count: options.nanoid.count, nanoIdSize: options.nanoid.size })
  } else if (type === 'ulid') {
    results.ulid = await generateIds({ type: 'ulid', count: options.ulid.count })
  } else if (type === 'uuidv1') {
    results.uuidv1 = await generateIds({ type: 'uuidv1', count: options.uuidv1.count })
  } else if (type === 'uuidv7') {
    results.uuidv7 = await generateIds({ type: 'uuidv7', count: options.uuidv7.count })
  } else if (type === 'uuidv5') {
    results.uuidv5 = await generateIds({ type: 'uuidv5', count: 1, name: options.uuidv5.name, namespace: options.uuidv5.namespace })
  } else if (type === 'ksuid') {
    results.ksuid = await generateIds({ type: 'ksuid', count: options.ksuid.count })
  } else if (type === 'cuid2') {
    results.cuid2 = await generateIds({ type: 'cuid2', count: options.cuid2.count, prefix: options.cuid2.prefix })
  } else if (type === 'xid') {
    results.xid = await generateIds({ type: 'xid', count: options.xid.count })
  }

  const opt = (options as any)[type]
  store.addHistory({
    tool: 'uuid',
    action: ID_TYPE_LABELS[type as keyof typeof ID_TYPE_LABELS],
    inputPreview: `count=${opt.count ?? 1}`,
    outputPreview: results[type][0] || '',
    inputFull: JSON.stringify(opt),
    outputFull: results[type].join('\n'),
  })

  ElMessage.success(`已生成 ${opt.count ?? 1} 个 ${ID_TYPE_LABELS[type as keyof typeof ID_TYPE_LABELS]}`)
}

// 全部生成
const handleGenerateAll = async () => {
  await Promise.all(
    (['uuid', 'uuidv1', 'uuidv5', 'uuidv7', 'snowflake', 'objectid', 'sequence', 'nanoid', 'ulid', 'ksuid', 'cuid2', 'xid'] as const).map((type) => handleGenerate(type))
  )
}

// 清除全部
const handleClearAll = () => {
  Object.keys(results).forEach((key) => {
    results[key] = []
  })
  seqNextStart.value = options.sequence.start
  ElMessage.success('已清除全部数据')
}

// 批量复制某类型全部结果
const handleCopyAll = async (type: string) => {
  const list = results[type]
  if (!list.length) {
    ElMessage.warning('暂无生成结果')
    return
  }
  try {
    await navigator.clipboard.writeText(list.join('\n'))
    ElMessage.success(`已复制全部 ${list.length} 条`)
  } catch {
    ElMessage.error('复制失败')
  }
}

// 复制
const handleCopy = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
/* 标题栏辅助 */
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.header-actions {
  display: flex;
  align-items: center;
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
.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}
.tooltip-content p {
  margin: 2px 0;
}

/* 卡片网格布局 */
.data-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}
@media (max-width: 1200px) {
  .data-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
@media (max-width: 768px) {
  .data-grid {
    grid-template-columns: 1fr;
  }
}

/* 选项行 */
.options-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}
.options-row.wrap {
  flex-wrap: wrap;
}
.option-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
  white-space: nowrap;
}

/* 结果列表 */
.result-list {
  max-height: 200px;
  overflow-y: auto;
}
.result-list::-webkit-scrollbar {
  width: 4px;
}
.result-list::-webkit-scrollbar-track {
  background: transparent;
}
.result-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}
.data-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 8px;
  transition: border-color 0.3s;
}
.data-item:hover {
  border-color: var(--accent-cyan);
}
.item-index {
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  min-width: 30px;
  text-align: center;
}
.item-text {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}
</style>
