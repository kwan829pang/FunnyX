# FunnyX System Flow Charts

This doc is a detail diagram file for the master architecture in [project.md](project.md) and the API index in [api-master.md](../api-master.md).

**Detail scope:** Mermaid diagrams for the master business and runtime flows in [project.md §4](project.md#4-platform-business-flow-master) (and related topics); not a duplicate of the server catalog.

## 1. High-Level Platform Flow

```mermaid
flowchart TD
    CW[Client Web Token] --> B[Gateway]
    P[Company Partner Master] --> B
    C[Admin Panel] --> D[Admin API]
    E[Webhook Server] --> F[Message Center]
    E --> G[Client Center]
    H[Core Engine] --> F
    H --> G
    D --> I[Config Server]
    B --> G
    I --> B
    I --> H
    I --> D
    B --> R[In-Memory Service Map]
    R --> S[service_registry]
    R --> T[route_map]
    R --> U[health_state]
    R --> V[connection_pool]

    F --> J[(Redis)]
    G --> K[(PostgreSQL)]
    G --> GM[(MongoDB chat required)]
    F --> L[(MongoDB)]
    D --> M[(SQLite / LevelDB)]
    H --> N[(Database)]
    E --> O[External Deposit / Event Notice]
```

## 2. HTTP GET Flow

**C8:** Client Web uses Token Server token on APIs (after Public login). Company Partner → Client Center uses Master Account Code + Master ID + API Key + Secret.

```mermaid
sequenceDiagram
    participant CU as Client End User
    participant CW as Client Web
    participant GW as Gateway
    participant S as Target Service

    CU->>CW: Request data
    CW->>GW: HTTP GET with Session
    GW->>GW: Validate session token
    GW->>S: Route request to target service
    S-->>GW: Return response payload
    GW-->>CW: Final HTTP response
    CW-->>CU: Return result to end user
```

## 3. HTTP POST Flow

```mermaid
sequenceDiagram
    participant CU as Client End User
    participant CW as Client Web
    participant GW as Gateway
    participant S as Target Service

    CU->>CW: Submit action
    CW->>GW: HTTP POST with Session and body
    GW->>GW: Validate session token and payload
    GW->>S: Route POST request to target service
    S-->>GW: Process and return result
    GW-->>CW: Return HTTP response
    CW-->>CU: Confirm success or failure
```

## 4. Client Authentication and Socket Session Flow

Register on Client Web, Partner **OAuth 2.0**, or Platform OAuth for Partner; then Session Token Server issues the Client Web token.

```mermaid
sequenceDiagram
    participant CU as Client End User
    participant CW as Client Web
    participant GW as Gateway
    participant ST as Session Token Server
    participant P as Company Partner

    alt Client Web register plus login
        CU->>CW: Register new account
        CW->>GW: POST client register
        CU->>CW: Login credentials
        CW->>GW: Login request
    else Partner OAuth 2.0
        CU->>CW: Start Partner OAuth
        CW->>P: OAuth authorize
        P-->>CW: Authorization code
        CW->>GW: OAuth partner callback
        GW->>P: Token plus userinfo
        P-->>GW: Partner user identity
    end
    GW->>ST: Issue session token
    ST-->>CW: Session token returned
    CU->>GW: Connect socket with session token
    GW-->>CU: Socket session established
```

## 5. Order Placement and Matching Flow

```mermaid
flowchart LR
    U[End User] --> CW[Client Web Session]
    CW --> G[Gateway]
    B[Auto Matching Bot] --> G
    G --> C[Client Center]
    C --> E[Core Engine]
    C --> DB[(Order Database)]
    E --> DB
    E --> M[Matching Logic]
    M --> D[Trade Execution]
    D --> MC[Message Center]
    D --> CC[Client Center]
    MC --> Msg[(Message Storage)]
    CC --> Client[Client Notifications]
    CC --> Balance[User Balance Update]
    C -. internal private network .-> E
    G -. public Session or MasterSigned ingress .-> C
    B -. random interval order placement .-> G
```

## 6. Market Creation Workflow (Corp Submit + Admin Approve)

**C6 fees:** Year-1 **80K** / renewal **10K** / extra pair **10K**. Fiat = Admin base (**HKD** or **USD**, changeable); or pay **USDT**. See [partner.md](partner.md) §7.

Client Submitted → Under Pending (lock required Game Partner Game Coin pool amount) → Wait Admin User Review → Confirm approval by Admin User → Transfer Client Balance to Pool

```mermaid
flowchart TD
    A[Client Submitted Market Pair] --> B{C6 fee gate}
    B -->|Year1 package includes first pair| C[80K Admin fiat or USDT]
    B -->|Extra pair| D[10K Admin fiat or USDT]
    C --> E{First market?}
    D --> E
    E -->|Yes| F[Must pair with Platform Token]
    E -->|No| G[Under Pending]
    F --> G
    G --> H[Lock required amount on Game Partner Game Coin client balance]
    H --> I[Wait Admin User Review]
    I --> J{Confirm approval by Admin?}
    J -->|Yes| K[Transfer Client Balance to Pool]
    K --> L[Assign market to Core Engine]
    L --> M[Market ready for trading]
    J -->|No| N[Unlock client balance and revise]
```

## 7. Heartbeat and Health Check Flow

```mermaid
sequenceDiagram
    participant CS as Config Server
    participant GW as Gateway
    participant CC as Client Center
    participant CE as Core Engine
    participant MC as Message Center

    Note over CS,MC: Private in-system heartbeat (funnyx-heartbeat). Public GET /health is for Cloudflare / Game Partners only.

    loop Every heartbeat interval
        CS->>GW: Socket PING
        GW-->>CS: Socket PONG

        GW->>CC: Socket PING
        CC-->>GW: Socket PONG

        CC->>CE: Socket PING (private network)
        CE-->>CC: Socket PONG

        CE->>MC: Socket PING
        MC-->>CE: Socket PONG
    end
```

## 8. Config and Heartbeat Flow

```mermaid
flowchart TD
    CS[Config Server] --> HB[Heartbeat Monitor]
    G[Gateway] --> CS
    CE[Core Engine] --> CS
    A[Admin API] --> CS
    CS --> M[In-memory service registry]
    M --> A
    M --> G
    M --> CE
    G --> RT[Route requests]
    CE --> RT
```

## 9. Message Distribution Flow

```mermaid
sequenceDiagram
    participant CE as Core Engine
    participant MC as Message Center
    participant Redis as Redis Cache
    participant Mongo as MongoDB
    participant UI as Client Web / Frontend

    CE->>MC: Send trade or event message
    MC->>Redis: Publish immediate notification
    MC->>Mongo: Persist message record
    Redis-->>UI: Push live update
    UI-->>MC: Request message history
    MC-->>UI: Return message data
```

## 10. Client-to-Platform Socket and API Combined Flow

```mermaid
flowchart TD
    A[Client App] --> B[HTTP API Request]
    A --> C[Socket Connection]
    B --> D[Signature Validation]
    C --> E[Session Token Validation]
    D --> F[Gateway]
    E --> F
    F --> G[Target Service]
    G --> H[Core Engine or Client Center]
    H --> I[Response / Event Payload]
    I --> J[JSON / Gzip or Binary Buffer]
    J --> A
```

## 11. E-shop Purchase and Webhook Credit Flow

**C2 + C4:** Platform fixed PLT_* **and** Corp-created e-shop products. Partner **fiat** payment (HKD/USD per Admin system base currency). Shop orders do not enter the Core Engine matching path.

**C1 + OAuth:** End User registers on Client Web **or** Partner/Platform **OAuth 2.0** → Session Token Server token → Client Web calls Gateway with **Session**.

```mermaid
flowchart LR
    A[Register ClientWeb or OAuth2.0] --> B[Login TokenServer]
    B --> C[Session Token]
    C --> D[ClientWeb Eshop Catalog]
    D --> E{Seller lane}
    E -->|Platform| F[Select PLT package]
    E -->|Corp| G[Select Corp product]
    F --> H[Gateway Session]
    G --> H
    H --> I[ClientCenter CreateShopOrder]
    I --> J[PartnerFiatPayment]
    J --> K[WebhookCallback]
    K --> L{Paid?}
    L -->|Yes platform| M[CreditPlatformToken]
    L -->|Yes corp| N[CreditCoinOrFulfillItem]
    L -->|No| O[MarkFailedOrExpired]
    M --> P[MessageCenter Notify]
    N --> P
    O --> P
```

```mermaid
sequenceDiagram
    participant U as EndUser
    participant CW as ClientWeb
    participant ST as SessionTokenServer
    participant GW as Gateway
    participant CC as ClientCenter
    participant P as PartnerPayment
    participant WH as WebhookServer
    participant MC as MessageCenter

    alt Register on Client Web
        U->>CW: Register new account
        CW->>GW: POST client register
        GW->>CC: Create end user
        U->>CW: Login
        CW->>GW: Login request
        GW->>ST: Issue session token
        ST-->>CW: Session token
    else Partner OAuth 2.0
        U->>CW: Start Partner OAuth
        CW->>P: OAuth authorize
        P-->>CW: Authorization code
        CW->>GW: OAuth partner callback
        GW->>CC: Create or bind end user
        GW->>ST: Issue session token
        ST-->>CW: Session token
    end
    alt Platform fixed package
        U->>CW: Select PLT package
        CW->>GW: Create platform shop order with Session
    else Corp e-shop product
        U->>CW: Select Corp product
        CW->>GW: Create corp shop order with Session
    end
    GW->>CC: Validate and persist pending order
    CC->>P: Create fiat payment
    P-->>CC: partner_order_no and checkout_url
    CC-->>CW: Return checkout redirect
    U->>P: Complete fiat payment
    P->>WH: Payment callback
    WH->>CC: Settle shop order idempotently
    alt Platform order
        CC->>CC: Credit USER_WALLET Platform Token
    else Corp order
        CC->>CC: Credit coin or fulfill game item
    end
    CC->>MC: Publish paid or failed notice
    MC-->>CW: Notify user
```

## 12. Company Basic Token Submit → Approve → Buy (0.1% fee)

```mermaid
flowchart TD
    A[Corp Submits Company Basic Token] --> B[Pending Admin Review]
    B --> C{Admin Approved?}
    C -->|No| D[Not Buyable Revise]
    C -->|Yes| E[Token Buyable]
    E --> F[User Creates Buy Order]
    F --> G[Partner Token Payment]
    G --> H[Webhook Corp Token Payment]
    H --> I[Charge 0.1 percent Company Basic Token Fee]
    I --> J[Credit User 99.9 percent]
```
