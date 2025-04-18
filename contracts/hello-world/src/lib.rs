#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, String};

// Item status options
#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub enum ItemStatus {
    Listed,
    Sold,
    Unlisted,
}

// Item struct to store all item details
#[contracttype]
#[derive(Clone)]
pub struct Item {
    pub id: u64,                  // Unique item ID
    pub seller: Address,          // Seller of the item
    pub price: i128,              // Fixed price of the item
    pub description: String,      // Description of the item
    pub status: ItemStatus,       // Current status of the item
    pub buyer: Option<Address>,   // Buyer of the item, if sold
    pub list_time: u64,           // Timestamp when the item was listed
    pub expiry_time: u64,         // Expiry timestamp for the listing
}

// Mapping for data keys
#[contracttype]
pub enum DataKey {
    Item(u64),                    // Item ID -> Item
    ItemCounter,                  // Counter for generating unique item IDs
    SellerItems(Address),         // Seller -> Vector of Item IDs
}

#[contract]
pub struct FixedPriceAuctionContract;

#[contractimpl]
impl FixedPriceAuctionContract {
    // List a new item for sale
    pub fn list_item(
        env: Env, 
        seller: Address, 
        price: i128, 
        description: String, 
        duration_seconds: u64
    ) -> u64 {
        // Verify the seller
        seller.require_auth();
        
        // Validate inputs
        if price <= 0 {
            log!(&env, "Price must be greater than zero");
            panic!("Price must be greater than zero");
        }
        
        // Get the next item ID
        let item_counter: u64 = env.storage().instance().get(&DataKey::ItemCounter).unwrap_or(0);
        let item_id = item_counter + 1;
        
        // Calculate listing timestamps
        let current_time = env.ledger().timestamp();
        let expiry_time = current_time + duration_seconds;
        
        // Create new item
        let item = Item {
            id: item_id,
            seller: seller.clone(),
            price,
            description,
            status: ItemStatus::Listed,
            buyer: None,
            list_time: current_time,
            expiry_time,
        };
        
        // Store the item
        env.storage().instance().set(&DataKey::Item(item_id), &item);
        
        // Update the counter
        env.storage().instance().set(&DataKey::ItemCounter, &item_id);
        
        // Extend contract data TTL
        env.storage().instance().extend_ttl(100, 100);
        
        log!(&env, "Item listed with ID: {}", item_id);
        item_id
    }
    
    // Buy an item at the listed price
    pub fn buy_item(env: Env, item_id: u64, buyer: Address) -> bool {
        // Verify the buyer
        buyer.require_auth();
        
        // Get the item
        let mut item: Item = match env.storage().instance().get(&DataKey::Item(item_id)) {
            Some(i) => i,
            None => {
                log!(&env, "Item does not exist");
                panic!("Item does not exist");
            }
        };
        
        // Check if item is still listed
        if item.status != ItemStatus::Listed {
            log!(&env, "Item is no longer available");
            panic!("Item is no longer available");
        }
        
        // Check if listing has expired
        let current_time = env.ledger().timestamp();
        if current_time > item.expiry_time {
            item.status = ItemStatus::Unlisted;
            env.storage().instance().set(&DataKey::Item(item_id), &item);
            log!(&env, "Listing has expired");
            panic!("Listing has expired");
        }
        
        // Prevent seller from buying their own item
        if buyer == item.seller {
            log!(&env, "Seller cannot buy their own item");
            panic!("Seller cannot buy their own item");
        }
        
        // Update item status and buyer
        item.status = ItemStatus::Sold;
        item.buyer = Some(buyer.clone());
        
        // Store updated item
        env.storage().instance().set(&DataKey::Item(item_id), &item);
        
        // At this point, actual token transfer would happen in a production contract
        // but handling actual payments is outside the scope of this example
        
        // Extend contract data TTL
        env.storage().instance().extend_ttl(100, 100);
        
        log!(&env, "Item {} sold to buyer", item_id);
        true
    }
    
    // Unlist an item (only by seller)
    pub fn unlist_item(env: Env, item_id: u64, seller: Address) -> bool {
        // Verify the seller
        seller.require_auth();
        
        // Get the item
        let mut item: Item = match env.storage().instance().get(&DataKey::Item(item_id)) {
            Some(i) => i,
            None => {
                log!(&env, "Item does not exist");
                panic!("Item does not exist");
            }
        };
        
        // Check if the caller is the seller
        if item.seller != seller {
            log!(&env, "Only the seller can unlist this item");
            panic!("Only the seller can unlist this item");
        }
        
        // Check if item is still listed
        if item.status != ItemStatus::Listed {
            log!(&env, "Item is not in listed state");
            panic!("Item is not in listed state");
        }
        
        // Update item status
        item.status = ItemStatus::Unlisted;
        
        // Store updated item
        env.storage().instance().set(&DataKey::Item(item_id), &item);
        
        // Extend contract data TTL
        env.storage().instance().extend_ttl(100, 100);
        
        log!(&env, "Item {} unlisted by seller", item_id);
        true
    }
    
    // View item details
    pub fn view_item(env: Env, item_id: u64) -> Item {
        match env.storage().instance().get(&DataKey::Item(item_id)) {
            Some(item) => item,
            None => {
                log!(&env, "Item does not exist");
                panic!("Item does not exist");
            }
        }
    }
}