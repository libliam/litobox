/**
 * 打包后重命名 exe：litobox.exe → litobox-v{version}.exe
 * 从 package.json 读取版本号，在 release/bundle 目录中查找并重命名
 */
const fs = require('fs');
const path = require('path');

const pkg = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../package.json'), 'utf-8'));
const version = pkg.version;
const exeDir = path.resolve(__dirname, '../src-tauri/target/release');

const srcExe = path.join(exeDir, 'litobox.exe');
const dstExe = path.join(exeDir, `litobox-${version}.exe`);

if (!fs.existsSync(srcExe)) {
  console.error(`❌ 未找到构建产物: ${srcExe}`);
  console.error('   请先执行 npm run tauri build');
  process.exit(1);
}

// 如果目标文件已存在则先删除（覆盖旧版本）
if (fs.existsSync(dstExe)) {
  fs.unlinkSync(dstExe);
}

fs.copyFileSync(srcExe, dstExe);
const sizeMB = (fs.statSync(dstExe).size / (1024 * 1024)).toFixed(2);
console.log(`✅ 已生成版本化 exe: litobox-v${version}.exe (${sizeMB} MB)`);
console.log(`   路径: ${dstExe}`);
