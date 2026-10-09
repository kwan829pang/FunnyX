# Socket Message Specification

This doc is a detail protocol file for the architecture in [project.md](project.md). Constants are in [system_type.md](system_type.md), and the API index is in [api-master.md](../api-master.md).

**Detail scope:** Byte-buffer layouts, compression, and heartbeat rules for inter-service and client sockets—master comms summary in [project.md §7](project.md#7-inter-service-communication).

## 1. Overview

Field sizes, message categories, and payload models for S2S (lz4 byte buffers) and server-to-client (JSON + gzip). Routing roles are defined in [project.md](project.md); this file is the protocol reference only.

## 2. Server-to-Server Socket Communication

Gateway is the public entry; private channels connect internal services. Client Center forwards order traffic to Core Engine on the private network.

### 2.1 Compression

All server-to-server socket payloads are compressed using lz4_flex before transmission.

### 2.2 Data Format

Server-to-server messages use a byte buffer format. System constant values should be referenced from system_type.md.

### 2.3 Message Types

There are two supported socket message categories:

1. Normal Command
2. Complex Command

The message type is identified by checking the first byte of the payload.

### 2.4 Normal Command

Normal commands use a fixed message body size of 512 bytes.

| Field | Size | Type | Description |
| --- | --- | --- | --- |
| 1 | 1 | Int | Message Type |
| 2 | 1 | Int | Event Name |
| 3 | 1 | Int | Action Type |
| 4 | 2 | Int | Content Size |
| 5 | 507 | Object | Content Data |

### 2.4.1 Order Payload Model

For order messages, the content payload must include a dedicated order type field so the receiving system can identify the order model without ambiguity.

The platform supports only two order types:

- 01 = Market Order
- 02 = Price Order

The order payload inside the content area should follow this structure:

| Field | Size | Type | Description |
| --- | --- | --- | --- |
| 1 | 1 | Int | Order Type (01 Market Order, 02 Price Order) |
| 2 | 1 | Int | Trade Side (01 Bid, 02 Ask) |
| 3 | 8 | BigInt | Client ID |
| 4 | 8 | BigInt | Market ID |
| 5 | 8 | BigInt | Price (for price order; ignored or optional for market order) |
| 6 | 8 | BigInt | Quantity |
| 7 | 8 | BigInt | Timestamp |
| 8 | 1 | Int | Order Status (00 Pending, 01 Cancel, 02 Filled, 03 Partial Filled) |
| 9 | 1 | Int | Reserved / Future extension |
| 10 | 1 | Int | Reserved / Future extension |
| 11 | 1 | Int | Reserved / Future extension |
| 12 | 472 | Bytes | Remaining payload metadata |

Important rules:

- A Market Order does not require a limit price to be set; the engine resolves the execution price from the best available market price.
- A Price Order must include a valid limit price and the engine will match only when the market crosses that price.
- The order type always comes before the price/quantity payload so the Core Engine can route the message to the correct matching branch immediately.

### 2.5 Complex Command

Complex commands use a dynamic-length payload and are intended for larger or variable-size messages.

| Field | Size | Type | Description |
| --- | --- | --- | --- |
| 1 | 1 | Int | Message Type |
| 2 | 2 | Int | Content Size |
| 3 | N | Bytes | Content Data |

## 3. Server-to-Client Socket Communication

Server-to-client socket messages are encoded in JSON format and compressed using gzip before transmission.

This approach is used to ensure compatibility with client-side environments while maintaining efficient data transfer for browser and application clients.

## 4. Heartbeat and Health Checks

Two common mechanisms — different audiences. A service (e.g. Client Center) typically exposes **both**.

| Mechanism | Common crate | Audience | Network |
| --- | --- | --- | --- |
| Socket PING/PONG | `funnyx-heartbeat` | **Private / in-system** (Config Server, Gateway, peers) | Internal / private |
| HTTP `GET /health` | `funnyx-health` | **Public / third party** (Cloudflare, Game Partners, external LB) | Public HTTP |

**Socket heartbeat (private):** Config Server (and later Gateway peers) send length-prefixed PING; the peer replies PONG. No response within timeout → mark unreachable in the registry. This is how FunnyX services learn each other’s status.

**HTTP `/health` (public):** Minimal JSON (`funnyx-health`) so Cloudflare, Game Partners, or other external monitors can confirm the HTTP surface is open and stable. Not used as the platform registry heartbeat.

Gateway keeps an in-memory service table (registry snapshot, routes, upstream health from **socket** probes) for local routing decisions.

