# Food Item Management Canister

This Rust code defines a canister for managing food items. The canister provides functionalities to add, update, delete, and retrieve information about food items. It also includes additional features like listing all food items, searching by name, and performing queries based on quantity thresholds.

## Dependencies
- `serde`: Serialization and deserialization library for Rust.
- `candid`: Library for working with Candid, the interface description language used in the Internet Computer.
- `ic_cdk`: Library providing APIs for interacting with the Internet Computer.
- `ic_stable_structures`: Stable storage structures for persistent data.

## Data Structures

### `FoodItem`
A struct representing a food item with attributes such as ID, name, quantity, creation date, and expiration date.

### `FoodItemPayload`
A simplified version of `FoodItem`, used for creating new food items without specifying the ID, creation date, and expiration date.

### `ExcessFoodSharePayload`
A payload structure for sharing excess food, including the food ID and quantity.

## Storage Management

The canister uses stable storage structures, including a `StableBTreeMap`, to persistently store food items. The storage is thread-local, and a `MemoryManager` is employed to manage virtual memory.

## Core Functions

### `add_food_item`
Adds a new food item to the storage with automatically generated ID, creation date, and expiration date.

### `update_food_item`
Updates the attributes of an existing food item based on the provided ID.

### `delete_food_item`
Deletes a food item based on the provided ID.

### `get_food_item`
Retrieves detailed information about a specific food item based on its ID.

### `list_all_food_items`
Returns a list of all stored food items.

### `search_food_items_by_name`
Searches and returns food items that match a given name.

### `get_total_food_quantity`
Returns the total quantity of all food items.

### `get_food_items_above_quantity` and `get_food_items_below_quantity`
Retrieve food items with a quantity above or below a specified threshold.

### `get_average_food_quantity`
Calculates and returns the average quantity of all food items.

### `check_expiration_status`
Checks and returns the expiration status (expired or not expired) of a food item based on its ID.

### `clear_expired_food_items`
Removes expired food items from the storage.

## Candid Interface

The canister exports its Candid interface definitions using the `ic_cdk::export_candid!()` macro.

## Error Handling

Errors are represented using the `Error` enum, which includes a `NotFound` variant with a descriptive message.

Feel free to explore and integrate this canister into your Internet Computer project for efficient food item management!