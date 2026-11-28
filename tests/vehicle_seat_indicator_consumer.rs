#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Original consumers: Blizzard_UIPanels_Game/Shared/VehicleSeatIndicator.lua
//! (SetupVehicle, Update, OnClick) and Blizzard_UnitFrame PlayerFrame.lua
//! (UNIT_ENTERED_VEHICLE -> UnitInVehicleHidesPetFrame).

use crate::common::blizzard_addon_harness as harness;

const ENTER_AS_GUNNER: &str = r#"
    assert(VehicleSeatIndicator and not VehicleSeatIndicator:IsShown())
    A_Admin.SetVehicleUIIndicator(40, { background = 252256, seats = {
        { seat = 1, x = 0.5, y = 0.25 }, { seat = 2, x = 0.3, y = 0.75 }, { seat = 3, x = 0.7, y = 0.75 } } })
    local background, count = GetVehicleUIIndicator(40)
    assert(background == 252256 and count == 3 and GetVehicleUIIndicator(41) == nil)
    local seat, x, y = GetVehicleUIIndicatorSeat(40, 3)
    assert(seat == 3 and x == 0.7 and y == 0.75 and GetVehicleUIIndicatorSeat(40, 4) == nil)
    A_Admin.EnterVehicle({ indicatorID = 40, vehicleGUID = "Vehicle-0-1-1-1-33062-0000000001",
        playerSeat = 2, showVehicleFrame = true, mayChooseExit = true, hasPitch = false, hidesPetFrame = true, hasComboPoints = false,
        seats = {
            { controlType = "Root", occupant = "Driver", server = "OtherRealm", ejectable = false, canSwitch = true },
            { controlType = "Child", ejectable = false, canSwitch = true },
            { controlType = "None", ejectable = true, canSwitch = true },
        } })
"#;

const SEAT_INDICATOR_FLOW: &str = r#"
    local buttons = VehicleSeatIndicator.buttons
    assert(VehicleSeatIndicator:IsShown() and VehicleSeatIndicator.currSkin == 40)
    assert(buttons[1].virtualID == 1 and buttons[3].virtualID == 3 and buttons[3]:IsShown())
    assert(PlayerFrame.vehicleHidesPet == true, "PlayerFrame read UnitInVehicleHidesPetFrame")
    assert(buttons[2].PlayerIcon:IsShown() and not buttons[2].AllyIcon:IsShown(), "player seat")
    assert(buttons[1].AllyIcon:IsShown() and not buttons[3].AllyIcon:IsShown(), "ally and empty seats")
    local control, name, server, ejectable, canSwitch = UnitVehicleSeatInfo("player", 1)
    assert(control == "Root" and name == "Driver" and server == "OtherRealm")
    assert(ejectable == false and canSwitch == true)
    assert(UnitVehicleSeatCount("player") == 3 and UnitVehicleSeatCount("target") == 0)
    assert(UnitUsingVehicle("player") and not UnitUsingVehicle("target"))
    assert(not PlayerVehicleHasComboPoints())
    assert(not UnitInVehicleControlSeat("player") and CanSwitchVehicleSeat())
    assert(not CanEjectPassengerFromSeat(1), "only the Root occupant ejects")
    buttons[3]:Click("LeftButton")
    assert(buttons[3].PlayerIcon:IsShown() and not buttons[2].PlayerIcon:IsShown(), "switched seat")
    UnitSwitchToVehicleSeat("player", 1)
    assert(select(2, UnitVehicleSeatInfo("player", 1)) == "Driver", "occupied seat refuses switch")
    VehicleExit()
    assert(not VehicleSeatIndicator:IsShown() and UnitVehicleSeatCount("player") == 0)
    assert(UnitVehicleSeatInfo("player", 1) == nil and not UnitUsingVehicle("player"))
    return true
"#;

const DRIVER_EJECTS: &str = r#"
    A_Admin.SetVehicleUIIndicator(41, { background = 252257, seats = {
        { seat = 1, x = 0.5, y = 0.2 }, { seat = 2, x = 0.5, y = 0.8 } } })
    A_Admin.EnterVehicle({ indicatorID = 41, vehicleGUID = "Vehicle-0-1-1-1-28670-0000000002",
        playerSeat = 1, showVehicleFrame = true, mayChooseExit = true, hasPitch = true, hidesPetFrame = false, hasComboPoints = true,
        seats = {
            { controlType = "Root", ejectable = false, canSwitch = false },
            { controlType = "Child", occupant = "Passenger", ejectable = true, canSwitch = false },
        } })
    local buttons = VehicleSeatIndicator.buttons
    assert(VehicleSeatIndicator.currSkin == 41 and buttons[2].AllyIcon:IsShown())
    assert(UnitInVehicleControlSeat() and UnitInVehicleControlSeat("player"))
    assert(not UnitInVehicleHidesPetFrame("player") and not CanSwitchVehicleSeat())
    assert(PlayerVehicleHasComboPoints(), "supplied vehicle combo points")
    assert(CanEjectPassengerFromSeat(2) and not CanEjectPassengerFromSeat(1))
    EjectPassengerFromSeat(2)
    assert(select(2, UnitVehicleSeatInfo("player", 2)) == nil and not buttons[2].AllyIcon:IsShown())
    assert(not CanEjectPassengerFromSeat(2), "empty seat cannot be ejected")
    assert(not pcall(A_Admin.EnterVehicle, { indicatorID = 41, vehicleGUID = "x", playerSeat = 3,
        showVehicleFrame = true, mayChooseExit = true, hasPitch = true, hidesPetFrame = false, hasComboPoints = true,
        seats = { { controlType = "Root", ejectable = false, canSwitch = false } } }), "bad playerSeat")
    assert(not pcall(A_Admin.EnterVehicle, { indicatorID = 41, vehicleGUID = "x", playerSeat = 1,
        showVehicleFrame = true, mayChooseExit = true, hasPitch = true, hidesPetFrame = false, hasComboPoints = true,
        seats = { { controlType = "Turret", ejectable = false, canSwitch = false } } }), "bad controlType")
    return true
"#;

#[test]
fn vehicle_seat_indicator_tracks_seats_switches_and_ejection() {
    harness::with_blizzard_addon_startup_shape(
        &["Blizzard_UIPanels_Game", "Blizzard_UnitFrame", "Blizzard_DurabilityFrame"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
            let gunner_flow = format!("{ENTER_AS_GUNNER}{SEAT_INDICATOR_FLOW}");
            let gunner: bool = env.eval(&gunner_flow).expect("gunner flow");
            let driver: bool = env.eval(DRIVER_EJECTS).expect("driver flow");
            assert!(gunner && driver);
        },
    );
}
