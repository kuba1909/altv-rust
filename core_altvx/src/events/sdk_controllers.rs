use std::{cell::RefCell, ptr::NonNull};

use altv_sdk::ffi as sdk;
use autocxx::prelude::*;
use lazycell::LazyCell;

use crate::{
    base_objects::{col_shape, player, vehicle},
    exports::{AnyEntity, Vector3},
    helpers::{read_cpp_vector3, Hash},
    mvalue,
    resource::Resource,
    VoidResult,
};

use super::helpers::{
    base_event_to_specific, get_entity_from_event, get_non_null_entity_from_event,
    get_player_from_event, get_vehicle_from_event,
};

#[derive(Debug)]
pub struct PlayerConnect {
    pub player: player::PlayerContainer,
}

impl PlayerConnect {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        Self {
            player: get_player_from_event(
                sdk::events::to_CPlayerConnectEvent(event),
                resource,
                sdk::CPlayerConnectEvent::GetTarget,
            ),
        }
    }
}

#[derive(Debug)]
pub struct PlayerDisconnect {
    pub player: player::PlayerContainer,
    pub reason: String,
}

impl PlayerDisconnect {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let event = sdk::events::to_CPlayerDisconnectEvent(event);
        Self {
            player: get_player_from_event(event, resource, sdk::CPlayerDisconnectEvent::GetTarget),
            reason: sdk::CPlayerDisconnectEvent::GetReason(event).to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ServerStarted {}

impl ServerStarted {
    pub(crate) unsafe fn new(_: altv_sdk::CEventPtr, _: &Resource) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct ColshapeEvent {
    pub col_shape: col_shape::ColShapeMutPtr,
    pub entity: AnyEntity,
    pub state: bool,
}

impl ColshapeEvent {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let event = base_event_to_specific!(event, CColShapeEvent);

        let state = sdk::CColShapeEvent::GetState(event);

        let col_shape = sdk::CColShapeEvent::GetTarget(event);
        let col_shape = NonNull::new(col_shape).unwrap();

        Self {
            col_shape,
            entity: get_non_null_entity_from_event(event, resource, sdk::CColShapeEvent::GetEntity),
            state,
        }
    }
}

#[derive(Debug)]
pub struct ServerScriptEvent {
    pub name: String,
    event: *const sdk::alt::CServerScriptEvent,
    args: LazyCell<mvalue::MValueList>,
}

impl ServerScriptEvent {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, _: &Resource) -> Self {
        let event = base_event_to_specific!(event, CServerScriptEvent);
        let name = sdk::CServerScriptEvent::GetName(event).to_string();
        Self {
            name,
            event,
            args: LazyCell::new(),
        }
    }

    pub fn args(&self) -> &mvalue::MValueList {
        if self.args.filled() {
            self.args.borrow().unwrap()
        } else {
            let args = unsafe { sdk::CServerScriptEvent::GetArgs(self.event) };
            self.args
                .fill(Resource::with(|v| mvalue::deserialize_mvalue_args(args, v)))
                .unwrap();
            self.args.borrow().unwrap()
        }
    }
}

#[derive(Debug)]
pub struct ClientScriptEvent {
    pub name: String,
    pub player: player::PlayerContainer,
    event: *const sdk::alt::CClientScriptEvent,
    args: LazyCell<mvalue::MValueList>,
}

impl ClientScriptEvent {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let event = base_event_to_specific!(event, CClientScriptEvent);
        let name = sdk::CClientScriptEvent::GetName(event).to_string();
        let player = get_player_from_event(event, resource, sdk::CClientScriptEvent::GetTarget);

        Self {
            name,
            event,
            player,
            args: LazyCell::new(),
        }
    }

    pub fn args(&self) -> &mvalue::MValueList {
        if self.args.filled() {
            self.args.borrow().unwrap()
        } else {
            let args = unsafe { sdk::CClientScriptEvent::GetArgs(self.event) };
            self.args
                .fill(Resource::with(|v| mvalue::deserialize_mvalue_args(args, v)))
                .unwrap();
            self.args.borrow().unwrap()
        }
    }
}

#[derive(Debug)]
pub struct ConsoleCommandEvent {
    pub name: String,
    pub args: Vec<String>,
}

impl ConsoleCommandEvent {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, _: &Resource) -> Self {
        let event = sdk::events::to_CConsoleCommandEvent(event);
        Self {
            name: sdk::CConsoleCommandEvent::GetName(event).to_string(),
            args: sdk::CConsoleCommandEvent::GetArgs(event)
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct WeaponDamageEvent {
    pub source: player::PlayerContainer,
    pub target: AnyEntity,
    pub weapon_hash: Hash,
    pub body_part: altv_sdk::PlayerBodyPart,
    pub damage: u32,
    pub shot_offset: Vector3,

    event: *mut sdk::alt::CWeaponDamageEvent,
    base_event: altv_sdk::CEventPtr,
    custom_damage_set: RefCell<bool>,
}

impl WeaponDamageEvent {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let weapon_event = base_event_to_specific!(event, CWeaponDamageEvent);

        Self {
            source: get_player_from_event(
                weapon_event,
                resource,
                sdk::CWeaponDamageEvent::GetSource,
            ),
            target: get_non_null_entity_from_event(
                weapon_event,
                resource,
                sdk::CWeaponDamageEvent::GetTarget,
            ),
            weapon_hash: sdk::CWeaponDamageEvent::GetWeaponHash(weapon_event),
            body_part: {
                let raw = sdk::CWeaponDamageEvent::GetBodyPart(weapon_event);
                altv_sdk::PlayerBodyPart::from(raw).unwrap()
            },
            damage: sdk::CWeaponDamageEvent::GetDamageValue(weapon_event),
            shot_offset: {
                let raw = sdk::CWeaponDamageEvent::GetShotOffset(weapon_event).within_unique_ptr();
                read_cpp_vector3(raw)
            },

            // internal properties
            base_event: event,
            event: weapon_event,
            custom_damage_set: RefCell::new(false),
        }
    }

    pub fn set_damage(&self, value: u32) -> VoidResult {
        if *self.custom_damage_set.try_borrow()? {
            anyhow::bail!("Damage cannot be set multiple times")
        }
        *self.custom_damage_set.try_borrow_mut()? = true;
        unsafe { sdk::CWeaponDamageEvent::SetDamageValue(self.event, value) }
        Ok(())
    }

    pub fn cancel(&self) -> VoidResult {
        if unsafe { sdk::CEvent::WasCancelled(self.base_event) } {
            anyhow::bail!("Event cannot be cancelled multiple times")
        } else {
            unsafe { sdk::CEvent::Cancel(self.base_event) }
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct PlayerDeath {
    pub player: player::PlayerContainer,
    pub killer: Option<AnyEntity>,
    pub weapon_hash: Hash,
}

impl PlayerDeath {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let event = base_event_to_specific!(event, CPlayerDeathEvent);

        Self {
            player: get_player_from_event(event, resource, sdk::CPlayerDeathEvent::GetTarget),
            killer: get_entity_from_event(sdk::CPlayerDeathEvent::GetKiller(event), resource),
            weapon_hash: sdk::CPlayerDeathEvent::GetWeapon(event),
        }
    }
}

#[derive(Debug)]
pub struct PlayerDamage {
    pub player: player::PlayerContainer,
    pub attacker: Option<AnyEntity>,
    pub health_damage: u16,
    pub armour_damage: u16,
}

impl PlayerDamage {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let event = base_event_to_specific!(event, CPlayerDamageEvent);

        Self {
            player: get_player_from_event(event, resource, sdk::CPlayerDamageEvent::GetTarget),
            attacker: get_entity_from_event(sdk::CPlayerDamageEvent::GetAttacker(event), resource),
            health_damage: sdk::CPlayerDamageEvent::GetHealthDamage(event),
            armour_damage: sdk::CPlayerDamageEvent::GetArmourDamage(event),
        }
    }
}

macro_rules! player_enter_or_leave_vehicle {
    ($event: ident) => {
        pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
            let event = base_event_to_specific!(event, $event);

            Self {
                player: get_player_from_event(event, resource, sdk::$event::GetPlayer),
                vehicle: get_vehicle_from_event(sdk::$event::GetTarget(event), resource),
                seat: sdk::$event::GetSeat(event),
            }
        }
    };
}

#[derive(Debug)]
pub struct PlayerEnteringVehicle {
    pub player: player::PlayerContainer,
    pub vehicle: vehicle::VehicleContainer,
    pub seat: u8,
}

impl PlayerEnteringVehicle {
    player_enter_or_leave_vehicle!(CPlayerEnteringVehicleEvent);
}

#[derive(Debug)]
pub struct PlayerEnterVehicle {
    pub player: player::PlayerContainer,
    pub vehicle: vehicle::VehicleContainer,
    pub seat: u8,
}

impl PlayerEnterVehicle {
    player_enter_or_leave_vehicle!(CPlayerEnterVehicleEvent);
}

#[derive(Debug)]
pub struct PlayerLeaveVehicle {
    pub player: player::PlayerContainer,
    pub vehicle: vehicle::VehicleContainer,
    pub seat: u8,
}

impl PlayerLeaveVehicle {
    player_enter_or_leave_vehicle!(CPlayerLeaveVehicleEvent);
}

#[derive(Debug)]
pub struct PlayerChangeVehicleSeat {
    pub player: player::PlayerContainer,
    pub vehicle: vehicle::VehicleContainer,
    pub new_seat: u8,
    pub old_seat: u8,
}

impl PlayerChangeVehicleSeat {
    pub(crate) unsafe fn new(event: altv_sdk::CEventPtr, resource: &Resource) -> Self {
        let event = base_event_to_specific!(event, CPlayerChangeVehicleSeatEvent);

        Self {
            player: get_player_from_event(
                event,
                resource,
                sdk::CPlayerChangeVehicleSeatEvent::GetPlayer,
            ),
            vehicle: get_vehicle_from_event(
                sdk::CPlayerChangeVehicleSeatEvent::GetTarget(event),
                resource,
            ),
            new_seat: sdk::CPlayerChangeVehicleSeatEvent::GetNewSeat(event),
            old_seat: sdk::CPlayerChangeVehicleSeatEvent::GetOldSeat(event),
        }
    }
}
