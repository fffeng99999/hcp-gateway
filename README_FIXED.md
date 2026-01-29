# HCP Gateway - 已修复版本

## 功能说明

这是一个 **Rust 编写的 API 网关**，用于服务 `hcp-ui` 前端。

### 核心特性

✅ **JSON 数据存储** - 所有样例数据存储在 `data/mock_data.json`  
✅ **灵活的 API** - 21 个 RESTful 端点  
✅ **并行处理** - TaskExecutor 支持并发任务  
✅ **微服务就绪** - 预留接口用于后期集成真实服务  

## 快速开始

### 1. 编译

```bash
cd hcp-gateway
cargo build --release
```

### 2. 运行

```bash
cargo run --release
```

服务将在 `http://127.0.0.1:8080` 启动

### 3. 测试

```bash
# 健康检查
curl http://localhost:8080/health

# 获取共识算法列表
curl http://localhost:8080/consensus/algorithms

# 获取交易历史
curl http://localhost:8080/transaction/history

# 获取节点列表
curl http://localhost:8080/node/list

# 获取性能报告
curl http://localhost:8080/analysis/report
```

## API 端点概览

### 健康检查
- `GET /health` - 服务健康状态

### 共识管理
- `GET /consensus/algorithms` - 获取支持的算法
- `GET /consensus/config` - 获取当前配置
- `POST /consensus/select` - 选择算法
- `PUT /consensus/parameters` - 更新参数
- `POST /consensus/benchmark/start` - 启动基准测试
- `GET /consensus/benchmark/:id` - 获取测试结果
- `POST /consensus/benchmark/:id/stop` - 停止测试
- `GET /consensus/benchmark/history` - 获取历史测试

### 交易管理
- `POST /transaction/submit` - 提交交易
- `GET /transaction/:id` - 获取交易详情
- `GET /transaction/status` - 获取交易统计
- `GET /transaction/history` - 获取交易历史

### 节点管理
- `GET /node/list` - 列出所有节点
- `GET /node/:id` - 获取节点详情
- `GET /node/stats` - 获取节点统计

### 性能指标
- `GET /performance/metrics` - 获取当前指标
- `GET /performance/history` - 获取历史数据
- `GET /performance/comparison` - 算法对比

### 分析报告
- `GET /analysis/report` - 生成性能报告
- `GET /analysis/trends` - 获取性能趋势

## 数据文件结构

所有样例数据存储在 `data/mock_data.json`，包含：

```json
{
  "algorithms": [...],         # 共识算法列表
  "benchmarks": [...],         # 基准测试结果
  "transactions": [...],       # 交易数据
  "nodes": [...],              # 网络节点
  "consensus_config": {...}    # 当前配置
}
```

## 开发工作流

### 查看日志

```bash
RUST_LOG=debug cargo run
```

### 修改样例数据

编辑 `data/mock_data.json` 文件，重启服务自动加载新数据。

### 添加新 API 端点

1. 在 `src/api/` 中创建相应模块
2. 在 `src/main.rs` 中注册路由
3. 服务自动支持该端点

## 与前端集成

前端可以直接调用：

```typescript
const response = await fetch('http://localhost:8080/consensus/algorithms');
const data = await response.json();
```

## 后期微服务集成

当需要集成真实微服务时：

1. 设置环境变量
   ```bash
   export CONSENSUS_SERVICE_URL="http://your-service:8081"
   ```

2. 更新 `src/services/` 中的相应模块实现实际的 HTTP 调用

3. 网关自动切换到真实数据源

## 项目结构

```
src/
├── main.rs           # 服务器入口
├── data.rs           # JSON 数据加载
├── state.rs          # 应用状态
├── error.rs          # 错误处理
├── config.rs         # 配置管理
├── models.rs         # 数据模型
├── middleware.rs     # 中间件
├── tasks.rs          # 并行任务处理
├── services.rs       # 微服务占位符
└── api/
    ├── mod.rs
    ├── health.rs
    ├── consensus.rs
    ├── transaction.rs
    ├── node.rs
    ├── performance.rs
    └── analysis.rs

data/
└── mock_data.json    # 样例数据
```

## 技术栈

- **框架**: Axum 0.7
- **运行时**: Tokio
- **序列化**: Serde/JSON
- **语言**: Rust 1.70+

## 故障排除

### 端口被占用

```bash
lsof -i :8080
kill -9 <PID>
```

### 编译错误

```bash
cargo clean
cargo build --release
```

### 数据未更新

确保 `data/mock_data.json` 存在且有效，重启服务。

## 许可证

Apache-2.0
