#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::{api::time, error, storage};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    {BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable},
};
use std::cell::RefCell;
use std::collections::BTreeMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct GreenSpace {
    id: u64,
    name: String,
    location: String,
    description: String,
}

impl Storable for GreenSpace {
    fn to_bytes(&self) -> Vec<u8> {
        Encode!(self).unwrap()
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Decode!(bytes.as_slice(), Self).unwrap()
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
        IdCell::init(
            GREEN_SPACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0,
        )
        .expect("Cannot create a counter for green spaces"),
    );

    static GREEN_SPACE_STORAGE: RefCell<StableBTreeMap<u64, GreenSpace, Memory>> =
        RefCell::new(StableBTreeMap::init(
            GREEN_SPACE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        ));
}

fn do_insert_green_space(space: &GreenSpace) {
    GREEN_SPACE_STORAGE.with(|service| {
        service.borrow_mut().insert(space.id, space.clone());
    });
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct GreenSpaceUpdatePayload {
    name: String,
    location: String,
    description: String,
}

#[ic_cdk::update]
fn add_green_space(space: GreenSpaceUpdatePayload) -> Option<GreenSpace> {
    let id = GREEN_SPACE_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        let current_value = *counter.get();
        counter.set(current_value + 1);
        current_value + 1
    });

    let green_space = GreenSpace {
        id,
        name: space.name.clone(),
        location: space.location.clone(),
        description: space.description.clone(),
    };

    if validate_green_space(&green_space) {
        do_insert_green_space(&green_space);
        Some(green_space)
    } else {
        None
    }
}

#[ic_cdk::query]
fn get_green_space(id: u64) -> Result<GreenSpace, Error> {
    match _get_green_space(&id) {
        Some(space) => Ok(space),
        None => Err(Error::NotFound {
            msg: format!("A green space with id={} not found", id),
        }),
    }
}

fn validate_green_space(space: &GreenSpace) -> bool {
    // Implement validation logic for green space data
    // For example, ensure names, locations, and descriptions are not empty
    !space.name.is_empty() && !space.location.is_empty() && !space.description.is_empty()
}

fn _get_green_space(id: &u64) -> Option<GreenSpace> {
    GREEN_SPACE_STORAGE.with(|s| s.borrow().get(id).cloned())
}

#[ic_cdk::update]
fn update_green_space(id: u64, payload: GreenSpaceUpdatePayload) -> Result<GreenSpace, Error> {
    match GREEN_SPACE_STORAGE.with(|service| service.borrow_mut().get_mut(&id)) {
        Some(space) => {
            space.name = payload.name.clone();
            space.location = payload.location.clone();
            space.description = payload.description.clone();
            do_insert_green_space(space);
            Ok(space.clone())
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a green space with id={}. Space not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_green_space(id: u64) -> Result<GreenSpace, Error> {
    match GREEN_SPACE_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(space) => Ok(space),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a green space with id={}. Space not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_all_green_spaces() -> Result<Vec<GreenSpace>, Error> {
    Ok(GREEN_SPACE_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .map(|(_, item)| item.clone())
            .collect()
    }))
}

#[ic_cdk::query]
fn search_green_spaces_by_name(name: String) -> Result<Vec<GreenSpace>, Error> {
    Ok(GREEN_SPACE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, space)| {
                if space.name.contains(&name) {
                    Some(space.clone())
                } else {
                    None
                }
            })
            .collect()
    }))
}

#[ic_cdk::query]
fn search_green_spaces_by_description(keyword: String) -> Result<Vec<GreenSpace>, Error> {
    Ok(GREEN_SPACE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, space)| {
                if space.description.contains(&keyword) {
                    Some(space.clone())
                } else {
                    None
                }
            })
            .collect()
    }))
}

#[ic_cdk::update]
fn update_green_space_location(id: u64, new_location: String) -> Result<GreenSpace, Error> {
    match GREEN_SPACE_STORAGE.with(|service| service.borrow_mut().get_mut(&id)) {
        Some(space) => {
            space.location = new_location.clone();
            do_insert_green_space(space);
            Ok(space.clone())
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
    Ok(GREEN_SPACE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, space)| {
                if space.location.contains(&location) {
                    Some(space.clone())
                } else {
                    None
                }
            })
            .collect()
    }))
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    #[serde(rename = "NotFound")]
    NotFound { msg: String },
}

// Export Candid interface definitions for the canister
ic_cdk::export_candid!();
