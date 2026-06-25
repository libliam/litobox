export function generateUUID(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0
    const v = c === 'x' ? r : (r & 0x3) | 0x8
    return v.toString(16)
  })
}

export interface UUIDOptions {
  count: number
  uppercase: boolean
  removeDashes: boolean
}

export function generateUUIDs(options: UUIDOptions = { count: 1, uppercase: false, removeDashes: false }): string[] {
  const uuids: string[] = []
  for (let i = 0; i < options.count; i++) {
    let uuid = generateUUID()
    if (options.uppercase) {
      uuid = uuid.toUpperCase()
    }
    if (options.removeDashes) {
      uuid = uuid.replace(/-/g, '')
    }
    uuids.push(uuid)
  }
  return uuids
}