# 实验：M2 Tool Runtime 七工具

第 4 章只使用 `read` 完成连续案例。本实验保留 M2 的完整工具目录，供需要深入实现细节的读者查阅。

## 工具目录

| 类别 | 工具 | 关键约束 |
|---|---|---|
| 观察 | `read` | 有界读取，输出带行号 |
| 观察 | `ls`、`find` | 相对路径、排序和 limit |
| 观察 | `grep` | 正则与显式字面量两种模式 |
| 修改 | `write` | 同目录临时文件和原子替换 |
| 修改 | `edit` | 原文中唯一、非重叠的精确替换 |
| 进程 | `bash` | 参数化进程，`shell=False`，默认关闭 |

完整 Python 实现位于 `examples/python/m2-tool-runtime/`，Rust 同构实现位于 `examples/rust/m2-tool-runtime/`。本实验不把 allowlist 当作 OS sandbox，也不把局部 postcondition 当作任务级 Validation。

## Workspace 与失败路径

Workspace 约束已存在路径、写入目标的现有父目录、`..`、绝对路径和 symlink 逃逸。写入失败时原文件必须保持不变。工具层还必须覆盖未知工具、非法参数、非法正则、非零退出、超时和输出截断。

## 验证

```bash
python3 -m py_compile \
  examples/python/m2-tool-runtime/*.py \
  examples/python/m2-tool-runtime/tools/*.py
python3 -m unittest discover \
  -s examples/python/m2-tool-runtime \
  -p 'test_*.py'
cargo test -p m2-tool-runtime
```

这些测试默认不访问网络，不启动危险进程，也不需要真实 API Key。
