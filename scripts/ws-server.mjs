// WebSocket 测试服务端
// 用法: node scripts/ws-server.mjs
// 然后在 LitoBox 的 WebSocket 工具里连接 ws://localhost:8080
import { WebSocketServer } from 'ws'

const PORT = process.argv[2] || 8080
const wss = new WebSocketServer({ port: PORT })

let clientId = 0
const clients = new Map()

console.log(`🚀 WebSocket 测试服务端启动: ws://localhost:${PORT}`)
console.log('   Ctrl+C 停止\n')

wss.on('connection', (ws) => {
  const id = ++clientId
  clients.set(id, ws)
  console.log(`[连接] 客户端 #${id} (当前 ${clients.size} 个)`)

  // 欢迎消息
  ws.send(JSON.stringify({
    type: 'welcome',
    msg: `你好，你是第 ${id} 号客户端`,
    serverTime: new Date().toISOString()
  }))

  ws.on('message', (raw) => {
    const text = raw.toString()
    console.log(`[收到 #${id}] ${text}`)

    // 尝试解析 JSON 并处理指令
    try {
      const msg = JSON.parse(text)
      handleCommand(id, ws, msg)
    } catch {
      // 非 JSON 原样回显
      ws.send(JSON.stringify({ type: 'echo', data: text, from: id }))
    }
  })

  ws.on('close', () => {
    clients.delete(id)
    console.log(`[断开] 客户端 #${id} (剩余 ${clients.size} 个)`)
  })

  ws.on('error', (err) => {
    console.log(`[错误 #${id}] ${err.message}`)
  })
})

// ============ 指令处理 ============
function handleCommand(id, ws, msg) {
  switch (msg.type) {
    case 'ping':
      ws.send(JSON.stringify({ type: 'pong', t: Date.now() }))
      break

    case 'broadcast':
      // 广播给所有客户端
      const payload = JSON.stringify({ type: 'broadcast', from: id, data: msg.data })
      for (const [, c] of clients) {
        if (c.readyState === 1) c.send(payload)
      }
      console.log(`[广播 #${id}] ${msg.data}`)
      break

    case 'subscribe':
      ws.send(JSON.stringify({ type: 'subscribed', channel: msg.channel || 'default' }))
      break

    case 'time':
      ws.send(JSON.stringify({ type: 'time', serverTime: new Date().toISOString() }))
      break

    default:
      ws.send(JSON.stringify({ type: 'unknown', received: msg }))
  }
}

// ============ 定时推送（每 5 秒向所有客户端广播服务器时间） ============
setInterval(() => {
  if (clients.size === 0) return
  const payload = JSON.stringify({
    type: 'tick',
    serverTime: new Date().toISOString(),
    clients: clients.size
  })
  for (const [, c] of clients) {
    if (c.readyState === 1) c.send(payload)
  }
}, 5000)

// 优雅退出
process.on('SIGINT', () => {
  console.log('\n👋 关闭中...')
  for (const [, c] of clients) c.close()
  wss.close()
  process.exit(0)
})
