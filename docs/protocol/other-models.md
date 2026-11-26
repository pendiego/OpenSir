# Other GameSir models: device table survey

Scope: every model in the vendor device table except the C2 detail and firmware update.
Source: observed from the vendor app's behaviour (its device table and main process logic).
Labels: `observed-static` (observed from the vendor app), `observed-hw` (seen on real device), `inferred`.

## How the table is used (observed-static)
- One table holds every model object. The app flattens all `Modes` and all `ConfigMethods` from it, and a helper exposes `isSupportedDevice`, `isSupportedGamepad`, `getConfig`, `devices`, `getDfuPackage`, `getDeviceProxy`.
- `isSupportedDevice`: true when a mode matches VID and PID, its `UsagePage` is not 65535, the caller's usage page (if given) equals it, and `HandleByDll` is not `true`. Such devices use the pure HID path.
- `isSupportedGamepad`: true when a mode matches VID and PID, its `GamepadUsagePage` is not 65535, and it equals the caller's usage page.
- A mode with `HandleByDll: true` (or `UsagePage` 65535) is not on the pure HID path. It goes through a SAPJoy DLL (see "DLL bindings").
- Dongle entries (`*_Dongle`, `*_DG`) have `Modes: []`. They are never selected directly.

## Overview

Transport: "HID" = pure HID (`isSupportedDevice` true). "DLL" = every mode has `HandleByDll: true`.
Driver: the model's own driver class in the vendor app. "bindDevice" = the class accepts a SAPJoy handle.
Similarity and effort are `inferred` from transport, proxy and feature overlap only. The protocol was not compared.

| Model (`Model`) | ProductName | Wired PID(s) | Wireless PID(s) | Transport | bindDevice | features | Similarity to C2 (inferred) | Effort (inferred) |
|---|---|---|---|---|---|---|---|---|
| C2 | CYCLONE 2 | 4125 (0x101D) | 4138 (0x102A) | HID | no | none | - | - |
| T3Lite | Tegenaria Lite | 4243 (0x1093) | - | HID | no | none | High | S |
| K2_8K | K2 8K | 4238 (0x108E) | 4255 (0x109F) | HID | no | ProfileRename, ProfileNameLength 32, StickDirectionMapping | Medium | S |
| T3CE | Tarantula CE | 4157 (0x103D), 2317 (0x090D) | - | HID | no | ProfileRename, ProfileNameLength 32 | Medium | S-M |
| G7ProCE | G7 Pro 8K | 10 PIDs (see section) | 10 PIDs (see section) | HID | no | ProfileRename, ProfileNameLength 32 | Medium-High | M |
| T3Pro | Tarantula Pro | 4176 (0x1050) | 4158 (0x103E) | DLL | yes | none | Medium | L |
| T4NPro | NovaPro | 4110 (0x100E) | 4158 (0x103E) | DLL | yes | none | Medium | L |
| T4N2 | Nova 2 Lite | 4111 (0x100F) | 4248 (0x1098) | DLL | yes | none | Low | L |
| N2S | NOVA VISION | 4417 (0x1141) | 4285 (0x10BD) | DLL | yes | ManualFirmwarePackageUpdate, ProfileRename, ProfileNameLength 32 | Low | L |
| T3XS | T3XS | 4186 (0x105A) | 4286 (0x10BE) | DLL | yes | ManualFirmwarePackageUpdate | Low | L |
| G7se (6 variants) | GameSir-G7 SE/HE Controller for Xbox | - | - | not in HID table | none | none | Low | unknown (Xbox type) |

Summary: 5 HID models (C2 plus 4 others) and 5 DLL models.

## DLL bindings (observed-static)

Two SAPJoy libraries are bound through ffi. Both are loaded from the app's `dfu/` folder.

| Library | init (watch list) | Drivers |
|---|---|---|
| `sapjoyuniapis_x86_v1.0.16` | VID 13623; PIDs 4176, 4158, 4110 | T3Pro, T4NPro |
| `sapjoyuniapis_t4n2_x86_v2606151` | VID 13623; PIDs 4111, 4417, 4186 | T4N2, N2S, T3XS |

Notes:
- Both `init` functions create the SAPJoy instance with `pcwszName: "GameSirConnect"`, `vid: [13623]` and a PID list, then install the device-change callback.
- The T3Pro init also starts a T3Pro update worker after 1 s.
- Wireless PIDs 4248 (T4N2), 4285 (N2S), 4286 (T3XS) are NOT in either watch list. The modes still say `HandleByDll: true`. How those wireless devices reach the DLL is unresolved (`inferred`: a dongle or a different enumeration path).
- The T3Pro/T4NPro watch list includes PID 4158, which is the shared wireless PID.
- The N2S and T3XS drivers log the string "proxyT4N2.bindDevice" in their `bindDevice`. This looks like copy-paste; the classes are separate.

### Export names the app binds (observed-static)
Names only. Argument types are declared in the app's ffi bindings. Behaviour is not verified.

| Group | Exports | In T3Pro lib | In T4N2 lib |
|---|---|---|---|
| Lifecycle | `SAPJoyUni_Create`, `SAPJoyUni_Release`, `SAPJoyUniDevice_GetDevicePID`, `GetHardwareID`, `GetDeviceType`, `GetDeviceInfo`, `GetVersion`, `GetMacAddress`, `SetISPMode` | yes | yes |
| Firmware (out of scope) | `SAPJoyUniDevice_UpdateFirmware`, `IsFirmwareUpdateSupported` | yes | yes |
| Mapping and curves | `Get/SetKeyMappingHSV`, `Get/SetStickMapping`, `Get/SetStickCurveHSV`, `Get/SetMotionSenseMapping`, `Get/SetMotionSenseCurveHSV`, `Get/SetMacroStep`, `Get/SetTriggerParam` | yes | yes |
| Light, vibration, D-pad | `Get/SetLightParam`, `Get/SetLightColorHSV`, `Get/SetVibrateLevel`, `SetVibrateTest`, `Get/SetDPadParam`, `SaveConfigPage` | yes | yes |
| Gamepad state and test | `SetGamepadStateReport`, `SetABXYRotationTest` | yes | yes |
| T4N2 group only | `Get/SetGamepadParam`, `Get/SetReportRate`, `RecordMacro`, `StandLightReset`, `Get/SetCustomCalibration` | no | yes |

- Calibration, macro recording, light set and vibration test are write or action exports. They are not called by this survey.
- `inferred`: the T3Pro and T4N2 DLLs are the protocol path for all their modes. The app never builds HID reports for them.

## Detection and hotplug pipeline (observed-static)

1. Startup and HID hotplug. The app runs a USB watcher. The startup search for VID 13623 only detects G7SE; it does not enumerate devices. Listeners: `add` and `remove` for VID 13623.
2. Add:
   - VID 13623 PIDs in the G7SE list are G7SE. The handler emits `g7seStatechanged` and skips the HID-mode service.
   - If a focus flag is set, skip.
   - VID 19530 goes to a separate handler (body not read).
   - A device matching an entry from `getAllMethods` (contents not checked) fires after 300 ms.
   - Otherwise, if `isSupportedDevice` is true, connect with retry. The retry loop allows up to 5 attempts, 150 ms apart.
3. Device list build. The filter is `isSupportedDevice || isSupportedGamepad`. A new device gets a driver through `getDeviceProxy` and is pushed into the device list. Unknown devices get `ProductName: "Unrecognized"`. For a supported device, `loadProfile` runs; for a gamepad, `loadGamePad` runs.
4. Remove. G7SE is handled first. Otherwise cached HID entries with the same VID and PID (and serial, if given) are removed by path. The removal is ambiguous when several match and no serial is given. Follow-up work is in a 200 ms timeout (body not read).
5. DLL device change (one callback per library). The callback gets (address, connected flag, device id, context):
   - Connected: after 20 ms it reads the hardware ID (`GetHardwareID`). Then it evaluates a predicate on it (not resolved).
     - If the predicate is true: if a driver already exists for that path, call its `bindDevice`; otherwise store the handle keyed by path.
     - If the predicate is false: after 50 ms, read the device-type id, create the driver, and call its `bindDevice`. A handle stored for that path is then bound.
   - Disconnected: drop the stored handle, emit `deviceRemoved` and `disconnected`.

## Per-model sections

### T3Lite (Tegenaria Lite) - observed-static
- ProductName "Tegenaria Lite", ProductType "T3Lite", UdiskProductName "T3_Lite". SpecialMapping is the same as C2. Its driver has no `bindDevice`.
- Modes: one entry. Wired PID 4243 (0x1093), UsagePage 65520, GamepadUsagePage -1, AssetsFolder "T3Lite/White". No DeviceTypeId, no HandleByDll. Pure HID.
- No wireless entry. No `features`.

### K2_8K - observed-static
- ProductName "K2 8K", ProductType "K2_8K", UdiskProductName "GS_K2_DV". SpecialMapping `{}`. Its driver has no `bindDevice`.
- `features`: ProfileRename "enabled", ProfileNameLength 32, StickDirectionMapping "enabled".
- Modes: wired PID 4238 (0x108E) and wireless PID 4255 (0x109F). Both UsagePage 65520, GamepadUsagePage -1, AssetsFolder "K2_8K/Black". Pure HID.
- Dongle `GS_K2_8K_DG`: "K2 8K Dongle", UdiskProductName "GS_K2_DG", `Modes: []`.

### T3CE (Tarantula CE) - observed-static
- ProductName "Tarantula CE", ProductType "T3CE", UdiskProductName "GS_T3CE". SpecialMapping `{}`. Its driver has no `bindDevice`.
- `features`: ProfileRename "enabled", ProfileNameLength 32.
- Modes:
  - Wired PID 4157 (0x103D), UsagePage 65520, GamepadUsagePage -1, AssetsFolder "T3CE/Default". No DeviceTypeId, no HandleByDll. Pure HID.
  - Second wired PID 2317 (0x090D), same usage fields. Its own `features`: KeyboardAndMouse "disabled", MapToMoreKeys "disabled", Turbo "disabled", MappingTabs ["Buttons"], HiddenMenus ["/motion"].
- No wireless entry.

### G7ProCE (G7 Pro 8K) - observed-static
- ProductName "G7 Pro 8K", ProductType "G7ProCE", UdiskProductName "GS_G7_Pro_CE_DV". SpecialMapping `{}`. Its driver has no `bindDevice`.
- `features`: ProfileRename "enabled", ProfileNameLength 32.
- Modes: wired/wireless pairs. All UsagePage 65520, GamepadUsagePage -1, no DeviceTypeId, no HandleByDll. Pure HID.

| AssetsFolder | Wired PID | Wireless PID | Restricted features |
|---|---|---|---|
| G7ProCE/Gold | 4279 | 4281 | no |
| G7ProCE/Gold | 4301 | 4302 | yes |
| G7ProCE/Aimlabs | 4293 | 4294 | no |
| G7ProCE/GoldenApple | 4349 | 4350 | no |
| G7ProCE/Nioh | 4295 | 4296 | no |
| G7ProCE/NBA2K | 4297 | 4298 | no |
| G7ProCE/Gold | 4299 | 4300 | no |
| G7ProCE/Crown | 4146 | 4147 | no |
| G7ProCE/Gold | 4148 | 4149 | no |
| G7ProCE/GradientBlue | 4347 | 4348 | no |

- "Restricted features" = a `features` block: KeyboardAndMouse "disabled", MapToMoreKeys "disabled", Turbo "disabled", MappingTabs ["Buttons"], HiddenMenus ["/motion"].
- Dongle `GS_G7_Pro_CE_DG`: "G7 Pro CE Dongle", `Modes: []`.

### T3Pro (Tarantula Pro) - observed-static, DLL
- ProductName "Tarantula Pro", ProductType "T3Pro". SpecialMapping `{}`. Its driver defines `bindDevice`, which checks `isFirmwareUpdateSupported`. No `features`.
- Modes, both DLL (`HandleByDll: true`):
  - Wired PID 4176 (0x1050), UsagePage 65535, GamepadUsagePage 65535, DeviceTypeId 0, DeviceTypeThemeId 0, AssetsFolder "T3Pro/Black".
  - Wireless PID 4158 (0x103E), UsagePage 65520, GamepadUsagePage -1, DeviceTypeId 0, DeviceTypeThemeId 0. The PID is shared with T4NPro.
- Dongle `T3Pro_Dongle`: UdiskProductName "G760402_V1.0", `Modes: []`.

### T4NPro (NovaPro) - observed-static, DLL
- ProductName "NovaPro", ProductType "T4NPro". SpecialMapping `{}`. Its driver defines `bindDevice`. No `features`.
- Modes: DeviceTypeId 3 on every entry, all DLL:
  - Wired PID 4110 (0x100E), UsagePage 65535, GamepadUsagePage 65535. Three themes: T4NPro/Black (ThemeId 0), T4NPro/HatsuneMiku (ThemeId 2), T4NPro/YYS (ThemeId 3).
  - Wireless PID 4158 (0x103E), UsagePage 65520, GamepadUsagePage -1, one entry per theme (ThemeId 0, 2, 3).

### T4N2 (Nova 2 Lite) - observed-static, DLL
- ProductName "Nova 2 Lite", ProductType "T4N2", UDiskVersionId "N2LT", UdiskProductName "AC695X_TRANS".
- Has its own Mapping; DefaultMapping `{}`, SpecialMapping `{}`. Its driver defines `bindDevice`, which registers a gamepad-state report handler in "Mapped" mode and starts listening for gamepad state. No `features`.
- Wired: PID 4111 (0x100F), UsagePage 65535, GamepadUsagePage 65535, DeviceTypeId 0, HandleByDll true, AssetsFolder "T4N2/White". In the T4N2 watch list.
- Wireless: UDiskVersionId "N2DG", UdiskProductName "AC632N_TRANS", PID 4248 (0x1098), UsagePage 65535, HandleByDll true. Not in the watch list (see DLL notes).
- Dongle `T4N2_Dongle`: "Dongle", UDiskVersionId "N2DG", `Modes: []`.

### N2S (NOVA VISION) - observed-static, DLL
- ProductName "NOVA VISION", ProductType "N2S", UDiskVersionId "N2S\0", UdiskProductName "AC695X_TRANS".
- Its driver defines `bindDevice`.
- `features`: ManualFirmwarePackageUpdate "enabled", ProfileRename "enabled", ProfileNameLength 32.
- Modes: four DeviceTypeId variants (0-3). Each has a wired entry (PID 4417, 0x1141, in the T4N2 watch list) and a wireless entry (PID 4285, 0x10BD, not in it). All UsagePage 65535, GamepadUsagePage 65535, HandleByDll true.
  - AssetsFolders: N2S/White (type 0), N2S/Black (1), N2S/MaiShiranui (2), N2S/Nioh (3). The PIDs are shared across the variants; the variants differ in theme.
- Dongle `N2S_Dongle`: "N2S Dongle", UDiskVersionId "NSDG", `Modes: []`.

### T3XS - observed-static, DLL
- ProductName "T3XS", ProductType "T3XS", UDiskVersionId "T3XS\0", UdiskProductName "AC695X_TRANS". PackedFirmware version 4041 (firmware, out of scope).
- Default mapping `{}`. Special: SelectBtn ["CRO"], CancelBtn ["CIR"], NavigationKeys ["LS", "DPad"]. SpecialMapping `{}`.
- Its driver defines `bindDevice`.
- `features`: ManualFirmwarePackageUpdate "enabled".
- Modes, both DLL:
  - Wired PID 4186 (0x105A), UsagePage 65535, GamepadUsagePage 65535, DeviceTypeId 0, AssetsFolder "T3XS/White". In the T4N2 watch list.
  - Wireless PID 4286 (0x10BE), same fields. Not in the watch list.

### G7se (GameSir-G7 SE/HE Controller for Xbox) - observed-static
- Six entries, ProductType "JS3101". UdiskProductName varies (JS_SL3101, _Orange, _Black, _Blue, _Gray, _Purple). Has a PackedFirmware entry. `Modes: []`. No driver; not driven by the HID table.
- Xbox-type controller. The G7SE PIDs are handled by the hotplug add path and skipped for HID mode services.
- `inferred`: the Xbox/XInput route. The app loads an XInput helper library (`XInputGetCaps`, `IsConnected`, `Rumble`). Not verified against G7se.

## Dongles (not driven directly, observed-static)
- `GS_C2_Dongle`: "C2 Dongle", `Modes: []`.
- `T3Pro_Dongle`, `T4N2_Dongle`, `N2S_Dongle`, `GS_G7_Pro_CE_DG`, `GS_K2_8K_DG`.

## DFU (one paragraph, observed-static; out of scope, not exercised)
Firmware packages live in the app's `dfu/` folder. The T3Pro path starts a worker and posts `startDfu` with the package path. The T4N2 path calls `updateFirmware`. It takes a package path and an optional `WhiteListPids` list and runs the DLL's `UpdateFirmware` export. Other models use JL upgrade helpers. No firmware or DFU call was made in this survey. The read-only constraint on the hardware was kept.

## Not yet covered
- Renderer menus per model (which pages and entries appear for each `features` set). Only the C2 pages and the route table were read.
- The wireless PIDs 4248, 4285 and 4286: how they reach the DLL, given they are not in the watch lists.
- The hardware-ID predicate in the DLL device-change callback, and the entries returned by `getAllMethods`.
- Heartbeat, test-mode and input-report layout for T3CE, G7ProCE and K2_8K (see "Wire comparison" below for what is known).
- PID 0x0575, the Cyclone 2 PID on the connected device. It is not in C2's `ConfigModePids` (4125, 4138, 4179, 4107). How the vendor app accepts it is unresolved.

## Wire comparison of the pure-HID models (observed-static)

No driver shares a base class with the C2 driver; each has its own `writeProfile`. The frame
builders are nevertheless byte-identical across C2, T3CE, G7ProCE and K2_8K:

| Command | Bytes | Models |
|---|---|---|
| Write chunk | `0f 03 slot off_hi off_lo len data[≤58]` | C2, T3CE, G7ProCE, K2_8K |
| Chunk size | 58 (`EveryOutPacketLength`) | C2, T3CE, G7ProCE, K2_8K |
| Firmware / current profile / RGB read | `0f 09` / `0f 0b` / `0f 0d` | C2, T3CE, G7ProCE, K2_8K |
| Vibrate | `0f 20 66 55 l r` | C2, T3CE, G7ProCE, K2_8K |
| Light playback | `0f 0d play frame` | C2, T3CE, G7ProCE, K2_8K |
| Switch profile | `0f 07 n` | C2, T3CE (not seen for G7ProCE, K2_8K) |
| Input report id | 18 (`GamepadKeyStateReportId`) | C2, T3CE, G7ProCE, K2_8K |

What differs is the data: packet lengths and codecs (`fromModel` functions).

| Model | Profile length | Light length | Confidence |
|---|---|---|---|
| C2 | 680 | 635 | implemented |
| T3CE | 1935 | 102 | observed (codecs directly used by `writeProfile`) |
| G7ProCE | 1070 | 165 | inferred binding |
| K2_8K | 1408 | 229 | inferred binding (light codec not read) |

T3Lite is different: no profile write path in its driver, an empty
`heartBeat()`, and a different input report (`16 == t[0] && 20 == t[1]`, colour 15-17,
flags 18-19, turbo byte 14 bit 4). Only the vibrate bytes match the C2.

Result (implemented): T3CE, G7ProCE and K2_8K reuse the C2 transport with their own block lengths and the V2
profile/light codecs (`hid-family-profile.md`, `hid-family-transport.md`). The SAPJoy models' wire frames were
captured by running the vendor libraries under emulation (`sapjoy-gs3.md` for T3XS and the T4N2 library's
0xFF7A path, `sapjoy-t3pro.md` for Tarantula Pro / Nova Pro, N2S extras in `sapjoy-t3xs-n2s.md`). T3Lite:
`t3lite.md`. Those models are listed and drawn but not opened on hardware yet (`crates/opensir-core/src/models.rs`).
