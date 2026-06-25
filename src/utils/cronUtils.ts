// src/utils/cronUtils.ts

// Cron表达式字段类型
export interface CronFields {
  second?: Set<number>  // 0-59 (6字段时启用)
  minute: Set<number>   // 0-59
  hour: Set<number>     // 0-23
  day: Set<number>      // 1-31
  month: Set<number>    // 1-12
  weekday: Set<number>  // 0-6 (0=周日)
}

// 将Cron字段转换为表达式字符串
export function buildCronExpression(fields: CronFields, isSixField: boolean = false): string {
  const { minute, hour, day, month, weekday } = fields;
  let result = '';

  // 如果是6字段，添加秒字段
  if (isSixField && fields.second) {
    result += setCronPart(fields.second, 0, 59) + ' ';
  }

  // 添加其他字段
  result += [
    setCronPart(minute, 0, 59),
    setCronPart(hour, 0, 23),
    setCronPart(day, 1, 31),
    setCronPart(month, 1, 12),
    setCronPart(weekday, 0, 6)
  ].join(' ');

  return result.trim();
}

// 将Set<number>转换为最优Cron语法
function setCronPart(values: Set<number>, min: number, max: number): string {
  // 检查是否全选
  if (values.size === (max - min + 1)) {
    return '*';
  }

  // 检查是否为步长模式（如*/5）
  const sortedValues = Array.from(values).sort((a, b) => a - b);
  if (sortedValues.length > 1) {
    // 检查是否是等差数列
    const diff = sortedValues[1] - sortedValues[0];
    let isArithmetic = true;
    for (let i = 2; i < sortedValues.length; i++) {
      if (sortedValues[i] - sortedValues[i - 1] !== diff) {
        isArithmetic = false;
        break;
      }
    }

    // 如果是等差数列，检查是否从最小值开始
    if (isArithmetic && sortedValues[0] === min && diff > 1) {
      return `*/${diff}`;
    }
  }

  // 检查是否为连续范围
  if (sortedValues.length > 1 && sortedValues[sortedValues.length - 1] - sortedValues[0] === sortedValues.length - 1) {
    return `${sortedValues[0]}-${sortedValues[sortedValues.length - 1]}`;
  }

  // 返回逗号分隔的值
  return sortedValues.join(',');
}

// 解析Cron表达式为CronFields
export function parseCronExpression(expression: string, isSixField: boolean = false): CronFields | null {
  // 清理表达式，移除多余空格
  const parts = expression.trim().split(/\s+/);
  
  // 验证字段数量
  if (parts.length !== (isSixField ? 6 : 5)) {
    return null;
  }

  let partIndex = 0;
  const fields: CronFields = {
    minute: new Set<number>(),
    hour: new Set<number>(),
    day: new Set<number>(),
    month: new Set<number>(),
    weekday: new Set<number>()
  };

  // 如果是6字段，先处理秒字段
  if (isSixField) {
    const secondResult = parseCronPart(parts[partIndex++], 0, 59);
    if (!secondResult) return null; // 解析失败
    fields.second = secondResult;
  }

  // 解析其他字段
  const minuteResult = parseCronPart(parts[partIndex++], 0, 59);
  const hourResult = parseCronPart(parts[partIndex++], 0, 23);
  const dayResult = parseCronPart(parts[partIndex++], 1, 31);
  const monthResult = parseCronPart(parts[partIndex++], 1, 12);
  const weekdayResult = parseCronPart(parts[partIndex++], 0, 6);

  // 验证解析结果
  if (!minuteResult || !hourResult || !dayResult || !monthResult || !weekdayResult) {
    return null;
  }

  fields.minute = minuteResult;
  fields.hour = hourResult;
  fields.day = dayResult;
  fields.month = monthResult;
  fields.weekday = weekdayResult;

  return fields;
}

// 解析单个Cron字段为Set<number>
function parseCronPart(part: string, min: number, max: number): Set<number> | null {
  const values = new Set<number>();

  // 处理多种情况：*, */n, n, n-m, n,m, n-m/n
  const segments = part.split(',');
  for (const segment of segments) {
    if (segment === '*') {
      // 全选
      for (let i = min; i <= max; i++) {
        values.add(i);
      }
    } else if (segment.includes('/')) {
      // 步长模式
      const [range, stepStr] = segment.split('/');
      const step = parseInt(stepStr);
      if (isNaN(step) || step <= 0) return null;

      if (range === '*') {
        // */n 形式
        for (let i = min; i <= max; i += step) {
          values.add(i);
        }
      } else if (range.includes('-')) {
        // n-m/n 形式
        const [startStr, endStr] = range.split('-');
        const start = parseInt(startStr);
        const end = parseInt(endStr);
        if (isNaN(start) || isNaN(end) || start < min || end > max || start > end) return null;

        for (let i = start; i <= end; i += step) {
          values.add(i);
        }
      } else {
        return null; // 无效格式
      }
    } else if (segment.includes('-')) {
      // 范围模式 n-m
      const [startStr, endStr] = segment.split('-');
      const start = parseInt(startStr);
      const end = parseInt(endStr);
      if (isNaN(start) || isNaN(end) || start < min || end > max || start > end) return null;

      for (let i = start; i <= end; i++) {
        values.add(i);
      }
    } else {
      // 单个数字
      const num = parseInt(segment);
      if (isNaN(num) || num < min || num > max) return null;
      values.add(num);
    }
  }

  return values;
}

// 验证Cron表达式合法性
export function validateCronExpression(expression: string, isSixField: boolean = false): { isValid: boolean; error?: string } {
  if (!expression.trim()) {
    return { isValid: false, error: '表达式不能为空' };
  }

  const fields = parseCronExpression(expression, isSixField);
  if (!fields) {
    return { isValid: false, error: '表达式格式不正确' };
  }

  // 额外检查：日和周不能同时为*（标准Cron中，如果两者都为*则匹配所有日期；如果一方指定了值，另一方必须为?或*）
  // 注意：在标准5字段Cron中，日和周同时为*是合法的（表示"匹配任意日期且匹配任意星期"）
  // 这里仅检查明显错误的情况：如果用户手动指定了日和周的具体值且两者冲突

  return { isValid: true };
}

// 计算下次执行时间
export function getNextExecutionTime(expression: string, isSixField: boolean = false, startDate: Date = new Date()): Date | null {
  const fields = parseCronExpression(expression, isSixField);
  if (!fields) {
    return null;
  }

  // 从给定时间的下一分钟开始搜索
  const now = new Date(startDate);
  now.setSeconds(now.getSeconds() + 1);
  if (isSixField && fields.second) {
    now.setMilliseconds(0);
  } else {
    now.setSeconds(0, 0);
  }

  // 限制搜索范围为1年，防止无限循环
  const maxDate = new Date(now);
  maxDate.setFullYear(maxDate.getFullYear() + 1);

  // 逐步增加时间直到找到匹配
  while (now < maxDate) {
    // 检查是否匹配当前时间
    if (isCronMatch(now, fields, isSixField)) {
      return new Date(now);
    }

    // 增加时间单位
    if (isSixField) {
      now.setSeconds(now.getSeconds() + 1);
    } else {
      now.setMinutes(now.getMinutes() + 1);
    }
  }

  // 一年内都没有匹配，返回null
  return null;
}

// 计算多次执行时间
export function getNextExecutionTimes(expression: string, isSixField: boolean = false, startDate: Date = new Date(), count: number = 5): Date[] {
  const results: Date[] = [];
  let currentDate = new Date(startDate);
  
  for (let i = 0; i < count; i++) {
    const nextTime = getNextExecutionTime(expression, isSixField, currentDate);
    if (!nextTime) break;
    results.push(nextTime);
    currentDate = new Date(nextTime);
    // 增加1秒以搜索下一次
    if (isSixField) {
      currentDate.setSeconds(currentDate.getSeconds() + 1);
    } else {
      currentDate.setMinutes(currentDate.getMinutes() + 1);
    }
  }
  
  return results;
}

// 检查给定时间是否匹配Cron表达式
function isCronMatch(date: Date, fields: CronFields, isSixField: boolean): boolean {
  // 检查各个字段
  if (isSixField && fields.second && !fields.second.has(date.getSeconds())) {
    return false;
  }
  
  if (!fields.minute.has(date.getMinutes())) {
    return false;
  }
  
  if (!fields.hour.has(date.getHours())) {
    return false;
  }
  
  // 日期和星期的处理需要特殊考虑
  // 如果日期和星期都是"*"，则表示每天
  // 否则，如果其中一个为"*"，则另一个生效
  // 如果都不是"*"，则两个都要满足（OR关系）
  const isDayMatch = fields.day.has(date.getDate());
  const isWeekdayMatch = fields.weekday.has(date.getDay());
  
  // 如果日期和星期都不全是*，则需要满足其中之一
  if (!(isAllDays(fields.day) && isAllWeekdays(fields.weekday))) {
    if (!isAllDays(fields.day) && !isAllWeekdays(fields.weekday)) {
      // 如果都不是*，则只需满足其中一个
      if (!isDayMatch && !isWeekdayMatch) {
        return false;
      }
    } else if (!isAllDays(fields.day)) {
      // 如果只有星期是*，则日期需要匹配
      if (!isDayMatch) {
        return false;
      }
    } else if (!isAllWeekdays(fields.weekday)) {
      // 如果只有日期是*，则星期需要匹配
      if (!isWeekdayMatch) {
        return false;
      }
    }
  }
  
  if (!fields.month.has(date.getMonth() + 1)) {
    return false;
  }
  
  return true;
}

// 辅助函数：检查是否是全选
function isAllDays(days: Set<number>): boolean {
  return days.size === 31 && Array.from({length: 31}, (_, i) => i+1).every(d => days.has(d));
}

function isAllWeekdays(weekdays: Set<number>): boolean {
  return weekdays.size === 7 && Array.from({length: 7}, (_, i) => i).every(w => weekdays.has(w));
}
