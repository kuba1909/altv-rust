// auto-generated from build.rs

#![allow(non_camel_case_types)]
use primitive_enum::primitive_enum;
primitive_enum! { EventType u16 ;
    None,
    // server
    ServerStarted,
    // shared
    PlayerConnect,
    PlayerBeforeConnect,
    PlayerDisconnect,
    PlayerConnectDenied,
    PlayerSpawn,
    ConnectionQueueAdd,
    ConnectionQueueRemove,
    ResourceStart,
    ResourceStop,
    ResourceError,
    ServerScriptEvent,
    ClientScriptEvent,
    MetaChange,
    SyncedMetaChange,
    StreamSyncedMetaChange,
    GlobalMetaChange,
    GlobalSyncedMetaChange,
    LocalSyncedMetaChange,
    PlayerDamage,
    PlayerDeath,
    FireEvent,
    ExplosionEvent,
    StartProjectileEvent,
    WeaponDamageEvent,
    VehicleDestroy,
    VehicleDamage,
    CheckpointEvent,
    ColshapeEvent,
    PlayerEnterVehicle,
    PlayerEnteringVehicle,
    PlayerLeaveVehicle,
    PlayerChangeVehicleSeat,
    PlayerWeaponChange,
    PlayerRequestControl,
    VehicleAttach,
    VehicleDetach,
    VehicleHorn,
    VehicleSiren,
    NetownerChange,
    CreateBaseObjectEvent,
    RemoveBaseObjectEvent,
    DataNodeReceivedEvent,
    ConsoleCommandEvent,
    PlayerChangeAnimationEvent,
    PlayerChangeInteriorEvent,
    PlayerWeaponShootEvent,
    PlayerDimensionChange,
    // client
    ConnectionComplete,
    DisconnectEvent,
    WebViewEvent,
    KeyboardEvent,
    GameEntityCreate,
    GameEntityDestroy,
    WebSocketClientEvent,
    AudioEvent,
    TaskChange,
    Spawned,
    RmluiEvent,
    WindowFocusChange,
    WindowResolutionChange,
    WorldObjectPositionChange,
    WorldObjectStreamIn,
    WorldObjectStreamOut,
    All,
    Size,
}