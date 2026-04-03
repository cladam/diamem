## diamem DSL Cheat Sheet

### Syntax at a glance

| What              | Syntax                    | Example                          |
|-------------------|---------------------------|----------------------------------|
| Comment           | `# text`                  | `# this is ignored`              |
| Connection        | `A -> B`                  | `Code -> Deploy`                 |
| Chain             | `A -> B -> C`             | `Code -> Build -> Deploy`        |
| Labeled (old)     | `A -[label]-> B`          | `API -[REST]-> DB`               |
| Labeled (new)     | `A -(label)> B`           | `API -(REST)> DB`                |
| Sequence          | `A > B : Message`         | `User > App : Login`             |
| Grouping (old)    | `[Name] { A, B }`         | `[Backend] { API, Worker }`      |
| Grouping (new)    | `@ Name: A, B`            | `@ Backend: API, Worker`         |
| Node              | `Name`                    | `Standalone`                     |
| Mindmap root      | `mindmap: Root`           | `mindmap: My Project`            |
| Mindmap branch    | `- Name`                  | `- Frontend`                     |
| Mindmap leaf      | `-- Name`                 | `-- React`                       |
| Timeline title    | `timeline: Title`         | `timeline: History`              |
| Timeline period   | `- period : event`        | `- 2004 : Facebook`              |
| Timeline events   | `- period : ev1, ev2`     | `- 2004 : Facebook, Google`      |
| Timeline section  | `@ SectionName`           | `@ Early Days`                   |

---

### Example 1 — Mindmap: Project Architecture
```
[Frontend] { WebApp, MobileApp, CLI }
[Backend] { API, AuthService, Worker }
[Storage] { Postgres, Redis, S3 }

WebApp -> API
MobileApp -> API
CLI -> API
API -(queries)> Postgres
API -(caches)> Redis
Worker -(uploads)> S3
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

### Example 3 — Mindmap: Life Routine (using chains)
```
@ Morning: Wake, Coffee, Review
@ DeepWork: Code, Design, Write
@ WindDown: Walk, Read, Journal

Wake -> Coffee -> Review -> Code -> Design -> Write -> Walk -> Read -> Journal
```

### Example 4 — Mindmap: ADHD Task Breakdown (using @ and chains)
```
@ Phase1: Scaffold, Parser, BasicUI
@ Phase2: SVGRender, LivePreview
@ Phase3: PNGExport, ShotextLink
@ Phase4: Themes, Shortcuts, Polish

Scaffold -> Parser -> BasicUI -> SVGRender -> LivePreview
LivePreview -> PNGExport -> ShotextLink -> Themes -> Shortcuts -> Polish
```

### Example 5 — Mixed: Microservices + Sequence
```
@ Gateway: APIGateway
@ Services: UserSvc, OrderSvc, PaymentSvc
@ Infra: Kafka, Postgres, Redis

APIGateway -(REST)> UserSvc
APIGateway -(REST)> OrderSvc
OrderSvc -(event)> Kafka
Kafka -(consumes)> PaymentSvc
UserSvc -[reads]-> Postgres
PaymentSvc -(caches)> Redis

# Checkout sequence
User > APIGateway : POST /checkout
APIGateway > OrderSvc : Create order
OrderSvc > Kafka : OrderCreated event
Kafka > PaymentSvc : Process payment
PaymentSvc > OrderSvc : PaymentConfirmed
OrderSvc > APIGateway : Order complete
APIGateway > User : 200 OK
```

### Example 6 — Mindmap: Project Planning
```
# A mindmap: block switches to Mermaid's mindmap renderer.
# Depth is expressed by dash count: - = level 1, -- = level 2, etc.

mindmap: diamem
- DSL
-- Parser
-- Grammar
-- Syntax
- Rendering
-- Mermaid
-- SVG
-- PNG Export
- UI
-- Editor
-- Preview
-- Themes
- Integration
-- Shotext
-- OCR Footer
```

### Example 7 — Mindmap: ADHD Daily Breakdown
```
mindmap: My Day
- Morning
-- Wake up
-- Coffee
-- Review tasks
- Deep Work
-- Code
-- Design
-- Write docs
- Wind Down
-- Walk
-- Read
-- Journal
```

### Example 8 — Timeline: Social Media History
```
timeline: History of Social Media
- 2002 : LinkedIn
- 2004 : Facebook
- 2005 : Youtube
- 2006 : Twitter
```

### Example 9 — Timeline with Sections and Multiple Events
```
# Sections group related time periods.
# Comma-separated values create multiple events per period.

timeline: History of Social Media
@ Early Days
- 2002 : LinkedIn
- 2004 : Facebook, Google
@ Growth
- 2005 : Youtube
- 2006 : Twitter
- 2007 : Tumblr, iPhone
@ Modern Era
- 2010 : Instagram, Pinterest
- 2011 : Snapchat
- 2016 : TikTok
```

> **Tip:** You can mix all syntax types freely in one diagram. Both `[Group] { ... }` and `@ Group: ...` define clusters. Both `-[label]->` and `-(label)>` add labeled edges. Use `->` chains to lay out linear flows in a single line, and `> :` for message sequences. Use `mindmap:` to create hierarchical mind maps and `timeline:` to create timeline charts — when present, the output switches from `graph TD` to Mermaid's `mindmap` or `timeline` renderer respectively.
