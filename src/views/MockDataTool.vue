<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">随机假数据</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>生成各类模拟测试数据</p>
                <p>• 所有数据均为随机生成，仅供测试使用</p>
                <p>• 每个卡片可独立配置生成数量</p>
                <p>• 部分类型支持额外选项配置</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <span class="option-label">数量</span>
          <el-input-number v-model="allCount" :min="1" :max="50" size="small" style="width: 80px" />
          <el-button size="small" @click="handleClearAll">清除全部</el-button>
          <el-button size="small" type="primary" @click="handleGenerateAll">全部生成</el-button>
        </div>
      </div>
    </div>

    <div class="data-grid">
      <!-- 姓名 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">姓名</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('name')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('name')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.name.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.name.gender" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="男" value="male" />
              <el-option label="女" value="female" />
            </el-select>
          </div>
          <div v-if="results.name.length" class="result-list">
            <div v-for="(item, idx) in results.name" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 英文姓名 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">英文姓名</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('englishName')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('englishName')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.englishName.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.englishName.gender" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="男" value="male" />
              <el-option label="女" value="female" />
            </el-select>
          </div>
          <div v-if="results.englishName.length" class="result-list">
            <div v-for="(item, idx) in results.englishName" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 身份证 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">身份证</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('idCard')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('idCard')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.idCard.count" :min="1" :max="20" size="small" style="width: 80px" />
            <el-select v-model="options.idCard.gender" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="男" value="male" />
              <el-option label="女" value="female" />
            </el-select>
          </div>
          <div v-if="results.idCard.length" class="result-list">
            <div v-for="(item, idx) in results.idCard" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 手机号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">手机号</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('phone')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('phone')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.phone.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.phone.carrier" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="移动" value="mobile" />
              <el-option label="联通" value="unicom" />
              <el-option label="电信" value="telecom" />
            </el-select>
          </div>
          <div v-if="results.phone.length" class="result-list">
            <div v-for="(item, idx) in results.phone" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 邮箱 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">邮箱</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('email')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('email')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.email.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.email.domain" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="QQ" value="qq" />
              <el-option label="163" value="163" />
              <el-option label="Gmail" value="gmail" />
            </el-select>
          </div>
          <div v-if="results.email.length" class="result-list">
            <div v-for="(item, idx) in results.email" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 国内地址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">国内地址</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('address')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('address')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.address.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.address.length" class="result-list">
            <div v-for="(item, idx) in results.address" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 邮政编码 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">邮政编码</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('zipCode')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('zipCode')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.zipCode.count" :min="1" :max="50" size="small" style="width: 80px" />
          </div>
          <div v-if="results.zipCode.length" class="result-list">
            <div v-for="(item, idx) in results.zipCode" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 个人档案 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">个人档案</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('profile')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('profile')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.profile.count" :min="1" :max="10" size="small" style="width: 80px" />
          </div>
          <div v-if="results.profile.length" class="result-list json-list">
            <div v-for="(item, idx) in results.profile" :key="idx" class="data-item json-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <pre class="item-text json-content">{{ item }}</pre>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 用户名 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">用户名</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('username')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('username')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.username.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.username.style" size="small" style="width: 90px">
              <el-option label="随机" value="random" />
              <el-option label="下划线" value="snake" />
              <el-option label="点" value="dot" />
              <el-option label="横杠" value="dash" />
              <el-option label="无分隔" value="none" />
            </el-select>
          </div>
          <div v-if="results.username.length" class="result-list">
            <div v-for="(item, idx) in results.username" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- IP地址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">IP地址</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('ip')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('ip')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.ip.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.ip.type" size="small" style="width: 80px">
              <el-option label="IPv4" value="ipv4" />
              <el-option label="IPv6" value="ipv6" />
            </el-select>
          </div>
          <div v-if="results.ip.length" class="result-list">
            <div v-for="(item, idx) in results.ip" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- MAC 地址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">MAC 地址</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('mac')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('mac')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.mac.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.mac.format" size="small" style="width: 80px">
              <el-option label="冒号" value="colon" />
              <el-option label="横杠" value="dash" />
              <el-option label="点分" value="dot" />
            </el-select>
          </div>
          <div v-if="results.mac.length" class="result-list">
            <div v-for="(item, idx) in results.mac" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 网址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">网址</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('url')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('url')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.url.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.url.protocol" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="HTTP" value="http" />
              <el-option label="HTTPS" value="https" />
            </el-select>
          </div>
          <div v-if="results.url.length" class="result-list">
            <div v-for="(item, idx) in results.url" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 公司名称 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">公司名称</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('company')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('company')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.company.count" :min="1" :max="50" size="small" style="width: 80px" />
          </div>
          <div v-if="results.company.length" class="result-list">
            <div v-for="(item, idx) in results.company" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 统一社会信用代码 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">统一社会信用代码</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('creditCode')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('creditCode')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.creditCode.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.creditCode.length" class="result-list">
            <div v-for="(item, idx) in results.creditCode" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 银行卡号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">银行卡号</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('bankCard')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('bankCard')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.bankCard.count" :min="1" :max="20" size="small" style="width: 80px" />
            <el-select v-model="options.bankCard.type" size="small" style="width: 80px">
              <el-option label="储蓄卡" value="debit" />
              <el-option label="信用卡" value="credit" />
            </el-select>
          </div>
          <div v-if="results.bankCard.length" class="result-list">
            <div v-for="(item, idx) in results.bankCard" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 金额 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">金额</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('amount')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('amount')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.amount.count" :min="1" :max="50" size="small" style="width: 80px" />
            <span class="option-label">范围</span>
            <el-input-number v-model="options.amount.min" :min="0.01" :step="10" size="small" style="width: 90px" />
            <span style="color: var(--text-secondary)">~</span>
            <el-input-number v-model="options.amount.max" :min="0.01" :step="100" size="small" style="width: 100px" />
          </div>
          <div class="options-row">
            <span class="option-label">小数位</span>
            <el-select v-model="options.amount.decimals" size="small" style="width: 90px" :disabled="options.amount.upper">
              <el-option label="0位" :value="0" />
              <el-option label="1位" :value="1" />
              <el-option label="2位" :value="2" />
            </el-select>
            <span class="option-label">货币</span>
            <el-select v-model="options.amount.currency" size="small" style="width: 80px" :disabled="options.amount.upper">
              <el-option label="¥" value="cny" />
              <el-option label="$" value="usd" />
              <el-option label="€" value="eur" />
              <el-option label="无" value="none" />
            </el-select>
            <span class="option-label">大写</span>
            <el-switch v-model="options.amount.upper" size="small" />
          </div>
          <div v-if="results.amount.length" class="result-list">
            <div v-for="(item, idx) in results.amount" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 快递单号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">快递单号</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('courier')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('courier')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.courier.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.courier.carrier" size="small" style="width: 90px">
              <el-option label="随机" value="random" />
              <el-option label="顺丰" value="sf" />
              <el-option label="圆通" value="yt" />
              <el-option label="中通" value="zt" />
              <el-option label="韵达" value="yd" />
              <el-option label="申通" value="sto" />
              <el-option label="EMS" value="ems" />
              <el-option label="京东" value="jd" />
            </el-select>
          </div>
          <div v-if="results.courier.length" class="result-list">
            <div v-for="(item, idx) in results.courier" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 车牌号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">车牌号</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('plate')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('plate')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.plate.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.plate.length" class="result-list">
            <div v-for="(item, idx) in results.plate" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 车架号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">车架号</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('vin')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('vin')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.vin.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.vin.length" class="result-list">
            <div v-for="(item, idx) in results.vin" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 经纬度坐标 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">经纬度坐标</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('coordinate')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('coordinate')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.coordinate.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.coordinate.range" size="small" style="width: 90px">
              <el-option label="国内" value="china" />
              <el-option label="全球" value="global" />
            </el-select>
            <span class="option-label">小数位</span>
            <el-input-number v-model="options.coordinate.decimals" :min="0" :max="6" size="small" style="width: 80px" />
          </div>
          <div v-if="results.coordinate.length" class="result-list">
            <div v-for="(item, idx) in results.coordinate" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- UUID -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">UUID</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('uuid')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('uuid')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.uuid.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.uuid.format" size="small" style="width: 80px">
              <el-option label="标准" value="standard" />
              <el-option label="无横杠" value="no-dash" />
              <el-option label="大写" value="upper" />
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

      <!-- 日期时间 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">日期时间</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('dateTime')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('dateTime')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.dateTime.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.dateTime.range" size="small" style="width: 80px">
              <el-option label="近7天" value="recent7" />
              <el-option label="近30天" value="recent30" />
              <el-option label="近1年" value="recent365" />
            </el-select>
            <el-select v-model="options.dateTime.format" size="small" style="width: 90px">
              <el-option label="日期" value="date" />
              <el-option label="日期时间" value="datetime" />
              <el-option label="时间戳" value="timestamp" />
            </el-select>
          </div>
          <div v-if="results.dateTime.length" class="result-list">
            <div v-for="(item, idx) in results.dateTime" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 颜色值 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">颜色值</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('color')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('color')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.color.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.color.format" size="small" style="width: 80px">
              <el-option label="HEX" value="hex" />
              <el-option label="RGB" value="rgb" />
              <el-option label="HSL" value="hsl" />
            </el-select>
          </div>
          <div v-if="results.color.length" class="result-list">
            <div v-for="(item, idx) in results.color" :key="idx" class="data-item color-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <span class="color-preview" :style="{ background: item }"></span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 随机文本 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">随机文本</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('text')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('text')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.text.count" :min="1" :max="10" size="small" style="width: 80px" />
            <span class="option-label">字数</span>
            <el-input-number v-model="options.text.wordCount" :min="20" :max="500" :step="10" size="small" style="width: 80px" />
          </div>
          <div v-if="results.text.length" class="result-list">
            <div v-for="(item, idx) in results.text" :key="idx" class="data-item text-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <span class="item-text text-content">{{ item }}</span>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 随机 JSON -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">随机 JSON</span>
          <div class="header-actions">
            <el-button size="small" @click="handleGenerate('json')">生成</el-button>
            <el-button size="small" @click="handleCopyAll('json')">批量复制</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.json.count" :min="1" :max="10" size="small" style="width: 80px" />
            <span class="option-label">深度</span>
            <el-input-number v-model="options.json.depth" :min="1" :max="5" size="small" style="width: 80px" />
          </div>
          <div v-if="results.json.length" class="result-list json-list">
            <div v-for="(item, idx) in results.json" :key="idx" class="data-item json-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <pre class="item-text json-content">{{ item }}</pre>
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
import {
  generateName,
  generateIdCard,
  generatePhone,
  generateEmail,
  generateIP,
  generateURL,
  generateAddress,
  generateBankCard,
  generateCreditCode,
  generateVIN,
  generatePlate,
  generateMAC,
  generateText,
  generateDateTime,
  generateZipCode,
  generateUUID,
  generateColor,
  generateJSON,
  generateEnglishName,
  generateCoordinate,
  generateCompanyName,
  generatePersonalProfile,
  generateUsername,
  generateCourierNumber,
  generateAmountValues,
  formatAmountValue
} from '@/utils/mockDataUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// 配置选项
const options = reactive({
  name: { count: 1, gender: 'random' as 'male' | 'female' | 'random' },
  idCard: { count: 1, gender: 'random' as 'male' | 'female' | 'random', ageRange: 'random' as 'adult' | 'elder' | 'random' },
  phone: { count: 1, carrier: 'random' as 'mobile' | 'unicom' | 'telecom' | 'random' },
  email: { count: 1, domain: 'random' as 'qq' | '163' | '126' | 'gmail' | 'outlook' | 'sina' | 'random' },
  ip: { count: 1, type: 'ipv4' as 'ipv4' | 'ipv6' },
  url: { count: 1, protocol: 'random' as 'http' | 'https' | 'random' },
  address: { count: 1 },
  bankCard: { count: 1, type: 'debit' as 'debit' | 'credit' },
  creditCode: { count: 1 },
  vin: { count: 1 },
  plate: { count: 1 },
  mac: { count: 1, format: 'colon' as 'colon' | 'dash' | 'dot' },
  text: { count: 1, wordCount: 100 },
  dateTime: { count: 1, range: 'recent30' as 'recent7' | 'recent30' | 'recent365', format: 'datetime' as 'date' | 'datetime' | 'timestamp' },
  zipCode: { count: 1 },
  uuid: { count: 1, format: 'standard' as 'standard' | 'no-dash' | 'upper' },
  color: { count: 1, format: 'hex' as 'hex' | 'rgb' | 'hsl' },
  json: { count: 1, depth: 2 },
  englishName: { count: 1, gender: 'random' as 'male' | 'female' | 'random' },
  coordinate: { count: 1, range: 'china' as 'china' | 'global', decimals: 6 },
  company: { count: 1 },
  profile: { count: 1 },
  username: { count: 1, style: 'random' as 'random' | 'snake' | 'dot' | 'dash' | 'none' },
  courier: { count: 1, carrier: 'random' as 'sf' | 'yt' | 'zt' | 'yd' | 'sto' | 'ems' | 'jd' | 'random' },
  amount: { count: 1, min: 1, max: 10000, decimals: 2 as 0 | 1 | 2, currency: 'cny' as 'cny' | 'usd' | 'eur' | 'none', upper: false }
})

// 结果存储
const results = reactive<Record<string, string[]>>({
  name: [],
  idCard: [],
  phone: [],
  email: [],
  ip: [],
  url: [],
  address: [],
  bankCard: [],
  creditCode: [],
  vin: [],
  plate: [],
  mac: [],
  text: [],
  dateTime: [],
  zipCode: [],
  uuid: [],
  color: [],
  json: [],
  englishName: [],
  coordinate: [],
  company: [],
  profile: [],
  username: [],
  courier: [],
  amount: []
})

// 金额卡片原始数值（用于大写开关切换时实时重算格式，不随刷新丢失）
const amountValues = reactive<number[]>([])

// 切换金额"大写"开关时，已有结果实时切换大小写形式（无需重新生成）
watch(() => options.amount.upper, () => {
  if (!amountValues.length) return
  results.amount = amountValues.map(v => formatAmountValue(v, options.amount))
})

// 生成单个类型（countOverride 用于"全部生成"时按顶部数量覆盖，不修改卡片独立配置）
const handleGenerate = (type: string, countOverride?: number, silent = false) => {
  const opts = (options as any)[type]
  const useOpts = countOverride != null ? { ...opts, count: countOverride } : opts

  switch (type) {
    case 'name':
      results.name = generateName(useOpts)
      break
    case 'idCard':
      results.idCard = generateIdCard(useOpts)
      break
    case 'phone':
      results.phone = generatePhone(useOpts)
      break
    case 'email':
      results.email = generateEmail(useOpts)
      break
    case 'ip':
      results.ip = generateIP(useOpts)
      break
    case 'url':
      results.url = generateURL(useOpts)
      break
    case 'address':
      results.address = generateAddress(useOpts)
      break
    case 'bankCard':
      results.bankCard = generateBankCard(useOpts)
      break
    case 'creditCode':
      results.creditCode = generateCreditCode(useOpts)
      break
    case 'vin':
      results.vin = generateVIN(useOpts)
      break
    case 'plate':
      results.plate = generatePlate(useOpts)
      break
    case 'mac':
      results.mac = generateMAC(useOpts)
      break
    case 'text':
      results.text = generateText(useOpts)
      break
    case 'dateTime':
      results.dateTime = generateDateTime(useOpts)
      break
    case 'zipCode':
      results.zipCode = generateZipCode(useOpts)
      break
    case 'uuid':
      results.uuid = generateUUID(useOpts)
      break
    case 'color':
      results.color = generateColor(useOpts)
      break
    case 'json':
      results.json = generateJSON(useOpts)
      break
    case 'englishName':
      results.englishName = generateEnglishName(useOpts)
      break
    case 'coordinate':
      results.coordinate = generateCoordinate(useOpts)
      break
    case 'company':
      results.company = generateCompanyName(useOpts)
      break
    case 'profile':
      results.profile = generatePersonalProfile(useOpts)
      break
    case 'username':
      results.username = generateUsername(useOpts)
      break
    case 'courier':
      results.courier = generateCourierNumber(useOpts)
      break
    case 'amount': {
      // 保留原始数值，切换大写开关时可直接重算格式
      const values = generateAmountValues(useOpts)
      amountValues.length = 0
      amountValues.push(...values)
      results.amount = values.map(v => formatAmountValue(v, useOpts))
      break
    }
  }

  store.addHistory({
    tool: 'mockData',
    action: `generate_${type}`,
    inputPreview: `count=${useOpts.count}`,
    outputPreview: results[type][0] || '',
    inputFull: JSON.stringify(useOpts),
    outputFull: results[type].join('\n'),
  })

  if (!silent) ElMessage.success(`已生成 ${useOpts.count} 条数据`)
}

// 全部生成数量（独立于各卡片配置）
const allCount = ref(5)

// 全部生成（按顶部数量统一生成，仅提示一次）
const handleGenerateAll = () => {
  Object.keys(options).forEach(type => {
    handleGenerate(type, allCount.value, true)
  })
  ElMessage.success(`已生成 ${Object.keys(options).length} 种类型，每类 ${allCount.value} 条`)
}

// 清除全部
const handleClearAll = () => {
  Object.keys(results).forEach(key => {
    results[key] = []
  })
  amountValues.length = 0
  ElMessage.success('已清除全部数据')
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
</script>

<style scoped>
.tool-container {
  height: 100%; /* 与全局 main.css 一致，100vh 会导致底部超出视口被裁剪 */
  padding: 20px;
  background: var(--bg-primary);
}

/* 工具卡片 */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

/* 置顶卡片 */
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  margin-bottom: 16px;
}

/* 标题栏 */
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
.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 卡片内容 */
.card-body {
  padding: 16px 20px;
}

/* 提示图标 */
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

/* 网格布局 */
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
  gap: 12px;
  margin-bottom: 12px;
}
.option-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

/* 结果列表 */
.result-list {
  max-height: 200px;
  overflow-y: auto;
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

/* 滚动条 */
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

/* 随机文本样式 */
.text-item {
  align-items: flex-start;
}
.text-content {
  font-family: inherit;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

/* 颜色值样式 */
.color-item {
  align-items: center;
}
.color-preview {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  flex-shrink: 0;
}

/* JSON 样式 */
.json-list {
  max-height: 300px;
}
.json-item {
  align-items: flex-start;
}
.json-content {
  white-space: pre;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
}
</style>
