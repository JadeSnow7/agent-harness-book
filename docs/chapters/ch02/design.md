# ch2 设计边界：一次模型调用

本章新增 `tutorial/agent-harness` Rust 累计工程，目标是完成一次非流式 Responses API 调用。它把配置、Provider 请求构造、HTTP Transport 和响应解析分开，但仍停留在 M0。

## 保留的边界

- `Config` 从环境读取 API Key、模型、可选 Base URL 和超时；错误不回显密钥。
- `build_request` 只生成 Provider JSON 的 `model` 与 `input`。
- `Transport` 是测试缝；默认测试使用 Fake Transport，不访问网络。
- `ReqwestTransport` 仅用于手动真实请求。
- 解析器遍历 `output/content`，跳过 reasoning、未知项目和空文本，按顺序合并 `output_text`。

本章不引入统一协议、工具、会话、重试、流式响应或 Agent Loop。Provider JSON 仍停留在边界内；统一的 `Message` 和 `ContentBlock` 留给下一章。
