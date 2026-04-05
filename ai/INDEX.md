# CloverViewer AI 辅助开发文档索引

本目录包含供 AI 助手（Claude Code）使用的开发文档，旨在提升代码生成质量、保持项目一致性。

## 📑 文档清单

| 文档 | 用途 | 优先级 |
|------|------|--------|
| [CLAUDE.md](CLAUDE.md) | **必读** - 项目概览、架构说明、常见任务指引 | P0 |
| [coding-standards.md](coding-standards.md) | 编码规范、命名约定、避坑指南 | P1 |
| [egui-patterns.md](egui-patterns.md) | egui 框架使用模式、GUI 编程最佳实践 | P1 |
| [skills.md](skills.md) | 可使用的自定义 Skill 命令列表 | P2 |
| [architecture.md](architecture.md) | 架构决策记录、技术选型说明 | P2 |

## 🚀 快速开始

**如果你是第一次使用本项目的 AI 助手：**
1. 首先阅读 [CLAUDE.md](CLAUDE.md) 了解项目全貌
2. 查看 [coding-standards.md](coding-standards.md) 了解代码风格要求
3. 根据任务类型，参考对应的专项文档

**如果你要执行特定任务：**
- 添加新功能 → 查看 [skills.md](skills.md) 中的 `/add-feature` skill
- 修复 Bug → 查看 [skills.md](skills.md) 中的 `/fix-bug` skill
- 修改 UI → 参考 [egui-patterns.md](egui-patterns.md)
- 添加设置项 → 查看 [skills.md](skills.md) 中的 `/add-setting` skill

## 📝 文档维护

- 当项目技术栈或架构发生重大变化时，请及时更新相关文档
- 当发现 AI 经常犯某种错误时，将避坑指南添加到 [coding-standards.md](coding-standards.md)
- 当新增可自动化的任务模式时，在 [skills.md](skills.md) 中添加新的 skill 定义

---

*最后更新: 2026-04-05*
