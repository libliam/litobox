import type { CheatSheetData } from './index'

export const packageManagerCheatsheet: CheatSheetData = {
  id: 'packageManager',
  name: 'NPM / Yarn / Pnpm',
  description: '三大前端包管理器命令对照速查，npm 为基准，左侧 npm 命令，右侧 Yarn / Pnpm 对应写法',
  columns: [
    { key: 'npm', label: 'npm', width: '200px', copyable: true },
    { key: 'yarn', label: 'yarn', width: '180px', copyable: true },
    { key: 'pnpm', label: 'pnpm', width: '200px', copyable: true },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // === 项目初始化 ===
    { npm: 'npm init -y', yarn: 'yarn init -y', pnpm: 'pnpm init', description: '初始化 package.json（-y 跳过提问）', examples: ['npm init -y  # 最常用', 'yarn init -y', 'pnpm init'] },
    { npm: 'npm create vite@latest my-app', yarn: 'yarn create vite my-app', pnpm: 'pnpm create vite my-app', description: '创建项目脚手架（create 命令三系通用）', examples: ['npm create vite@latest my-app -- --template vue', 'pnpm create vite my-app --template react', 'yarn create next-app my-app'] },
    // === 安装依赖 ===
    { npm: 'npm install', yarn: 'yarn', pnpm: 'pnpm install', description: '安装全部依赖（项目根目录）', examples: ['npm install', 'yarn  # yarn install 简写', 'pnpm install  # 或 pnpm i'] },
    { npm: 'npm install 包名', yarn: 'yarn add 包名', pnpm: 'pnpm add 包名', description: '安装包到 dependencies（生产依赖）', examples: ['npm install axios', 'yarn add axios', 'pnpm add axios vue-router@4'] },
    { npm: 'npm install -D 包名', yarn: 'yarn add -D 包名', pnpm: 'pnpm add -D 包名', description: '安装到 devDependencies（开发依赖）', examples: ['npm install -D typescript eslint vite', 'yarn add -D jest', 'pnpm add -D tailwindcss postcss autoprefixer'] },
    { npm: 'npm install -g 包名', yarn: 'yarn global add 包名', pnpm: 'pnpm add -g 包名', description: '全局安装', examples: ['npm install -g typescript', 'yarn global add vue-cli', 'pnpm add -g pnpm@latest  # 升级自己'] },
    { npm: 'npm install 包名@版本', yarn: 'yarn add 包名@版本', pnpm: 'pnpm add 包名@版本', description: '指定版本安装', examples: ['npm install vue@3.4.21', 'yarn add react@^18.2.0', 'pnpm add lodash@4.17.21'] },
    // === 升级/卸载 ===
    { npm: 'npm update [包名]', yarn: 'yarn upgrade [包名]', pnpm: 'pnpm update [包名]', description: '升级包（pnpm 默认只改 package.json 中指定范围的最新版本）', examples: ['npm update', 'yarn upgrade lodash', 'pnpm update vue  # pnpm up 简写'] },
    { npm: 'npm install 包名@latest', yarn: 'yarn upgrade-latest 包名', pnpm: 'pnpm add 包名@latest', description: '强制升级到最新（忽略版本范围）', examples: ['npm install vue@latest', 'pnpm add vue@latest  # pnpm 的做法更简洁'] },
    { npm: 'npm uninstall 包名', yarn: 'yarn remove 包名', pnpm: 'pnpm remove 包名', description: '卸载并从 package.json 移除', examples: ['npm uninstall axios', 'yarn remove old-lib', 'pnpm remove lodash'] },
    { npm: 'npm uninstall -g 包名', yarn: 'yarn global remove 包名', pnpm: 'pnpm remove -g 包名', description: '卸载全局', examples: ['npm uninstall -g typescript', 'pnpm remove -g pnpm@8'] },
    // === 执行脚本 ===
    { npm: 'npm run script [args]', yarn: 'yarn script [args]', pnpm: 'pnpm script [args]', description: '执行 package.json 中的脚本', examples: ['npm run dev', 'yarn dev', 'pnpm build', 'pnpm run test -- --watch'] },
    { npm: 'npx 命令', yarn: 'yarn 命令', pnpm: 'pnpm exec 命令', description: '执行项目内安装的 CLI（npx 是 npm 5.2+，yarn 直接调用）', examples: ['npx webpack', 'yarn prettier --write .', 'pnpm exec eslint src/'] },
    { npm: 'npm run', yarn: 'yarn run', pnpm: 'pnpm run', description: '查看所有可用脚本', examples: ['npm run  # 列出脚本清单', 'pnpm run'] },
    // === 检查/缓存 ===
    { npm: 'npm outdated', yarn: 'yarn outdated', pnpm: 'pnpm outdated', description: '检查过期依赖', examples: ['npm outdated', 'yarn outdated', 'pnpm outdated --json  # 输出JSON'] },
    { npm: 'npm audit', yarn: 'yarn audit', pnpm: 'pnpm audit', description: '安全漏洞检查', examples: ['npm audit', 'npm audit fix  # 自动修复低危漏洞', 'pnpm audit'] },
    { npm: 'npm cache clean --force', yarn: 'yarn cache clean', pnpm: 'pnpm store prune', description: '清理缓存（pnpm 是 store，磁盘节省明显）', examples: ['npm cache clean --force', 'pnpm store prune  # 清理未被任何项目引用的包', 'du -sh ~/.pnpm-store  # 看 pnpm 缓存大小'] },
    { npm: 'npm ls [包名]', yarn: 'yarn list [包名]', pnpm: 'pnpm ls [包名]', description: '查看已安装包（pnpm 显示依赖树最清晰）', examples: ['npm ls vue', 'pnpm ls --depth 0  # 只看直接依赖', 'pnpm why lodash  # 查看为什么装了这个包'] },
    // === 锁文件 ===
    { npm: 'package-lock.json', yarn: 'yarn.lock', pnpm: 'pnpm-lock.yaml', description: '锁文件（固定依赖树，提交到 git 保证一致安装）', examples: ['# 三个锁文件不要混用，gitignore 只保留自己项目用的\n# 建议 .npmrc 中 packageManager="pnpm@8"'] },
    // === pnpm 独有特性 ===
    { npm: '—', yarn: '—', pnpm: 'pnpm add @scope/pkg@1.x -D --filter', description: 'pnpm 独有：严格隔离 node_modules + workspace 命令 + content-addressable store（节省磁盘）', examples: ['pnpm why vue  # pnpm 专属：查看为什么安装了某包', 'pnpm dlx create-vite my-app  # pnpm dlx = npx', '# pnpm 硬链接/符号链接 node_modules，多个项目共享一份包，节省磁盘'] },
    // === workspace 多包管理 ===
    { npm: 'npm workspaces', yarn: 'yarn workspaces', pnpm: 'pnpm -r 子命令', description: 'monorepo 多包管理（pnpm 最成熟，yarn workspaces 1.x 有限）', examples: ['# root package.json\n{\n  "workspaces": ["packages/*", "apps/*"]\n}\n\n# pnpm 批量执行\npnpm -r build  # 所有包 build\npnpm -r --filter @myscope/app dev  # 过滤特定包', '# pnpm workspace 链接符号，monorepo 原生支持最好'] },
    // === 配置 ===
    { npm: '~/.npmrc', yarn: '~/.yarnrc.yml', pnpm: '~/.npmrc（同npm）', description: '用户级配置（registry、代理、认证等）', examples: ['# ~/.npmrc 镜像源\nregistry=https://registry.npmmirror.com/', 'npm config set registry https://registry.npmmirror.com/', '# pnpm 也用 ~/.npmrc\n# 项目根 .npmrc 优先级更高'] },
    { npm: 'npm config get/set key val', yarn: 'yarn config set key val', pnpm: 'pnpm config set key val', description: '设置配置项（registry 换源最常用）', examples: ['npm config set registry https://registry.npmmirror.com/', 'pnpm config set registry https://registry.npmmirror.com/', 'yarn config set registry https://registry.npmmirror.com/'] },
  ],
}
