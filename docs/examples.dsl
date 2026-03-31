# diamem DSL — Examples
#
# Paste any of these into the editor to try them out.
# Lines starting with # are comments and are ignored.


# ─── MINDMAP: Project Architecture ────────────────────────────

# Groups define clusters of related concepts
[Frontend] { WebApp, MobileApp, CLI }
[Backend] { API, AuthService, Worker }
[Storage] { Postgres, Redis, S3 }

# Simple connections show relationships
WebApp -> API
MobileApp -> API
CLI -> API

# Labeled connections add context
API -[queries]-> Postgres
API -[caches]-> Redis
Worker -[uploads]-> S3
API -[dispatches]-> Worker


# ─── MINDMAP: Life Routine ────────────────────────────────────

# Model your daily routine as a flow
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


# ─── SEQUENCE: User Login ────────────────────────────────────

# Sequence arrows show message passing between actors
User > Browser : Opens app
Browser > API : POST /login
API > AuthService : Validate credentials
AuthService > DB : SELECT user
DB > AuthService : User record
AuthService > API : JWT token
API > Browser : 200 OK + token
Browser > User : Dashboard loaded


# ─── SEQUENCE: Shotext Pipeline ──────────────────────────────

diamem > Renderer : DSL text
Renderer > FileSystem : PNG export
FileSystem > Shotext : File watcher detects new image
Shotext > OCR : Extract text from context footer
OCR > Index : Store searchable entry
Index > User : Diagram found via search


# ─── MINDMAP: ADHD Task Breakdown ────────────────────────────

# Break a big task into small, doable chunks
# Uses @ header grouping (simpler than [Group] { ... })
@ Phase1: Scaffold, Parser, BasicUI
@ Phase2: SVGRender, LivePreview
@ Phase3: PNGExport, ShotextLink
@ Phase4: Themes, Shortcuts, Polish

# Chain connections: whole flow in fewer lines
Scaffold -> Parser -> BasicUI -> SVGRender -> LivePreview
LivePreview -> PNGExport -> ShotextLink -> Themes -> Shortcuts -> Polish


# ─── MIXED: Microservice Communication ──────────────────────

@ Gateway: APIGateway
@ Services: UserSvc, OrderSvc, PaymentSvc
@ Infra: Kafka, Postgres, Redis

# -(label)> is the cleaner labeled syntax
APIGateway -(REST)> UserSvc
APIGateway -(REST)> OrderSvc
OrderSvc -(event)> Kafka
Kafka -(consumes)> PaymentSvc

# Classic -[label]-> still works too
UserSvc -[reads]-> Postgres
OrderSvc -[reads]-> Postgres
PaymentSvc -[caches]-> Redis

# Sequence showing a checkout flow
User > APIGateway : POST /checkout
APIGateway > OrderSvc : Create order
OrderSvc > Kafka : OrderCreated event
Kafka > PaymentSvc : Process payment
PaymentSvc > OrderSvc : PaymentConfirmed
OrderSvc > APIGateway : Order complete
APIGateway > User : 200 OK

