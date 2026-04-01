# Stripe Integration Guide

Connects to the Stripe API for managing payments, customers, and subscriptions.

## Setup & Authentication
1. Generate a Secret Key in your [Stripe Dashboard](https://dashboard.stripe.com/apikeys).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_SECRET_KEY`).
3. Set the `Stripe-Version` header to the desired API version.

## Available Actions

### `customers.create`
Creates a new customer in Stripe.
- **Key Inputs**: 
    - `email`: Customer email.
    - `name`: Customer name.
    - `description`: (Optional) Customer details.
- **Outputs**: 
    - `customer`: The created customer object.

### `charges.create`
Creates a new charge for a customer.
- **Key Inputs**: 
    - `amount`: (Number) Amount in the smallest currency unit (e.g., `100` for $1.00).
    - `currency`: (e.g., `usd`).
    - `customer`: (Optional) Customer ID.
    - `source`: (Optional) Payment source ID.
- **Outputs**: 
    - `charge`: The created charge object.

## Examples (WAML)

### Creating a Customer
```waml
- step: new_customer
  call: stripe.customers.create
  with:
    email: inputs.user_email
    name: inputs.user_name
```

### Charging a Card
```waml
- step: process_payment
  call: stripe.charges.create
  with:
    amount: inputs.total_cents
    currency: "usd"
    customer: steps.new_customer.id
```
```
