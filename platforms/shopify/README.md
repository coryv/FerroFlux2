# Shopify Integration Guide

Connects to the Shopify Admin REST API to manage orders, products, and customers.

## Setup & Authentication
1. Generate an Access Token in your [Shopify Admin Dashboard](https://help.shopify.com/en/manual/apps/app-types/custom-apps).
2. Add the required Shopify permissions: `write_orders`, `write_products`, `write_customers`.
3. In FerroFlux, create a new Connection and add it to the `X-Shopify-Access-Token` header field.
4. Set the `config.base_url` to `https://<shop_name>.myshopify.com/admin/api/2023-10`.

## Available Actions

### `orders.create`
Creates a brand new order in Shopify.
- **Key Inputs**: 
    - `line_items`: Array of line item objects.
    - `customer`: (Optional) Customer object or ID.
- **Outputs**: 
    - `order`: The created order object.

### `products.create` / `products.list`
Creates a new product or lists existing ones.

### `inventory.adjust`
Adjusts inventory levels for a specific location.

### `orders.fulfill`
Marks an order or fulfillment order as fulfilled.

## Available Triggers

### `orders.new`
Fires when a new order is placed in Shopify.
- **Settings**: `poll_interval`.

### `customers.new`
Fires when a new customer is created.

## Examples (WAML)

### Creating an Order
```waml
- step: new_sale
  call: shopify.orders.create
  with:
    line_items:
      - variant_id: "12345678"
        quantity: 1
    customer:
      id: inputs.shopify_customer_id
```

### Listing Products
```waml
- step: fetch_inventory
  call: shopify.products.list
  with:
    limit: 50
```
```
