# System Type and Constant Values

This doc is a detail constants file for the architecture in [project.md](project.md). Socket layouts live in [socket_message.md](socket_message.md).

**Detail scope:** Message, server, role, order, shop, event, action, and target constants referenced from [project.md §6](project.md#6-server-group-structure) and [§7](project.md#7-inter-service-communication).

## Constant tables

Message Type - Byte
0xF1 - `Normal Command` 
0xF2 - `Complex Command`

Server Type - Int
00 - Config & Heartbeat
01 - Admin API
02 - Admin Panel
03 - Webhook Server
04 - Message Center
05 - Session Token Server
06 - Gateway
07 - Core Engine
08 - Client Center
09 - Client Web
10 - Cache Layer like Redis
11 - Document DB like MongoDB
12 - Relational DB like PostgreSQL

User Role Type - Int
00 - Admin - Root Admin
01 - Admin - Manager
02 - Admin - Staff (Maker)
03 - Admin - Staff (Checker)
04 - Admin - Author
05 - Admin - ReadOnly
06 - Client - Normal User
07 - Client - Corp User
08 - Client - End User
09 - AI Agent
10 - Public User

Order Type (System Command Type):
01 - Market Order
02 - Price Order

Order Status Type:
00 - Pending
01 - Cancel
02 - Filled
03 - Partial Filled

Shop Package Status:
00 - Active
01 - Inactive
02 - Archived

Shop Order Status:
00 - Pending
01 - Paid
02 - Failed
03 - Expired
04 - Cancelled

Shop Payment Event Status:
00 - Received
01 - Settled
02 - IgnoredDuplicate
03 - Rejected

Trade Action:
01 - Bid
02 - Ask

Date Type:
00 - NULL
01 - Boolean
02 - ENUM (System Config Server)
03 - Integer
04 - Long/BigInt
05 - String (Max 50 characters)
06 - Float
07 - Double
08 - Object - JSON
09 - Object - Array

Event List - Int
00 - Heartbeat - PING / PONG

01 - Admin User Login
02 - Admin User Logout
03 - Admin User Forget Password
04 - Admin User Submit

11 - Client User Login
12 - Client User Logout
13 - Client User Forget Password
14 - Client User Submit

21 - Shop Package Created
22 - Shop Package Updated
23 - Shop Order Created
24 - Shop Payment Callback Received
25 - Shop Order Paid
26 - Shop Order Failed
27 - Shop Order Expired
28 - Shop Wallet Credited

31 - Company Basic Token Submitted
32 - Company Basic Token Approved
33 - Company Basic Token Rejected
34 - Company Basic Token Buy Order Created
35 - Company Basic Token Buy Paid
36 - Company Basic Token Fee Charged (0.1%)

Action List - Int
00 - Hold
01 - Insert
02 - Update
03 - Delete
04 - Lock
05 - Unlock
06 - Expired
07 - Send Notice
08 - Enable
09 - Disable
10 - Cancel
11 - Checkout
12 - Settle
13 - Credit Wallet
14 - Approve
15 - Reject
16 - Charge Fee

Target Type:
00 - Admin Config
01 - Admin API
02 - Client Config
03 - Client API
04 - Normal Config - Game Partner Game Coin
05 - Normal Config - Internal System
06 - Normal Config - Public view
07 - Normal Config - Market (Core Engine)
08 - Normal Config - Gateway
09 - Frontend (All end point)
10 - E-shop Package
11 - E-shop Order
12 - E-shop Payment Callback
13 - Company Basic Token
14 - Company Basic Token Buy Order
