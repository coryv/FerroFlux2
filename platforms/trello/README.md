# Trello Integration Guide

Connects to the Trello API for managing boards, lists, and cards in your workspace.

## Setup & Authentication
1. Generate an API Key and Token in the [Trello Developer Portal](https://trello.com/app-key).
2. In FerroFlux, create a new Connection and add the following:
    - `key`: `YOUR_API_KEY`
    - `token`: `YOUR_API_TOKEN`

## Available Actions

### `cards.create`
Creates a new card in a specific list.
- **Key Inputs**: 
    - `idList`: List ID.
    - `name`: Card title.
    - `desc`: (Optional) Card description.
    - `due`: (Optional) Due date.
- **Outputs**: 
    - `card`: The created card object.

### `cards.update`
Updates an existing card's fields or status.
- **Key Inputs**: 
    - `idCard`: The ID of the card.
    - `name`, `desc`, `closed` (boolean).

### `lists.create`
Creates a new list on a specific board.

### `comments.create`
Adds a comment to a card.

### `labels.add`
Adds a label to a card.

## Examples (WAML)

### Creating a Card
```waml
- step: create_todo
  call: trello.cards.create
  with:
    idList: "123456789"
    name: "Complete documentation"
    desc: "Finish all integration guides."
```

### Moving a Card
```waml
- step: move_to_done
  call: trello.cards.update
  with:
    idCard: "987654321"
    idList: "DONE_LIST_ID"
```
```
