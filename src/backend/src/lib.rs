#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;
// ... (existing imports and types)

// Import necessary libraries and modules

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct GreenSpace {
    id: u64,
    name: String,
    location: String,
    description: String,
}

impl Storable for GreenSpace {
    // Implement Storable trait methods for serialization and deserialization
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for GreenSpace {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static GREEN_SPACE_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static GREEN_SPACE_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(GREEN_SPACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter for green spaces")
    );

    static GREEN_SPACE_STORAGE: RefCell<StableBTreeMap<u64, GreenSpace, Memory>> =
        RefCell::new(StableBTreeMap::init(
            GREEN_SPACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

// Helper method to perform insert for GreenSpace
fn do_insert_green_space(space: &GreenSpace) {
    GREEN_SPACE_STORAGE.with(|service| service.borrow_mut().insert(space.id, space.clone()));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct GreenSpaceUpdatePayload {
    name: String,
    location: String,
    description: String,
}

// Function to add a green space
#[ic_cdk::update]
fn add_green_space(space: GreenSpaceUpdatePayload) -> Option<GreenSpace> {
    let id = GREEN_SPACE_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter for green spaces");

    let green_space = GreenSpace {
        id,
        name: space.name,
        location: space.location,
        description: space.description,
    };

    do_insert_green_space(&green_space);
    Some(green_space)
}

// Function to get a green space by ID
#[ic_cdk::query]
fn get_green_space(id: u64) -> Result<GreenSpace, Error> {
    match _get_green_space(&id) {
        Some(space) => Ok(space),
        None => Err(Error::NotFound {
            msg: format!("A green space with id={} not found", id),
        }),
    }
}

// Internal function to get a green space by ID
fn _get_green_space(id: &u64) -> Option<GreenSpace> {
    GREEN_SPACE_STORAGE.with(|s| s.borrow().get(id))
}

// Function to update a green space
#[ic_cdk::update]
fn update_green_space(id: u64, payload: GreenSpaceUpdatePayload) -> Result<GreenSpace, Error> {
    match GREEN_SPACE_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut space) => {
            space.name = payload.name;
            space.location = payload.location;
            space.description = payload.description;
            do_insert_green_space(&space);
            Ok(space)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't update a green space with id={}. Space not found",
                id
            ),
        }),
    }
}

// Function to delete a green space
#[ic_cdk::update]
fn delete_green_space(id: u64) -> Result<GreenSpace, Error> {
    match GREEN_SPACE_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(space) => Ok(space),
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't delete a green space with id={}. Space not found",
                id
            ),
        }),
    }
}

// Function to get all green spaces
#[ic_cdk::query]
fn get_all_green_spaces() -> Result<Vec<GreenSpace>, Error> {
    GREEN_SPACE_STORAGE.with(|service| {
        let storage = service.borrow_mut();
        let result: Vec<_> = storage.iter().map(|(_, item)| item.clone()).collect();
        Ok(result)
    })
}

#[ic_cdk::query]
fn search_green_spaces_by_name(name: String) -> Result<Vec<GreenSpace>, Error> {
    GREEN_SPACE_STORAGE.with(|service| {
        let borrow = service.borrow();
        let result: Vec<_> = borrow
            .iter()
            .filter_map(|(_, space)| {
                if space.name.contains(&name) {
                    Some(space.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(result)
    })
}

#[ic_cdk::query]
fn search_green_spaces_by_description(keyword: String) -> Result<Vec<GreenSpace>, Error> {
    GREEN_SPACE_STORAGE.with(|service| {
        let borrow = service.borrow();
        let result: Vec<_> = borrow
            .iter()
            .filter_map(|(_, space)| {
                if space.description.contains(&keyword) {
                    Some(space.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(result)
    })
}

#[ic_cdk::update]
fn update_green_space_location(id: u64, new_location: String) -> Result<GreenSpace, Error> {
    match GREEN_SPACE_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut space) => {
            space.location = new_location;
            do_insert_green_space(&space);
            Ok(space)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Couldn't update location for green space with id={}. Space not found",
                id
            ),
        }),
    }
}

#[ic_cdk::query]
fn get_green_space_count() -> Result<u64, Error> {
    Ok(GREEN_SPACE_STORAGE.with(|service| service.borrow().len() as u64))
}

#[ic_cdk::query]
fn search_green_spaces_by_location(location: String) -> Result<Vec<GreenSpace>, Error> {
    GREEN_SPACE_STORAGE.with(|service| {
        let borrow = service.borrow();
        let result: Vec<_> = borrow
            .iter()
            .filter_map(|(_, space)| {
                if space.location.contains(&location) {
                    Some(space.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(result)
    })
}

// Enum for error handling
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// Export Candid interface definitions for the canister
ic_cdk::export_candid!();
