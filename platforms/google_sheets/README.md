# Google Sheets Integration Guide

Connects to the Google Sheets API to read and write rows in spreadsheets.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Google Cloud Console](https://console.cloud.google.com/).
2. Add the required Google Sheets permissions: `https://www.googleapis.com/auth/spreadsheets`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `values.append`
Appends a new row to a spreadsheet.
- **Key Inputs**: 
    - `spreadsheetId`: The ID of the spreadsheet.
    - `range`: The sheet name or range (e.g., `Sheet1!A1:C1`).
    - `values`: An array of values to append (e.g., `["Jane Doe", 25, "active"]`).
- **Outputs**: 
    - `result`: The API response.

### `values.get`
Retrieves rows from a spreadsheet.
- **Key Inputs**: 
    - `spreadsheetId`, `range`.
- **Outputs**: 
    - `values`: An array of row objects.

### `values.update`
Updates an existing row or cell in a spreadsheet.

### `values.clear`
Clears values in a spreadsheet range.

## Examples (WAML)

### Appending to a Sheet
```waml
- step: log_transaction
  call: google_sheets.values.append
  with:
    spreadsheetId: "1pABC...XYZ"
    range: "Transactions!A1:C1"
    values: [inputs.date, inputs.amount, inputs.description]
```

### Retrieving Rows
```waml
- step: fetch_data
  call: google_sheets.values.get
  with:
    spreadsheetId: "1pABC...XYZ"
    range: "Inventory"
```
```
