# Development Documentation

This doc is a detail guide for the architecture in [project.md](project.md). Matching bot behavior is also described in [gateway_core_engine_logic.md](gateway_core_engine_logic.md).

**Detail scope:** Local simulators and helpers for integration testing—not production deployment or the master platform flow.

Simulators and queues below validate deposits, notices, partner APIs, and matching without live traffic.

Partner setup UML: [partner-uml.md](partner-uml.md).

## 1. Deposit Simulator

The deposit simulator is used to generate simulated deposit events for testing and validation purposes.

It allows developers to trigger synthetic deposit transactions and verify that the downstream processing flow, event propagation, and account updates are functioning as expected.

This simulator is especially useful for validating:

- deposit event ingestion
- webhook processing
- notification flow
- account balance updates
- client-facing status responses

## 2. Notice Message Queue

The notice message queue is a message tracking and monitoring component used to review outbound system notifications.

It provides a centralized view of outgoing messages so developers can confirm whether the sending logic succeeded and whether a message was processed correctly by downstream services.

This queue supports validation of:

- outbound event notifications
- message delivery status
- alert visibility and traceability
- operational debugging for system communication flows

## 3. Auto Order Matching

An auto matching order bot will perform automated order placement and matching activity. It is designed to simulate trading behavior in a controlled development environment and to validate the end-to-end latency, execution flow, and matching logic of the system.

This bot will:

- place buy and sell orders automatically
- route order requests through the Gateway and Client Center path
- send valid internal order commands to the Core Engine
- trigger the matching engine using randomized timing patterns
- generate execution results and trading notices for validation

The bot is intended to operate with random timing intervals so that the platform can be tested under irregular traffic patterns, burst behavior, and non-uniform load. This helps developers validate matching fairness, ring-buffer throughput, and queue stability under realistic but unpredictable conditions.

## 4. Local Development (Docker-based)

Dev simulators + Core Engine compose stack lives under [`dev_simulator/docker-compose.yml`](../dev_simulator/docker-compose.yml).

```bash
cd dev_simulator
docker compose up --build
```

| Container | Port | Notes |
| --- | --- | --- |
| `funnyx-payment-gate` | `18100` | Fiat / deposit webhook sim |
| `funnyx-message-queue` | `18101` | Notice queue (Core Engine `MESSAGE_CENTER_URL`) |
| `funnyx-company-a-server` | `18102` | Partner APIs + notice callback |
| `funnyx-market-bot` | `18103` | Book watcher / fill bot → `core-engine` |
| `funnyx-core-engine` | `18200` | Matching; data volume `core_engine_data` |

Network: `funnyx-dev-sim` (`172.28.0.0/24`). Full platform DB Docker (Postgres/Redis/Mongo) remains a separate Phase 1 item.

## 5. Development Principles

The development environment is designed to support safe validation of platform behavior without affecting live trading or production system integrity.

All simulator-based activities should be isolated from production traffic, and all generated data should be clearly marked as test or simulated data where appropriate.
