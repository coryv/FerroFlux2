# Microsoft Excel Integration Guide

Connects to the Microsoft Graph API for managing workbooks and worksheets in your OneDrive or SharePoint.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Azure App Registrations](https://portal.azure.com/).
2. Add the required Microsoft Graph permissions: `Files.ReadWrite`, `Sites.ReadWrite.All`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `rows.append`
Appends a new row to an Excel table or range.
- **Key Inputs**: 
    - `itemId`: The ID or path of the Excel file.
    - `tableName` / `range`: The name of the table or a range like `Sheet1!A1:C1`.
    - `values`: An array of values to append (e.g., `["Jane Doe", 25, "active"]`).
- **Outputs**: 
    - `result`: The API response.

### `rows.list`
Lists rows in an Excel table.
- **Key Inputs**: 
    - `itemId`, `tableName`.
- **Outputs**: 
    - `rows`: An array of row objects.

### `sheets.list`
Lists all worksheets in a workbook.

## Examples (WAML)

### Appending to a Table
```waml
- step: log_transaction
  call: excel.rows.append
  with:
    itemId: "01ABC...XYZ"
    tableName: "Transactions"
    values: [inputs.date, inputs.amount, inputs.description]
```

### Retrieving Rows
```waml
- step: fetch_data
  call: excel.rows.list
  with:
    itemId: "01ABC...XYZ"
    tableName: "Inventory"
```
```
