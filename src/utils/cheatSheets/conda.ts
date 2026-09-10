import type { CheatSheetData } from './index'

export const condaCommands: CheatSheetData = {
  id: 'conda',
  name: 'Conda 命令',
  description: 'Conda 环境/包管理速查，含环境创建激活、包安装、Channel 配置、镜像源、导出/迁移',
  columns: [
    { key: 'command', label: '命令', width: '260px', copyable: true },
    { key: 'category', label: '分类', width: '110px' },
    { key: 'params', label: '常用参数', width: '230px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // === 环境管理 ===
    { command: 'conda create [选项] -n 名称 [包=版本]', category: '环境管理', params: '-n / --name 环境名; -p / --prefix 指定路径; -c 指定channel; --clone 克隆环境; -y 自动确认; --dry-run 预览', description: '创建新环境（可同时装包）', examples: ['conda create -n py311 python=3.11', 'conda create -n torch python=3.10 pytorch=2.0 cudatoolkit=11.8', 'conda create -n myenv --clone base', 'conda create -p /opt/env customenv python=3.9'] },
    { command: 'conda env create -f environment.yml', category: '环境管理', params: '-f yml文件; -n 覆盖yml中环境名; -p 指定前缀', description: '从 yml 文件创建环境（团队协作标准）', examples: ['conda env create -f environment.yml', 'conda env create -f env-prod.yml -n production'] },
    { command: 'conda activate / conda deactivate', category: '环境管理', params: 'conda activate env_name 激活; conda deactivate 退出(可叠退); source activate/dactivate 旧写法', description: '激活/退出环境', examples: ['conda activate py311', 'conda deactivate  # 退回base', 'conda deactivate  # 再执行一次可退出base'] },
    { command: 'conda env list / conda info --envs', category: '环境管理', params: '查看所有环境; 当前环境前有 *', description: '列出所有环境', examples: ['conda env list', 'conda info --envs'] },
    { command: 'conda env remove -n 名称', category: '环境管理', params: '-n 删指定; -p 按路径删; -y 自动确认', description: '删除环境（谨慎）', examples: ['conda env remove -n old_env', 'conda remove --name old_env --all'] },
    { command: 'conda env rename 旧名 新名', category: '环境管理', params: '重命名环境（或 conda rename）', description: '重命名环境', examples: ['conda env rename temp_env new_env', 'conda rename temp_env new_env'] },
    { command: 'conda env export > environment.yml', category: '环境管理', params: '--no-builds 去掉构建号; --from-history 仅包含手动安装的包(推荐)', description: '导出环境配置（便于迁移）', examples: ['conda env export > environment.yml', 'conda env export --from-history > environment.yml  # 干净版', 'conda env export -n prod > prod.yml'] },
    { command: 'conda-pack 环境打包', category: '环境管理', params: 'conda安装: conda install conda-forge::conda-pack; conda-pack -n env -o env.tar.gz; 解压后 source bin/activate 或 conda-unpack', description: '把环境整体打包成 tar.gz（跨机器迁移）', examples: ['conda pack -n py311 -o py311.tar.gz', 'tar -xzf py311.tar.gz -C /opt/env && cd /opt/env && conda-unpack'] },
    { command: 'conda update -n base -c defaults conda', category: '环境管理', params: '-n 指定环境; -c channel', description: '更新 conda 自身到最新版', examples: ['conda update -n base -c defaults conda', 'conda update conda  # 当前环境'] },
    // === 包管理 ===
    { command: 'conda install [选项] 包[=版本]', category: '包管理', params: '-n / -p 环境; -c channel; -y 自动确认; --dry-run 预览; --no-update-deps 不更新依赖', description: '安装包', examples: ['conda install numpy', 'conda install numpy=1.24 pandas=2.0', 'conda install -n py311 numpy', 'conda install -c conda-forge ffmpeg'] },
    { command: 'conda search 包', category: '包管理', params: '-c 指定channel; -i 大小写不敏感; --info 详细; package * 版本列表', description: '搜索可用包', examples: ['conda search numpy', 'conda search "pytorch=2.0*" -c pytorch', 'conda search "python=3.1*"'] },
    { command: 'conda list [-n env]', category: '包管理', params: '-n / -p 指定环境; -e 精确版本列表; 无参查看当前环境', description: '列出环境内已安装包', examples: ['conda list', 'conda list -n py311', 'conda list -e > requirements.txt  # 纯版本列表'] },
    { command: 'conda update 包 / conda update --all', category: '包管理', params: '--all 更新全部; -n 环境; -y 确认; --dry-run 预览', description: '更新包', examples: ['conda update numpy', 'conda update --all  # 更新所有包（谨慎，可能破坏兼容）', 'conda update -n base mamba'] },
    { command: 'conda remove [选项] 包', category: '包管理', params: '-n / -p; -y; --force-pkgs-dirs 强制删包目录; --all 删除环境内全部包', description: '卸载包', examples: ['conda remove numpy', 'conda remove -n py311 pandas matplotlib', 'conda remove --all -n old_env'] },
    { command: 'conda install pip + pip install', category: '包管理', params: 'conda里装pip后可以混用; conda-forge装conda-forge::pip', description: '用 pip 补充 conda 没有的包', examples: ['conda install pip && pip install requests', 'conda install -n py311 pip', '# 建议优先 conda，pip 装 conda 没有的\nconda install numpy\npip install some-weird-pkg'] },
    { command: 'conda verify [包名]', category: '包管理', params: '--all 验证所有; -n 环境', description: '校验包完整性（损坏检测）', examples: ['conda verify --all', 'conda verify numpy'] },
    // === Channel/镜像源 ===
    { command: 'conda config --show channels', category: '配置', params: '--add / --remove channel; --show; --show-sources 配置文件位置; --set key value', description: '配置 Channel（软件源）', examples: ['conda config --add channels conda-forge', 'conda config --remove channels defaults', 'conda config --show', 'conda config --show-sources'] },
    { command: '配置国内镜像源（清华）', category: '配置', params: '--add channels + --set show_channel_urls yes', description: '加速下载（必配）', examples: ["conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main", "conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/free", "conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge", "conda config --set show_channel_urls yes"] },
    { command: 'conda config --set channel_priority strict', category: '配置', params: 'strict / flexible 默认; strict 优先用先添加的 channel', description: '设置 channel 优先级', examples: ['conda config --set channel_priority strict  # 防止多channel混装冲突', 'conda config --set channel_priority flexible'] },
    { command: 'conda config --set solver classic / libmamba', category: '配置', params: 'classic 默认求解器; libmamba 更快(新版conda默认); experimental=libmamba_batched 批处理', description: '选择求解器（新版推荐 libmamba）', examples: ['conda config --set solver libmamba  # 大幅加速求解', 'conda install -n base -c conda-forge mamba  # 或者用更快的 mamba 替代'] },
    // === Mamba 加速 ===
    { command: 'mamba [命令] 替代 conda', category: '配置', params: 'mamba create/install/update/env... 用法完全同conda; mamba=conda用libmamba解算器，快数倍', description: '超加速版 conda（需要单独装）', examples: ['mamba create -n py311 python=3.11 numpy', 'mamba install -n torch pytorch cudatoolkit', 'mamba env export > env.yml'] },
    // === 清理维护 ===
    { command: 'conda clean --all / --index-pkgs / --tarballs', category: '清理', params: '--all 清一切; -i index缓存; -p 包缓存; -t 下载的tarballs; -y 自动确认; -d dry-run预览', description: '清理缓存释放空间（Conda 易涨快）', examples: ['conda clean -i -p -t -y  # 常用组合', 'conda clean --all -y  # 清理所有（最大释放）', 'du -sh ~/miniconda3/pkgs  # 看缓存占多少'] },
    { command: 'conda list | grep "^#" 失效包', category: '清理', params: 'conda clean --packages --tarballs 删孤立包; conda list 看哪些前缀 #', description: '检查/清理孤立包（不再被任何环境引用）', examples: ['conda list | grep "^#" | head -5', 'conda clean --packages --tarballs'] },
    // === .condarc 配置文件 ===
    { command: '~/.condarc（用户级） / .condarc（项目级）', category: '配置', params: 'channels: 列表; show_channel_urls: true; default_channels; custom_channels; envs_dirs; pkgs_dirs; proxy_servers', description: '手动编辑配置文件', examples: ['# ~/.condarc 示例\nchannels:\n  - https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main\n  - https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge\n  - defaults\nshow_channel_urls: true\nchannel_priority: strict', 'conda config --show-sources  # 查看当前配置文件路径'] },
    // === 其他 ===
    { command: 'conda info', category: '其他', params: '-e 环境列表; --base 基环境路径; -n env 环境信息', description: '查看 conda 环境信息', examples: ['conda info', 'conda info -e'] },
    { command: 'conda run -n env python script.py', category: '其他', params: '-n 指定环境执行命令; -b 不激活输出; -v 详细; --no-capture-output 显示stderr', description: '不激活直接在某环境下运行命令（CI场景）', examples: ['conda run -n py311 python train.py', 'conda run -n lint pylint src/', 'conda run -n test pytest -v'] },
    { command: 'conda create -n env python=3.11 -y 快速新建', category: '其他', params: '标准快速创建模板', description: '一键创建 Python 环境', examples: ['conda create -n py311 python=3.11 -y && conda activate py311', 'conda create -n data python=3.10 numpy pandas scikit-learn matplotlib jupyter -y'] },
  ],
}
