<<<<<<< HEAD
# rust-crate-recommender
rust crate的智能推荐系统
=======
# Rust Crate Recommender

一个智能的 Rust crates 推荐系统，帮助开发者找到最适合他们需求的 Rust 库。

## 功能特点

- 基于自然语言的智能推荐
- 综合考虑多个因素（更新时间、下载量、评分等）
- 用户友好的 Web 界面
- 实时推荐结果

## 技术栈

- 后端：Rust + Axum
- 前端：React + TypeScript
- AI：OpenAI API
- 数据源：crates.io API

## 开发环境设置

1. 克隆仓库
```bash
git clone [repository-url]
cd rust-crate-recommender
```

2. 安装依赖
```bash
# 后端
cargo build

# 前端
cd frontend
npm install
```

3. 配置环境变量
创建 `.env` 文件并添加必要的环境变量：
```
OPENAI_API_KEY=your_api_key
```

4. 运行项目
```bash
# 后端
cargo run

# 前端
cd frontend
npm run dev
```

## 项目结构

```
rust-crate-recommender/
├── src/                 # Rust 后端代码
├── frontend/           # React 前端代码
├── Cargo.toml          # Rust 依赖配置
└── README.md           # 项目文档
```

## 贡献指南

欢迎提交 Pull Requests 和 Issues！

## 许可证

MIT 
>>>>>>> 0ba844e (first commit)
