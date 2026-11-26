# Cyclone 2 (C2) UI spec: GameSir Connect 1.14.5

Clean-room reference for rebuilding the C2 screens in a Svelte UI. Facts only: routes, controls, ranges, defaults, write triggers. Labels: `observed-static` (observed from the vendor app's behaviour), `observed-hw` (seen on the device), `inferred`.

Protocol bytes are out of scope (see the protocol doc).

## 1. Routes and shell

- Route table: the vendor UI router (vue-router 3.6.5). C2 block is the `path: "/C2/profileInfo"` entry. `observed-static`
- Child routes are absolute paths, not relative. `observed-static`:

| Route | Page |
|---|---|
| `/C2/profileInfo` (parent) | `ProfileInfo` shell |
| `/` (child, relative to root) | overview / landing |
| `/C2/profileInfo/mapping` | mapping |
| `/C2/profileInfo/triggers` | triggers |
| `/C2/profileInfo/sticks` | sticks |
| `/C2/profileInfo/motors` | motors |
| `/C2/profileInfo/rgb` | RGB |
| `/C2/profileInfo/motion` | motion |
| `/C2/buttonTest` | button test (outside the shell) |

Because the child path `/` resolves to the root, the overview's `$route.path` is `/`, not `/C2/profileInfo`. `inferred` from vue-router 3 absolute child semantics.

### 1.1 Shell (`ProfileInfo`)

- Renders a `transition` (name `slide`) around `.page-container`, then a `transition` (name from data, mode from data) around `router-view`. `observed-static`
- Per-route layout table (`observed-static`), keyed by a relative `url`:

| url | marginTop | width |
|---|---|---|
| /profileInfo | 28 | 500 |
| /mapping | -14 | 500 |
| /sticks | -14 | 500 |
| /triggers | 111 | 450 |
| /motors | -1 | 500 |
| /motion | 11 | 500 |
| /rgb | -17 | 500 |

- `syncPosition` matches an entry's `url` against `$route.path` by exact equality. Since `$route.path` is the full path (`/C2/profileInfo/mapping`), no entry matches, so `showCover` stays false and the table margins/widths are never applied. `inferred` from the equality test plus the route table. Verify on device before using these numbers.
- Transition on route change: `out-in` + `fade` when the path matches an entry (regex `search`, not equality), otherwise no transition. `observed-static`. In practice this path is also never matched, so no fade. `inferred`.

### 1.2 Reset (shell)

- Reset button opens a confirm dialog (`showDialog`). Confirm emits `ResetPageSetting` on the event bus, then shows the toast `ResetToDefaultSuccessful` with the current profile index. Cancel closes the dialog and refocuses the reset button. `observed-static`
- Each page's `ResetPageSetting` listener sends IPC `resetProfile` with `{deviceId, type: ""}` only when the current path contains `/profileInfo` or `/buttonTest`. `observed-static`
- `type: ""` resets every group listed in section 1.3. `observed-static`

### 1.3 Reset semantics, C2 (main process, `observed-static`)

`resetProfile` takes `type`. `""` or missing = all groups. Named types reset one group only.

| type | Reset group | Written values (reset defaults) |
|---|---|---|
| `Mapping` | `KeyProfiles` (all keys) | `key` → `{keyName: "", keyType: "Self", turbo: 0, mapKeys: [], toggleOnPress: false}`; `macro` → `{cycle: false, cycleTime: 0, isPressToTrigger: false, steps: []}` |
| `Sticks` | `Sticks` (each stick) | `rawEnabled: false`; `advanceMap` (outputType from the stick's default output type, reverse off, cross 0, axisRatio 50, mouseDpi 50, empty wheelMapping); `curve` {type 0, curvature 100, points (25,25),(50,50),(75,75)}; `deadzone` {begin 10, end 100, antiBegin 0, antiEnd 100} |
| `Motion` | `Motion` (`Aim`, `Tilt`) | `activateBehavior: 0`; `orientation: 0`; `triggerKey: "NC"`; `activeAxis: 0`; `advanceMap` (outputType Left for Aim, Right for Tilt; same defaults as above); `curve` as above; `deadzone` as above |
| `Triggers` | `KeyProfiles.L2`, `KeyProfiles.R2` | `curve` {type 0, curvature 100, points (25,25),(50,50),(75,75)}; `deadzone` {begin 5, end 95, antiBegin 0, antiEnd 100}; `hairTrigger` {enabled false, mode 1, beginRange 1, endRange 100} |
| `Motors` | `BasicInfo.motor` | `leftMotor: 3`, `rightMotor: 3`, `ltMotor: 3`, `rtMotor: 3` |

- `""` runs Mapping, Sticks, Motion, Triggers and Motors together. RGB and lights are not reset here. `observed-static`
- After writing the profile, the handler dispatches `updateProfileProperties` and emits `reqSyncProfile`. `observed-static`
- Reset values here (e.g. Motion curvature 100, deadzone begin 10) differ from the seed values in the Motion page (section 3). The reset values are what the device gets; the page seed values are only the initial UI state. `observed-static`

### 1.4 Main window (`observed-static`)

| Option | Value |
|---|---|
| Minimum size | width 1000, height 700 |
| Size and position | reloaded from the saved bounds (`window-state.json` in userData) |
| Frame | `frame: false`, `titleBarStyle: "hidden"` |
| Transparent | false |
| Content size | `useContentSize: true` |
| Show on create | `show: false` (shown later; the show trigger is `inferred`) |
| Background | `#04051c` |
| Icon | `assets/icon.png` |

### 1.5 App shell and device list (`observed-static`)

- Language choices: `zh_hans` (简体中文), `en_US` (English), `pt` (Português), plus Polish and others (codes not captured).
- Startup locale: a user-chosen language wins. Otherwise the locale is used if it is in the list, else `en_US`.
- Language switch (`switchToLang`): sets locale, saves `setLanguage`, closes the dialog.
- Theme toggle: icon `static/icons/btns/moon.svg` when the current theme is light, `sun.svg` otherwise. Tooltip key `ThemeTips`.
- Device list: first card is C2 (mode label "CYCLONE 2", productType "C2", AssetsFolder "C2/White", cover `static/devices/C2/White/FullCover.png`, mode image `static/devices/C2/White/Mode.png`).
- Hidden cards (`show: false`: C3, N2S, G7ProCE, K2_8K, T3XS) appear only when that product is connected (`isDeviceVisible`). C2 has no `show` flag, so it is always shown.
- A card is `loaded` only when `fwVersion` is set.
- C2 page is fixed at 1000 × 720 px.
- Focus on mount: first connected device, else first focusable element.

## 2. Overview page (`/`)

Composes `Lights`, `MainInfo`, `Motion` (§3), `Trigger`, `Stick`, and `Mapping` in that order. The page is the `Overview` component.

- `Lights`: four light slots (`ConfigIndex` 0-3, buttons shown via `TextBtn`, round). Selecting a slot sets `ConfigIndex`, resets the preview frame, and saves. Copy mode copies one slot into another (`copyAnimationTo`), then saves. Save: `setProfileByProperty` `{propertyName:"Light", profileIndex:"Light"}`, the same write as §8.
- `MainInfo`: report rate values 0 = 250 Hz, 1 = 500 Hz, 2 = 1000 Hz. Tapping the row cycles to the next value and wraps. Save: `setProfileByProperty` `{propertyName:"BasicInfo", propertyValue:{extend:{reportRate}}}`. Not in the reset list (§1.3).
- `Trigger` card: save is `setProfileByProperty` `{propertyName:"Triggers"}`, same payload as §5 (`observed-static`).
- `Stick` card: save is `setProfileByProperty` `{propertyName:"Sticks"}`, same payload as §6 (`observed-static`).
- `Mapping` card: writes no keys. Its Shift button sends IPC `changeProfileIndex` `{profileIndex:"Shift"}` and pushes route `profileInfo/mapping` (`observed-static`). The page's `goto` pushes `/mapping`.


## 3. Motion (`Motion` component on the overview page)

Data model (seed values in the page's `data()`) `observed-static`:

- Two sub-objects: `Aim` and `Tilt`. Each has:
  - `deadzone`: `beginDeadzone` 0, `endDeadzone` 100, `beginAntiDeadzone` 0, `endAntiDeadzone` 100
  - `curve`: `type` 0, `curvature` 50, 3 points (30,30), (50,50), (70,70)
  - `mappingType`: 0 Aim, 1 Tilt
  - `orientation`: 2 Combine default; 0 Yaw; 1 Roll
  - `triggerKey`: "CIR" default
  - `activateBehavior`: 1 default
  - `advanceMap`: `outputType` 3 default; `reverseHorizontal`, `reverseVertical`; `cross` 0; `axisRatio` 50; `mouseDpi` 50; `wheelMapping` {up, under, left, right}

Enumerations:

- `activateBehavior`: 0 Off, 1 ByButton, 2 ByToggle, 3 Always
- `outputType`: 1 LeftStick, 2 RightStick, 3 Wheel, 4 Mouse
- `mappingType`: 0 Aim, 1 Tilt

Behaviour:

- Deadzone slider clamps: begin ≥ 0, end ≤ 100 (`sliderChanged`). `observed-static`
- Save: IPC `setProfileByProperty` with `{deviceId, propertyName: "Motion", propertyValue}`. `observed-static`
- Reset: IPC `resetProfile` with `{deviceId, type: "Motion"}` (section 1.3). `observed-static`
- Event bus: listens `ResetPageSetting`; emits `RequireSync`. `observed-static`

The seed values (curvature 50, points 30/50/70, default `triggerKey` "CIR", `activateBehavior` 1) differ from the reset values (section 1.3). Confirm which one the device shows after a reset before picking one for the Svelte seed.

## 4. Mapping (`Mappings` > `Controller`; key-mapping dialog)

Structure (`observed-static`):

- Route `/C2/profileInfo/mapping`. `Mappings` is the page wrapper: `transition name="slide"` around `.subpage-container`, which renders `Controller` with `Enabled: true`.
- `Controller` props: `Enabled` (bool), `TestMode` (bool). Child components: `Lines` (static SVG leader lines, viewBox `0 0 858 532`, stroke `#C9CACC`), `MappingKey`, `GLine`, `BaseController`, `DialogKeyMapping`.
- Canvas viewBox `-30 0 1060 720`. Images from `static/devices/<AssetsFolder>/`: `Cover.png`, `L4.png`, `R4.png`.
- When the current profile index is `"Shift"`, the page shows the `ShiftProfile` label.

Keys (`observed-static`, the page's key list; `direction` is the leader direction):

| Key | Direction | Key | Direction |
|---|---|---|---|
| Sel | up | R2 | right |
| Camera | up | R1 | right |
| Sta | up | TRI | right |
| L2 | left | CIR | right |
| L1 | left | CRO | right |
| L3 | left | SQU | right |
| Up | left | R3 | right |
| Left | left | FL1 | under |
| Under | left | FR1 | under |
| Right | left | | |

19 keys. Positions: Sel (405,138), Camera (500,138), Sta (595,138), L2 (110,138), L1 (110,215), L3 (110,285) in SVG units. The other positions are in the vendor page's key list; not yet tabulated here.

Visibility and data (`observed-static`):

- `isShowKeys(k)` = `!Enabled || canMappingKeys` contains `k`. `canMappingKeys` = `getCurrentDevice.CanMappingKeys`.
- `specialMapping` = `getCurrentDevice.SpecialMapping`. A key with a special entry uses that list instead of the device `MappingList`.
- In the Shift profile, `"Shift"` is removed from the mapping list.
- Data: `originalKey` (highlighted key), `mappingList`, `lastChoosedBtn` (refocus target).

Interaction (`observed-static`):

- `openMappingDialog(k)` acts only when `Enabled`. It sets `originalKey = k` and opens `DialogKeyMapping`.
- `mappingTheKey(t)` (single-button pick): choosing the button already mapped, or the key's own name, clears `mapKeys`. Otherwise `mapKeys = [t]`. Then `saveToProfile()`, then `cancel()`.
- `cancel()` clears the highlight and refocuses `lastChoosedBtn`.
- Gamepad "Cancel" closes the dialog only when it is shown and not `TestMode`. Presses are ignored for 100 ms after the module loads (timestamp taken at load, `inferred` quirk).

Write path (`observed-static`, key-mapping dialog):

- Every edit saves at once. `saveToProfile()` sends IPC `setKeyProfile` with `{deviceId, keyName, model}`, where `model` is a JSON copy of the dialog model.
- Model fields: `key.mapKeys`, `key.toggleOnPress`, `key.turbo`, `key.keyName`, `key.keyType`, `macro`, `sensitivity`, `triggerMode`.
- Save rules: if the macro has steps, `keyType = "Macro"`, `keyName = "Macro"`, `mapKeys = []`. If `mapKeys` has more than one entry, `keyType = "More"` (the full rule was not captured; verify in the dialog's `saveToProfile`).
- Multi-button: toggling a button off removes it. Toggling on pushes it and trims the list to `maxMappingButtonCount`.
- The advanced panel shows when `keyType` is `"Mapped"` or `"Self"`, or when the `StickDirectionMapping` feature applies to a stick-direction key.
- Advanced-panel labels (i18n keys): `MacroTriggerMode`, `MacroTriggerMode_Loop`, `MapToMultipleButtons`, `ToggleOnPress`.
- Reset: `ResetPageSetting` sends `resetProfile` with `type: "Mapping"` (section 1.3).

Quirks (`inferred`):

- The `Enabled` watcher is keyed by the literal string `"this.Enabled"`, so it never fires. Its body refers to an undefined local variable. Intended behaviour: focus the `L2` key when `Enabled` and not `TestMode`.

## 5. Triggers (`Triggers`)

Structure (`observed-static`):

- Route `/C2/profileInfo/triggers`. Page name "Triggers". Two sides, `Left` (L2) and `Right` (R2), chosen through the `side` computed value. Labels: `LeftTrigger`, `RightTrigger`.
- `syncModel()` copies `KeyProfiles.L2` into `model.Left` and `KeyProfiles.R2` into `model.Right`, deleting each side's `key` field first.
- Helpers: `getDeadzoneModel()`, `getCurveModel()`, `getHairTriggerModel()` all read `model[side]`.

Per-side fields and seed defaults (`observed-static`, the page's seed values):

| Field | Seed | Meaning |
|---|---|---|
| `deadzone.beginDeadzone` / `endDeadzone` | 10 / 100 | Stick-style deadzone, 0 to 100 |
| `deadzone.beginAntiDeadzone` / `endAntiDeadzone` | 10 / 100 | Anti-deadzone, 0 to 100 |
| `curve.type` | 0 | 0 Linear, 1 Concave/Convex, 2 S-curve, 3 Custom (`RespondCurve*Tips`) |
| `curve.curvature` | 100 | Curve shape value (`CurveAdjustment`) |
| `curve.points` | first entry x 25, y 25 | Custom curve points; the count is not checked here |
| `hairTrigger.enabled` | false | Hair-trigger on or off |
| `hairTrigger.mode` | 1 | 1 Rapid, 2 Fix (`setTriggerMode` mapping below) |
| `hairTrigger.beginRange` / `endRange` | 0 / 100 | Hair-trigger range, 0 to 100 |

A boolean reverse flag per side is toggled by `setReverse`. Its field name is not resolved here.

Behaviour (`observed-static`):

- Slider clamps in `sliderChanged`: begin ≥ 0, end ≤ 100. For deadzone, begin stays below curve point 0 (`x` for Deadzone, `y` for AntiDeadzone, set to point − 1). End stays above curve point 2 (+ 1).
- The `sliderChanged` branch for names without `Deadzone` writes `hairTrigger["begin"+name]` and `["end"+name]`. The stored hair-trigger fields are `beginRange`/`endRange`, so the caller name that reaches this branch is not verified (`inferred`).
- Point editor: `beginPointUpdate` and `endPointUpdate` store the deadzone and anti-deadzone values. `midPointsUpdate` copies every `x`/`y` pair into `curve.points`.
- The Custom curve editor (`DeadzoneIndicator`) is visible only when `curve.type == 3`.
- `setTriggerMode(0)` turns hair trigger off and keeps the stored `mode`. Any other value turns it on and stores that mode. The UI offers Off (0), Rapid (1), and Fix (2) (`HairTriggerMode_Off`, `_Rappid`, `_Fix`).
- `changeCurvature(t)` only updates the model. It does not save (`observed-static`, a difference from the other setters).

Write path (`observed-static`):

- `saveProfile()` sends IPC `setProfileByProperty` with `{deviceId, propertyName: "Triggers", propertyValue: {L2: model.Left, R2: model.Right}}`. Both sides go in every write.
- Reset: `resetProfile` with `type: "Triggers"` (section 1.3). The page listens for `RequireSync`.

## 6. Sticks (`Sticks`)

Structure (`observed-static`):

- Route `/C2/profileInfo/sticks`. Page name "Sticks". Two sides chosen by the tab index: `tapIndex` 1 is `Left`, any other value is `Right` (`side` computed). Default tab is 1 (Left).
- Advanced-mapping sub-page, opened by `advanceMapping()` (`subpage = true`). Its title is the side's stick label plus `StickAdvanceMapping`. Cover viewBox: Left `282 158 230 230`, Right `463 230 230 230`.
- Components: `RangeSlideBar`, `StickIndicator`, `DeadzoneIndicator`, `StickWheel`, `TabContainer`, `SubPage`, `BaseController`.
- `created` runs `syncModel()`. `mounted` subscribes to `ResetPageSetting` (sends `resetProfile` with `type: "Sticks"`) and `RequireSync` (runs `syncModel()`). A watcher on `subpage` focuses the first tab element when the sub-page closes.

Per-side model (`observed-static`, `data()`; Left and Right seeds are identical):

| Path | Seed | Written by |
|---|---|---|
| `rawEnabled` | false | `setRawStick` |
| `curve.type` | 0 | `setProperty("curve.type", 0..3)` |
| `curve.curvature` | 100 | `changeCurvature` (no save) |
| `curve.points[0..2]` | (25,25), (50,50), (75,75) | `midPointsUpdate` |
| `deadzone.beginDeadzone` / `endDeadzone` | 10 / 100 | `sliderChanged("Deadzone")` |
| `deadzone.beginAntiDeadzone` / `endAntiDeadzone` | 10 / 100 | `sliderChanged("AntiDeadzone")` |
| `advanceMap.outputType` | 0 | `changeOutputType` cycles 1→2→3→4→1 |
| `advanceMap.axisRatio` | 50 | `changeAxisRatio` |
| `advanceMap.reverseHorizontal` / `reverseVertical` | false | `setProperty("advanceMap.reverse…")` |
| `advanceMap.cross` | 50 | `changeCross` |
| `advanceMap.mouseDpi` | 50 | `changeDpi` |
| `advanceMap.wheelMapping` | outline LSB, up A, left B, right X, under Y | `updateWheelKey` |

Behaviour (`observed-static`):

- `sliderChanged(name, [begin, end])` for `Deadzone` or `AntiDeadzone`: the begin value is moved to one below curve point 0 (`x` for Deadzone, `y` for AntiDeadzone) when it reaches that point. The end value is moved to one above curve point 2 when it falls to it. Then begin is clamped to ≥ 0 and end to ≤ 100. Stores `deadzone["begin"+name]` and `["end"+name]`, then saves.
- `beginPointUpdate` and `endPointUpdate` store the deadzone and anti-deadzone pair and save. `midPointsUpdate` copies each `x`/`y` into `curve.points` and saves.
- `outputTypeTips` labels: 1 `MotionOutputLeftStickTips`, 2 `MotionOutputRightStickTips`, 3 `MotionOutputDpadTips`, 4 `MotionOutputMouseTips`. Value 0 has no tip label.
- `setReverse(name)` toggles `model[side][name]` at the top level of the side. The sub-page uses `setProperty("advanceMap.reverse…")` instead, so `setReverse` appears unused (`inferred`).
- `setProperty(path, value)` takes one dot-level path and saves.

Write path (`observed-static`):

- Every save sends IPC `setProfileByProperty` with `{deviceId, propertyName: "Sticks", propertyValue: model}`. The whole model, both sides, goes in each write.
- `syncModel()` deep-copies the store's `Sticks` profile into `model`. Arrays are replaced, not merged.
- Reset: `resetProfile` with `type: "Sticks"` (section 1.3).

## 7. Motors (`Motors`)

Structure (`observed-static`):

- Route `/C2/profileInfo/motors`. Page name "Motors". A slide transition wraps the sub-page (`subpage-container`). A `TabContainer` holds the two sliders and the grip toggle. The template also mounts `MotorIndicatorLeftSide`, `MotorIndicatorRightSide`, `MotorIndicatorEffect`, `TriggerBar` and `StickWheel`; their layout was not verified.
- `created` runs `syncModel()`. `mounted` subscribes to `ResetPageSetting` (`resetProfile` with `type: "Motors"`), `RequireSync` (`syncModel()`), and `onTriggerChange` (copies `LeftTrigger` and `RightTrigger` into the preview). It also starts a 10 ms interval that calls `vibration()` while `tryGripMotor` is on. The interval is cleared on destroy.

Model and seeds (`observed-static`, `data()`):

| Field | Seed | Control |
|---|---|---|
| `leftMotor` | 3 | Slider `LeftVibrationMotorStrengthText`, `RangeSlideBar` with max 4 |
| `rightMotor` | 3 | Slider `RightVibrationMotorStrengthText`, `RangeSlideBar` with max 4 |

The slider's minimum is not set in the template. Its lower bound is not verified (`inferred`: 0). `sliderChanged` stores the first argument as given.

Grip-motor toggle (`observed-static`):

- Label `GripMotorsText`. Tapping the row calls `revertProps`, which dispatches the store action `setTryGripMotor` with the negated value. The row's argument is unused.
- This state lives in the store, not in the profile. It is not saved to the device (`inferred`: the handler sends no IPC).

Live preview and output (`observed-static`):

- `preview` is `triggerPreview` while `tryGripMotor` is on. Otherwise it is `{leftGrip: 0, rightGrip: 0}`.
- `vibration()` sends IPC `vibration` `{deviceId, left, right, duration}` with `duration` 100 when the preview changes. It only sends while `isFocused` is true. It sends when either the current preview or the last-sent value is non-zero, so a return to 0 is sent once.
- A watcher on `isFocused` sends `vibration` `{left: 0, right: 0, duration: 0}` when focus is lost.
- Indicator and effect placement and bindings: not verified.

Write path (`observed-static`):

- `saveProfile()` sends IPC `setProfileByProperty` with `{deviceId, propertyName: "BasicInfo", propertyValue: {motor: {leftMotor, rightMotor}}}`. Both values go in every write.
- `syncModel()` copies `BasicInfo.motor` from the store profile into `model`.
- Reset: `resetProfile` with `type: "Motors"` (section 1.3).

Live writes: the slider saves on each change. The grip-motor toggle is not saved.

## 8. RGB / Lights (`Light`)

Structure (`observed-static`): route `/rgb`. Components: `LightBulbs`, `ColorBtn`, `LightBar`, `BaseController`. Four `ColorBtn` swatches sit in the controller SVG and are display-only (`disabled: true`).

Editing state (`observed-static`, `data()`):

| Field | Seed | Meaning |
|---|---|---|
| `maxKeyFrames` | 8 | Upper bound for `KeyFrameCount` |
| `preview.isPlaying` | true | Playback flag, sent via `setLightStatus` |
| `preview.selectedFrame` | 0 | Index of the frame being edited (0-based) |
| `preview.isCopying` / `copied` / `pasted` / `pasteError` | false | Copy and paste state for frames |
| `preview.Presets` | null | Last applied preset name |

The profile is `this.lmodel` (source not verified, `inferred`: the Light profile from the store). `selectedAnimation` is the animation inside `lmodel`; its fields are `Brightness`, `Speed`, `KeyFrameCount`, and `Frame0`..`Frame7`.

Colour slots (`observed-static`, `convertFrame` and `updateRGB`):

| Slot | LED name | Shown in UI |
|---|---|---|
| `RGB0` | LowerLeft | yes, index 0 |
| `RGB3` | Home | yes, index 1 |
| `RGB1` | LowerRight | yes, index 2 |
| `RGB4` | UpperRight | yes, index 3 |
| `RGB2` | UpperLeft | not shown (`observed-static`); whether the device uses it is unverified |

`updateRGB(index, color)` maps the UI index 1→3, 2→1, 3→4 onto the slot number, sets `selectedFrame["RGB"+n]` and saves only if the colour changed.

Animation controls (`observed-static`):

- Brightness: `RangeSlideBar`, max 100, initial value `selectedAnimation.Brightness`.
- Speed: `RangeSlideBar`, max 20, initial value `selectedAnimation.Speed`. The module also defines the list `[0, 1, 5, 10, 20]`; its binding in the template was not verified.
- Frames: `addFrame` increments `KeyFrameCount` only while below 8. `removeFrame` decrements only while above 1 and clamps `selectedFrame`. Both save.
- Presets: `applyPreset(name)` finds the preset in the vendor preset list by `Name`, sets `Speed`, `KeyFrameCount`, and writes `Frame0`..`Frame7` (black frame for slots past the preset's frame count). Saves once.
- Disabled controls (`observed-static`): audio-reactive switch, wake-up switch, and standby-time label are `Enabled: false`. Frame buttons are disabled while `isCopying` is set and the frame index equals `lmodel.ConfigIndex`.
- Playback: `syncPlayingState` sends IPC `setLightStatus` `{deviceId, isPlaying, frameIndex: selectedFrame+1}`. Callers were not traced (`inferred`).

Write path (`observed-static`, `saveProfile`):

- IPC `setProfileByProperty` `{deviceId, propertyName: "Light", propertyValue: lmodel, profileIndex: "Light"}`. The whole `lmodel` is sent on each write (`inferred` from the shape).
- Reset: not in the reset list (section 1.3). No reset control on this page.

## 9. Motion page (`/motion`)

Standalone page in the shell. Same `Motion` model as §3. Saved as the whole model.

| Item | Fact | Label |
|---|---|---|
| Tabs | Left tab Aim (`mappingType` 0), right tab Tilt (`mappingType` 1). Tab tips `AimTips` / `TiltTips`. | observed-static |
| Trigger key | `DialogKeySelector` bound to `triggerKey`, `onlyShowButtons` true, keys limited to `activateKeys`. | observed-static |
| Orientation (Aim only) | 0 Yaw, 1 Roll, 2 Combine. Images `orientation-yaw`, `orientation-roll`, `orientation-combine`. | observed-static |
| Output | `advanceMap.outputType` 1 LeftStick, 2 RightStick, 3 four-way panel (`StickWheel_FourWay`), 4 Mouse (DPI slider). | observed-static |
| Mouse DPI | Slider max 100, `changeDpi`, shown only for outputType 4. | observed-static |
| Cross | Slider max 100, `changeCross`. | observed-static |
| Deadzone | Begin and end sliders, min 0 max 100 (`beginDeadzone`, `beginAntiDeadzone`). | observed-static |
| Curvature | Slider max 100, `changeCurvature`. Deadzone indicator shown when curve type is 3. | observed-static |
| Axis ratio | Slider max 100, labels `AxisRatio_Horizontal` / `AxisRatio_Vertical`. | observed-static |
| Reverse rows | Row tap writes `advanceMap.reverseHorizontal` via `setProperty`. The row is enabled; the switch control inside is `Enabled: false`. Vertical key name not verified. | observed-static, inferred for vertical |

Save: `setProfileByProperty` `{deviceId, propertyName:"Motion", propertyValue: model}`. Per-change call sites of `saveProfile` not traced (`inferred`: each handler saves).
Reset: `resetProfile` `type:"Motion"` (see §1.3).

## 10. Button test (`/C2/buttonTest`)

Route outside the shell (§1). Read-only test screen: no profile write call was found on this page.

| Item | Fact | Label |
|---|---|---|
| Mount | Store `setTestMode` true, then IPC `doHeartbeat`. | observed-static |
| Destroy | Store `setTestMode` false. | observed-static |
| Main process | `doHeartbeat` handler calls focus status true. | observed-static |
| Input | Gamepad down/up events update a pressed-state map. LT and RT are excluded from the map. Events are ignored while a dialog is open. | observed-static |
| Pressed flags | M to `pressedM`, Share to `pressedBack`, Capture to `pressedCapture`, Options to `pressedStart`. | observed-static |
| Components | `DPad`, `Keys`, `Triggers`, `ControllerInspection` (emits `stateChanged`), `ButtonTest`. SVG viewBox `0 50 1000 720`. | observed-static |

## 11. Pending sections

Still to document (not checked):

1. Profile list and import/export.
2. Full i18n code list, visual tokens, fonts, layout grid, C2 image paths under `static/` (partly in §1.5).
3. Store getters used across pages (`getCurrentDevice`, `getCurrentProfile`, `getCurrentProfileIndex`, `getConnectedProductTypes`, `getDevices`, `getLanguage`, `IsTestMode`) and the `lmodel` source (§12 item 12).


## 12. Open questions

Unverified or conflicting items. Each needs a source trace or a device check (read-only only).

1. Motion: seed values (§3) differ from reset values (§1.3). Which set does the first run show? (`inferred`: reset values reach the device, seeds only the UI.)
2. Shell: the `marginTop` / `width` table (§1.1) is never applied. Dead code or missing wiring?
3. Mapping: the `Enabled` watcher uses the literal key `"this.Enabled"`, so it never fires. Body uses an undefined local variable. Likely bugs; effect on device unknown.
4. Mapping: the "More" keyType rule was cut off in an earlier read; re-read before implementing.
5. Some vendor UI modules are import-only shims; their targets were not resolved.
6. Triggers and Mapping curve-type and hair-trigger labels are i18n keys only; meaning on the device is unconfirmed.
7. Output type 3 is "Wheel" in §3 but the Sticks tip key is `MotionOutputDpadTips`. The Motion page shows a four-way panel (`StickWheel_FourWay`). Pick one name after checking the i18n strings.
8. `setReverse` (Sticks, Triggers) takes a flag name that is not resolved; appears unused in Sticks.
9. Triggers: name of the `sliderChanged` caller for non-Deadzone sliders not traced.
10. Motors: slider minimum is unverified (`inferred` 0). Grip-motor toggle is not saved; confirm no save path exists.
11. RGB: speed list `[0,1,5,10,20]` binding unverified. RGB2 (UpperLeft) is not shown in the UI; device use unverified.
12. RGB: `lmodel` source is `inferred`. Callers of `setLightStatus` not traced.
13. Motion: vertical reverse key name not verified; reverse row location approximate.
14. Button test: relation between the UI `doHeartbeat` and the main-process heartbeat timer is unverified.
15. Everything above is `observed-static` or `inferred`; nothing has been checked on the device.
