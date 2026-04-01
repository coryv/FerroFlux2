# Google Calendar Integration Guide

Connects to the Google Calendar API to manage events and schedule meetings.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Google Cloud Console](https://console.cloud.google.com/).
2. Add the required Google Calendar permissions: `https://www.googleapis.com/auth/calendar.events`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `events.create`
Creates a new event in a specific calendar.
- **Key Inputs**: 
    - `calendar_id`: The ID of the calendar (default is `primary`).
    - `summary`: Event title.
    - `start`: JSON object with `dateTime` and `timeZone`.
    - `end`: JSON object with `dateTime` and `timeZone`.
- **Outputs**: 
    - `event`: The created event object.

### `events.get`
Retrieves a single event by its ID.
- **Key Inputs**: `calendar_id`, `event_id`.

## Examples (WAML)

### Creating a Meeting
```waml
- step: schedule_sync
  call: google_calendar.events.create
  with:
    calendar_id: "primary"
    summary: "Sync with " + inputs.client_name
    start:
      dateTime: inputs.start_time
      timeZone: "UTC"
    end:
      dateTime: inputs.end_time
      timeZone: "UTC"
```

### Retrieving Event Details
```waml
- step: check_meeting
  call: google_calendar.events.get
  with:
    calendar_id: "primary"
    event_id: "evt_123456789"
```
```
