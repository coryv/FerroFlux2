# WooCommerce Integration Guide

Connects to the WooCommerce REST API to manage store orders, products, and customers.

## Setup & Authentication
1. Generate an API Consumer Key and Consumer Secret in your [WooCommerce Settings](https://woo.com/document/woocommerce-rest-api/).
2. Add the required WooCommerce permissions: `Read/Write`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(CONSUMER_KEY:CONSUMER_SECRET)`).
4. Set the `config.base_url` to `https://<shop_url>/wp-json/wc/v3`.

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
