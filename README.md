<div id="top" align="center">

# 🚀 RabbitMQ Async

![RabbitMQ](https://img.shields.io/badge/RabbitMQ-FF6600?style=for-the-badge&logo=rabbitmq&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-000000?style=for-the-badge&logo=tokio&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-4169E1?style=for-the-badge&logo=postgresql&logoColor=white)

[![Rust CI](https://github.com/GregTMJ/rabbitmq-async-example/actions/workflows/project_check.yaml/badge.svg?branch=main)](https://github.com/GregTMJ/rabbitmq-async-example/actions/workflows/project_check.yaml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A high-performance asynchronous RabbitMQ client built with Rust and Tokio

</div>

## 📖 Overview

This project is a Rust-based clone of an original Python implementation, designed to explore asynchronous programming patterns in Rust while improving upon the original architecture. During development, several hidden bugs in the original design were uncovered and resolved, leading to enhanced logic and performance.

**🎯 Project Goals:**
- Learn and implement async Rust patterns
- Improve upon existing Python architecture
- Eventually replace Python implementation with Rust
- Build a robust, production-ready RabbitMQ client

> **Note:** Currently awaiting Lapin bug fixes for channel reconnection handling. Project updates will follow as dependencies evolve.

---

## ✨ Features

- **⚡ Full Tokio Runtime** - Leveraging async/await for maximum performance
- **🐇 Lapin AMQP Client** - Robust RabbitMQ integration (with pending bug fixes)
- **🗄️ SQLx with PostgreSQL** - Type-safe database operations
- **🔄 Producer/Consumer Patterns** - Reliable message processing
- **🔧 Configurable Architecture** - Flexible and extensible design
- **🚀 High Performance** - Built with Rust's zero-cost abstractions

---

## 🏗️ Project Structure

```sh
rabbitmq-async-example/
├── 📁 src/
│   ├── 📁 bin/
│   │   └── main.rs              # Application entry point
│   ├── 📁 database/
│   │   ├── 📁 models/           # Database models and schemas
│   │   └── 📁 functions/        # Database operations + schemas
│   ├── 📁 mapping/              # Data mapping and validation
│   ├── 📁 rmq/                  # RabbitMQ handlers and schemas
│   ├── 📁 tasks/
│   │   ├── 📁 producer/         # Message producers
│   │   └── 📁 consumer/         # Message consumers
│   ├── configs.rs               # Configuration management
│   ├── errors.rs                # Error handling
│   └── lib.rs                   # Library exports
├── 📁 .github/workflows/        # CI/CD pipelines
├── 📁 migrations/               # SQLX migrations
├── Cargo.toml                   # Rust dependencies
├── rustfmt.toml                 # Code formatting
├── docker_compose.yml           # Docker images examples for project
└── LICENSE                      # MIT License
```

### 📋 Core Modules

<details open>
<summary><b>Project Architecture Overview</b></summary>

#### 🏗️ Main Components

| Module | Purpose | Key Files |
|--------|---------|-----------|
| **`src/rmq/`** | RabbitMQ connection management and message handling | `handlers.rs`, `schemas.rs` |
| **`src/tasks/`** | Producer and consumer implementations | `producer/methods.rs`, `consumer/methods.rs` |
| **`src/database/`** | PostgreSQL operations with SQLx | `models/`, `functions/mod.rs` |
| **`src/mapping/`** | Data validation and transformation | `schemas.rs`, `validators.rs` |
| **`src/configs/`** | Application configuration management | `configs.rs` |
| **`src/errors/`** | Custom error handling | `errors.rs` |

</details>

<details>
<summary><b>📁 Source Code Details</b></summary>

#### 🎯 Main Entry Points
- **`src/bin/main.rs`** - Application entry point and startup logic
- **`src/lib.rs`** - Library exports and module declarations
- **`src/configs.rs`** - Configuration struct and environment setup
- **`src/errors.rs`** - Custom error types and error handling

#### 🔄 RabbitMQ Modules (rmq/)
- **`src/rmq/schemas.rs`** - Message schemas and data structures for RabbitMQ
- **`src/rmq/handlers.rs`** - Connection handling and message processing logic

#### 📊 Task Management (tasks/)
- **`src/tasks/producer/methods.rs`** - All methods for producing messages to RabbitMQ
- **`src/tasks/consumer/methods.rs`** - Consumer logic and message processing methods
- **`src/tasks/consumer/utils.rs`** - Utility functions to simplify consumer logic

#### 🗄️ Database Layer (database/)
- **`src/database/models/`** - Database models and entity definitions:
  - `application_requests.rs` - Request models
  - `application_responses.rs` - Client response models
  - `service_responses.rs` - Service response models
  - `services.rs` - Main service models
  - `fail_table.rs` - Error and failure tracking models
- **`src/database/functions/mod.rs`** - Database operations and queries

#### 📝 Data Mapping (mapping/)
- **`src/mapping/schemas.rs`** - Data transformation schemas
- **`src/mapping/validators.rs`** - Input validation and data sanitization

#### ⚙️ Configuration Files
- **`Cargo.toml`** - Rust dependencies and project metadata
- **`rustfmt.toml`** - Code formatting configuration
- **`.env.example`** - Environment variables template
- **`.github/workflows/project_check.yaml`** - CI/CD pipeline configuration

</details>

---

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable version)
- **Cargo** (Rust package manager)
- **PostgreSQL** (database)
- **RabbitMQ** (message broker)

### Installation

#### 1. Clone the Repository
```bash
git clone https://github.com/GregTMJ/rabbitmq-async-example
cd rabbitmq-async-example
```

#### 2. Environment Configuration
```bash
cp .env.example .env
```
Edit the .env file with your specific configuration:

```sh
# RMQ main configs
RMQ_USER=user
RMQ_PASSWORD=bitnami
RMQ_HOST=lapin_rabbitmq
RMQ_PORT=5672
RMQ_VHOST=%2F
RMQ_EXCHANGE=servicehub
RMQ_EXCHANGE_TYPE=direct
RMQ_PREFETCH_COUNT=0

# RMQ exchange/queue configs
RMQ_REQUEST_QUEUE=servicehub.q.request
RMQ_RESPONSE_QUEUE=servicehub.q.response
RMQ_SERVICE_RESPONSE_QUEUE=servicehub.q.service_response
RMQ_DASHBOARD_SETTINGS_QUEUE=servicehub.q.dashboard_settings
RMQ_DASHBOARD_FANOUT_EXCHANGE=serhub_info_fanout_exchange
RMQ_TIMEOUT_QUEUE=timeout_requests
RMQ_DELAYED_EXCHANGE=delayed_exchange

# Postgres configs
POSTGRES_HOST=lapin_postgres
POSTGRES_PORT=5432
POSTGRES_DB=servicehub
POSTGRES_PASSWORD=LyhaEkM2D6TeH96W8jxG
POSTGRES_USER=servicehub

DATABASE_URL=postgres://servicehub:LyhaEkM2D6TeH96W8jxG@lapin_postgres:5432/servicehub

# Logging level configs
RUST_LOG=INFO

# Custom project configs
AVAILABLE_USERS=1,3,5,6,7,8,9,10,11,12
AVAILABLE_SERVICES=0,1,2,3,4,7,8,9,10,11,12,13,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52
```

#### 3. Build the Project
```sh
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing

#### 🧪 Test Suite

The project includes comprehensive testing to ensure reliability and performance:

**Core Test Categories:**
- **Unit Tests** - Individual function and module testing
- **Integration Tests** - Cross-module functionality testing  
- **Database Tests** - PostgreSQL operations and SQLx queries
- **RabbitMQ Tests** - Message producer/consumer integration
- **Async Tests** - Tokio runtime and asynchronous operations

**Test Configuration:**
- Tests run in isolated environments
- Database transactions are rolled back after each test
- RabbitMQ connections use test-specific queues
- All I/O operations use mock implementations where appropriate

#### 📊 Test Coverage


**Coverage Tools:**
- Automated coverage reporting in CI/CD
- Regular coverage analysis
- Coverage thresholds for new code

#### 🔧 Test Environment

**Test-Specific Configuration:**
- Separate test database
- Isolated RabbitMQ virtual hosts
- Mock external service dependencies
- Configurable test timeouts

**Test Data Management:**
- Factory functions for test data creation
- Database seeding for integration tests
- Clean state between test runs
- Consistent test data patterns

---

## 🗺️ Roadmap

### Current Development Focus

| Status | Task | Description | Priority |
|--------|------|-------------|----------|
| ✅ Completed | Core Implementation | Basic async RabbitMQ client with Tokio | High |
| 🔄 In Progress | Channel Management | Lapin channel reconnection handling | Critical |
| ⏳ Planned | Database Optimization | SQLx connection pooling and query optimization | High |
| ⏳ Planned | Error Handling | Comprehensive error management system | Medium |

### Upcoming Releases

**Version 1.5.0** - Stability & Bug Fixes
- Lapin dependency updates
- Connection resilience improvements
- Enhanced error recovery

**Version 2.0.0** - Performance Enhancements  
- Tokio runtime optimization
- Message throughput improvements
- Memory usage optimization

**Version 3.0.0** - Production Ready
- Docker containerization
- Kubernetes deployment support
- Monitoring and metrics
- Security hardening

### Long-term Vision

**Architecture Evolution:**
- Microservices readiness
- Event sourcing capabilities
- Distributed tracing integration
- Multi-tenancy support

**Ecosystem Integration:**
- OpenTelemetry support
- Prometheus metrics
- Grafana dashboards
- Health check endpoints

---

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

### Development Process

**Getting Started:**
1. Review open issues and feature requests
2. Check the roadmap for planned features
3. Discuss major changes via GitHub issues first
4. Follow Rust coding conventions and patterns

**Code Quality Standards:**
- All code must pass clippy checks
- Tests required for new functionality
- Documentation updates for API changes
- Benchmarking for performance-critical code

**Submission Guidelines:**
- Use conventional commit messages
- Include relevant test cases
- Update documentation as needed
- Ensure CI/CD pipelines pass

### Areas Needing Contribution

**High Priority:**
- RabbitMQ connection resilience
- Performance optimization
- Additional database support
- Enhanced monitoring

**Community Needs:**
- Usage examples and tutorials
- Integration guides
- Troubleshooting documentation
- Benchmark results

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for complete details.

**Key Permissions:**
- Commercial use
- Distribution
- Modification
- Private use

**Requirements:**
- License and copyright notice preservation
- Same license for substantial portions

**No Limitations On:**
- Liability
- Warranty
- Patent use
- Trademark use

---

## 🔗 Support

**Community Resources:**
- GitHub Discussions for questions
- Issue tracker for bug reports
- Documentation wiki for guides
- Discord channel for real-time help

**Professional Support:**
- Coming soon: Enterprise support options
- Planned: Commercial licensing
- Future: Consulting services

---

<div align="center">

**Need help?** 
Open an issue or join our community discussions!

---

**Built with ❤️ using Rust, Tokio, and RabbitMQ**

[⬆ Back to Top](#top)
</div>