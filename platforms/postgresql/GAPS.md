# Postgresql — Integration Gaps

## All Database Operations
- **Why omitted:** Database drivers use custom binary TCP protocols rather than HTTP. FerroFlux's `http_client` primitive is built for REST APIs.
- **Value:** High.
- **Unblocked by:** Creation of dedicated `sql_client` or `mongo_client` primitives.
