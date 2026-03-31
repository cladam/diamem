## diamem DSL Cheat Sheet

### Syntax at a glance

| What       | Syntax            | Example                     |
|------------|-------------------|-----------------------------|
| Comment    | `# text`          | `# this is ignored`         |
| Connection | `A -> B`          | `Code -> Deploy`            |
| Labeled    | `A -[label]-> B`  | `API -[REST]-> DB`          |
| Sequence   | `A > B : Message` | `User > App : Login`        |
| Grouping   | `[Name] { A, B }` | `[Backend] { API, Worker }` |
| Node       | `Name`            | `Standalone`                |

---

### Example 1 — Mindmap: Project Architecture
```
[Frontend] { WebApp, MobileApp, CLI }
[Backend] { API, AuthService, Worker }
[Storage] { Postgres, Redis, S3 }

WebApp -> API
MobileApp -> API
CLI -> API
API -[queries]-> Postgres
API -[caches]-> Redis
Worker -[uploads]-> S3
```

### Example 2 — Sequence: User Login
```
User > Browser : Opens app
Browser > API : POST /login
API > AuthService : Validate credentials
AuthService > DB : SELECT user
DB > AuthService : User record
AuthService > API : JWT token
API > Browser : 200 OK + token
Browser > User : Dashboard loaded
```

### Example 3 — Mindmap: Life Routine
```
[Morning] { Wake, Coffee, Review }
[Deep Work] { Code, Design, Write }
[Wind Down] { Walk, Read, Journal }

Wake -> Coffee
Coffee -> Review
Review -> Code
Code -> Design
Design -> Write
Write -> Walk
Walk -> Read
Read -> Journal
```

### Example 4 — Mindmap: ADHD Task Breakdown
```
[Phase1] { Scaffold, Parser, BasicUI }
[Phase2] { SVGRender, LivePreview }
[Phase3] { PNGExport, ShotextLink }
[Phase4] { Themes, Shortcuts, Polish }

Scaffold -> Parser
Parser -> BasicUI
BasicUI -> SVGRender
SVGRender -> LivePreview
LivePreview -> PNGExport
PNGExport -> ShotextLink
ShotextLink -> Themes
Themes -> Shortcuts
Shortcuts -> Polish
```

### Example 5 — Mixed: Microservices + Sequence
```
[Gateway] { APIGateway }
[Services] { UserSvc, OrderSvc, PaymentSvc }
[Infra] { Kafka, Postgres, Redis }

APIGateway -[REST]-> UserSvc
APIGateway -[REST]-> OrderSvc
OrderSvc -[event]-> Kafka
Kafka -[consumes]-> PaymentSvc
UserSvc -[reads]-> Postgres
PaymentSvc -[caches]-> Redis

# Checkout sequence
User > APIGateway : POST /checkout
APIGateway > OrderSvc : Create order
OrderSvc > Kafka : OrderCreated event
Kafka > PaymentSvc : Process payment
PaymentSvc > OrderSvc : PaymentConfirmed
OrderSvc > APIGateway : Order complete
APIGateway > User : 200 OK
```

> **Tip:** You can mix all syntax types freely in one diagram. Groups define clusters, `->` shows flow, `-[label]->` adds context, and `>` `:` shows message sequences.
