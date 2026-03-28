# Google Sheets — Integration Gaps

## Format Cells & Complex Updates
- **Why omitted:** Google Sheets `batchUpdate` endpoint uses deeply nested protobuf-based JSON requests which do not translate cleanly to our current input schemas.
- **API endpoint:** `POST /spreadsheets/{spreadsheetId}:batchUpdate`
- **Docs:** https://developers.google.com/sheets/api/reference/rest/v4/spreadsheets/batchUpdate
- **Value:** High. 

## Triggers (New Row, Updated Row, New Spreadsheet)
- **Why omitted:** Sheets API relies on polling cells/ranges without standard webhooks or simple list cursors. Native integration requires polling mechanisms or Google Drive Push Notifications configured with a persistent server.
- **Value:** High.
