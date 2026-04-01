# Open Meteo Integration Guide

Connects to the Open Meteo API for free, non-commercial weather data, forecasts, and historical records.

## Setup & Authentication
Open Meteo is free for non-commercial use and does note require an API key by default.
1. In FerroFlux, use the `open-meteo` platform directly.
2. The `config.base_url` is set to `https://api.open-meteo.com/v1`.

## Available Actions

### `forecast.daily`
Retrieves a daily weather forecast for a specific location.
- **Key Inputs**: 
    - `latitude`: (Number) Latitude decimal.
    - `longitude`: (Number) Longitude decimal.
    - `daily`: (Optional) Comma-separated list of fields (e.g., `temperature_2m_max,precipitation_sum`).
- **Outputs**: 
    - `daily`: The forecast data.

### `current.get`
Retrieves current weather conditions.
- **Key Inputs**: `latitude`, `longitude`, `current_weather` (boolean).

### `geocoding.search`
Search for coordinates by city name.
- **Key Inputs**: `name`.

### `historical.get`
Retrieves historical weather data for a specific date range.

## Examples (WAML)

### Getting Daily High Temperature
```waml
- step: weather_check
  call: open-meteo.forecast.daily
  with:
    latitude: inputs.lat
    longitude: inputs.lon
    daily: "temperature_2m_max"
```

### Searching for a City
```waml
- step: city_lookup
  call: open-meteo.geocoding.search
  with:
    name: "New York"
```
```
