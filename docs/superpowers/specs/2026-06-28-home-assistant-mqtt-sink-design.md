# Home Assistant MQTT Sink — Design

**Status:** Approved design, pending spec review
**Date:** 2026-06-28
**Related:** closes [#24](https://github.com/ncolomer/energy-monitor/issues/24); follow-up [#30](https://github.com/ncolomer/energy-monitor/issues/30); supersedes closed PR [#27](https://github.com/ncolomer/energy-monitor/pull/27)

## 1. Goal

Add an optional **Home Assistant** sink that publishes collected metrics over **MQTT Discovery**, alongside the existing InfluxDB sink. Each sink is enabled independently by the presence of its config block — a user may enable InfluxDB only, Home Assistant only, both, or neither. The two sinks operate independently; a failure of one must not affect the other.

## 2. Background & confirmation that HA-over-MQTT is the standard

Home Assistant's **MQTT Discovery** is the official, documented way for a device to self-register entities without any manual YAML in HA. Sources consulted:

- MQTT integration / discovery: <https://www.home-assistant.io/integrations/mqtt/#mqtt-discovery>
- MQTT sensor: <https://www.home-assistant.io/integrations/sensor.mqtt/>
- Energy management / electricity grid: <https://www.home-assistant.io/docs/energy/electricity-grid/>

Key facts that shape the design:

- **Discovery topic format:** `<discovery_prefix>/<component>/[<node_id>/]<object_id>/config`, with `<discovery_prefix>` defaulting to `homeassistant`. `<object_id>` must match `[a-zA-Z0-9_-]`.
- **Discovery config messages should be published `retain=true`** so entities are restored when HA restarts/reconnects.
- HA publishes a **birth message** to `<discovery_prefix>/status` with payload `online` on startup; devices should **re-announce** discovery when they see it.
- **`device.identifiers`** (a non-empty list) is required to register a device and group its entities.
- **Availability** is expressed via `availability_topic` plus an MQTT **Last-Will** (`online`/`offline`).
- **Device classes / state classes** (from the docs table):
  | Measurement | device_class | state_class | unit |
  |---|---|---|---|
  | instantaneous power | `power` | `measurement` | `W` |
  | apparent power | `apparent_power` | `measurement` | `VA` |
  | current | `current` | `measurement` | `A` |
  | voltage | `voltage` | `measurement` | `V` |
  | power factor | `power_factor` | `measurement` | *(none)* |
  | cumulative energy index | `energy` | `total_increasing` | `Wh` or `kWh` |

## 3. Lessons taken from PR #27 (what we deliberately do differently)

1. **No networked broker in tests.** PR #27's integration tests connected to the public `test.mosquitto.org` broker — non-deterministic and impossible to pass in offline CI. We test only **pure message-construction logic** (topics + JSON payloads), no live broker.
2. **Discovery is `retain=true` and re-announced** on every (re)connection and on the HA birth message — PR #27 used `retain=false` and announced once, so entities vanished on HA restart.
3. **`device.identifiers` is set** — PR #27 omitted it, so entities did not form a proper device.
4. **Discovery prefix is independent** (`homeassistant` by default), not nested under a state prefix as in PR #27.
5. **Energy index uses `total_increasing`** (Energy-Dashboard-ready); **power factor carries no unit** (lechacal emits 0–1, so `%` was wrong).
6. **No `entity_category: diagnostic`** (it hides sensors from normal dashboards / energy).
7. **Robust event loop**: poll forever so `rumqttc` auto-reconnects; **no `.unwrap()`** in the publish/announce paths.
8. **Fewer dependencies**: no `tokio-stream`/`async-stream`/`futures`/`regex` (those existed only for the broker test).

## 4. Architecture

The existing `DataLoggerActor` already owns the optional InfluxDB sink and emits `InfluxDbConnected/Disconnected` to the HMI. It is extended to also own an optional `HassMqttClient`.

```
RpictActor ─┐                         ┌─> InfluxDBClient (HTTP)   [if `influxdb` configured]
            ├─> DataLoggerActor ──────┤
LinkyActor ─┘        │                └─> HassMqttClient (MQTT)   [if `hassmqtt` configured]
                     └─> DataLoggerMessage::{Influx,HassMqtt}{Connected,Disconnected} ─> HmiActor ─> OLED
```

**Duplication cleanup (in scope):** today the publish + connection-status block is copy-pasted per source for InfluxDB; PR #27 made it 4× (2 sinks × 2 sources). Because both `RpictFrame` and `LinkyFrame` implement both sink traits, we collapse it into **one generic** `publish_to_sinks(&frame)` helper, and move the `*_connected` flags from `run()` locals to actor fields. Adding the second sink thus *reduces* duplication.

## 5. Components

### 5.1 `src/service/hassmqtt.rs` (new)

Mirrors `influxdb.rs`: **pure logic** (unit-tested) separated from **I/O** (thin wrapper).

**Constants:** `DEVICE_ID = "energy-monitor"`, `DEFAULT_DISCOVERY_PREFIX = "homeassistant"`, `SUPPORT_URL = "https://github.com/ncolomer/energy-monitor"`.

**Topics:**
- Availability: `energy-monitor/availability`
- State (rpict): `energy-monitor/rpict`
- State (linky): `energy-monitor/linky`
- Discovery config: `<discovery_prefix>/sensor/energy-monitor/<source>_<field>/config`
  (e.g. `homeassistant/sensor/energy-monitor/rpict_l1_real_power/config`)
- `unique_id`: `energy-monitor_<source>_<field>` (e.g. `energy-monitor_rpict_l1_real_power`)

**`Message { topic: String, payload: String }`** — the unit of publication.

**`Sensor`** descriptor (uses `&'static str`):
```rust
pub struct Sensor {
    pub name: &'static str,           // JSON field, e.g. "l1_real_power"
    pub source: &'static str,         // "rpict" | "linky"
    pub device_class: &'static str,   // "power", "energy", ...
    pub state_class: &'static str,    // "measurement" | "total_increasing"
    pub unit: Option<&'static str>,   // None => omit unit_of_measurement
}
```

**Discovery payload** (`Sensor::to_discovery_message(&self) -> Message`), JSON keys:
`name`, `unique_id`, `state_topic`, `value_template` (`{{ value_json.<field> }}`), `availability_topic`,
`device_class`, `state_class`, `unit_of_measurement` *(only when `unit` is `Some`)*,
`device` `{ identifiers: ["energy-monitor"], name, manufacturer: "DIY", model: "energy-monitor", sw_version }`,
`origin` `{ name: "energy-monitor", sw_version, support_url }`. `sw_version` = `env!("CARGO_PKG_VERSION")`.

**Sensor lists:** `RpictFrame::sensors() -> Vec<Sensor>` (15: L1/L2/L3 × {real_power W/power, apparent_power VA/apparent_power, irms A/current, vrms V/voltage, power_factor none/power_factor}), `LinkyFrame::sensors() -> Vec<Sensor>` (2: `hchc`, `hchp`, both `energy`/`total_increasing`/`Wh`).

**State payload** (`Publishable::to_state_message(&self) -> Message`): the frame serialized to JSON via `serde`. Requires `#[derive(Serialize)]` on `RpictFrame`/`LinkyFrame` and the `chrono` `serde` feature. Extra fields (`node_id`, `timestamp`) are harmless to HA.

**`HassMqttClient`** (I/O):
```rust
pub struct HassMqttClient {
    client: rumqttc::AsyncClient,
    connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
```
- `new(&settings::HassMqtt) -> Result<Self, Box<dyn Error>>`:
  - `MqttOptions::new("energy-monitor", host, port)`; set credentials when `username` (and `password`) present; `keep_alive = 60s`.
  - **Last-Will** on `energy-monitor/availability` = `offline`, QoS `AtLeastOnce`, `retain=true`.
  - Precompute the `Vec<Message>` of all discovery messages + the `online` availability message + the resolved `discovery_prefix`/status topic.
  - Spawn the **event-loop task** that polls forever:
    - on `Packet::ConnAck` → set `connected=true`, subscribe to `<discovery_prefix>/status`, publish **all discovery messages** (`retain=true`) and availability `online` (`retain=true`);
    - on `Packet::Publish` to the status topic with payload `online` → re-publish discovery + `online`;
    - on `Err(_)` → set `connected=false`, log, `sleep` short backoff, continue (rumqttc reconnects).
- `publish(&impl Publishable) -> Result<(), HassMqttClientError>`: publish state `Message` (QoS `AtLeastOnce`, `retain=false`); log + return `Err` on failure (never panics).
- `is_connected(&self) -> bool`: reads the `AtomicBool`.

### 5.2 `src/actor/datalogger.rs` (modify)

- `DataLoggerMessage` gains `HassMqttConnected`, `HassMqttDisconnected`.
- `DataLoggerActor` gains `hassmqtt: Option<HassMqttClient>` plus `influxdb_connected: bool` and `hassmqtt_connected: bool` fields.
- New generic helper:
  ```rust
  async fn publish_to_sinks<P>(&mut self, frame: &P)
  where P: crate::service::influxdb::InfluxDbSerialize + crate::service::hassmqtt::Publishable
  ```
  For InfluxDB: status = `publish(...).await.is_ok()`. For HA: attempt `publish`, status = `is_connected()`. Each emits a transition message only when the tracked flag changes (preserving today's semantics).
- `run()`'s rpict/linky arms call `self.publish_to_sinks(&frame).await`.
- `create(influxdb_settings, hassmqtt_settings, rpict, linky)`: build each client with `.map(..).transpose()?` (no `.unwrap()`).

### 5.3 `src/settings.rs` + yml (modify)

```rust
pub struct HassMqtt {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub discovery_prefix: Option<String>, // default "homeassistant" applied in client
}
// Settings gains: pub hassmqtt: Option<HassMqtt>,
```
- `settings.default.yml`: add `hassmqtt: null` (disabled by default; InfluxDB block unchanged).
- `settings.example.yml`: add a populated `hassmqtt` example (host `localhost`, port `1883`, commented username/password/discovery_prefix).

### 5.4 Display (modify) — house icon + dynamic icon row

- `src/display/icons.rs`: add `HASS_OFF` / `HASS_ON` (8×8). `HASS_ON` = per-row bitwise NOT of `HASS_OFF`, matching the existing pairs. `HASS_OFF` (filled house silhouette, roof + body + door):
  ```
  0b0001_1000, 0b0011_1100, 0b0111_1110, 0b0111_1110,
  0b0111_1110, 0b0110_0110, 0b0110_0110, 0b0000_0000,
  ```
- `src/display/pages.rs` `StartupPage`:
  - sink fields become `influxdb: Option<bool>` and `hassmqtt: Option<bool>` (`None` = not configured → not drawn; `Some(connected)` = drawn ON/OFF). `is_rpict_connected`/`is_linky_connected` stay `bool` (always drawn).
  - `new(version, influxdb_enabled, hassmqtt_enabled)` sets `enabled.then_some(false)`.
  - Setters keep `None` when disabled: `self.influxdb = self.influxdb.map(|_| connected)`.
  - `draw()` builds an **ordered icon list** (rpict, linky, then influxdb/hassmqtt if present) and lays them at `Point::new(20 + 10*i, 20)` — uniform 10px pitch, no gaps. (4 icons end at x=58, clear of the right-aligned version text.)
- `src/actor/hmi.rs`: `handle_datalogger` handles the two new messages (`hassmqtt_status(true/false)` + redraw). `HmiActor::create` takes `influxdb_enabled`/`hassmqtt_enabled` bools and forwards them to `StartupPage::new`.

### 5.5 `src/main.rs` (modify)

- `DataLoggerActor::create(&settings.influxdb, &settings.hassmqtt, &rpict, &linky)?`
- `HmiActor::create(&settings.hmi, &rpict, &linky, &datalogger, settings.influxdb.is_some(), settings.hassmqtt.is_some())?`

### 5.6 `Cargo.toml` (modify)

- `chrono = { version = "0.4.23", features = ["serde"] }`
- `rumqttc = { version = "0.24", default-features = false }` (TCP only — no TLS/ring → clean armv6 cross-compile). **Exact minor version validated with `cross check` during implementation.**
- No new `[dev-dependencies]`.

### 5.7 Documentation (modify)

- `README.md`: mention Home Assistant in the intro; add a "Home Assistant (MQTT)" subsection; add config-table rows (`hassmqtt.host`/`port`/`username`/`password`/`discovery_prefix` + `APP__HASSMQTT__*` env vars); add the new icon to the Startup-screen icon list with rendered `docs/images/icon-hass-on.png` / `icon-hass-off.png`.

## 6. Data flow

1. `RpictActor`/`LinkyActor` emit `NewFrame(frame)`.
2. `DataLoggerActor::publish_to_sinks(&frame)` publishes to each configured sink and emits status transitions.
3. For HA: state JSON → `energy-monitor/<source>`; HA maps fields to entities via the retained discovery configs.
4. On (re)connect / HA birth, the client re-announces discovery + availability `online`; on disconnect, the LWT sets `offline`.
5. `HmiActor` reflects per-sink connection status on the OLED startup page.

## 7. Error handling

- Construction (`HassMqttClient::new`) returns `Result`; a config error aborts startup with a clear message (same contract as InfluxDB).
- Runtime publish failures are logged and surfaced as `is_connected() == false` (→ OLED `OFF`); they never panic and never affect the InfluxDB sink.
- The event loop never exits on a connection error — it backs off and lets `rumqttc` reconnect.

## 8. Testing strategy

**Unit tests only — deterministic, no broker, runnable under `cross test`:**
- `hassmqtt.rs`: exact discovery `Message` for a power sensor (rpict) and an energy sensor (linky); a power-factor sensor **omits** `unit_of_measurement`; `RpictFrame::sensors().len() == 15` and `LinkyFrame::sensors().len() == 2`; exact `to_state_message` topic + JSON for both frames.
- `settings.rs`: extend `test_load_example_settings` to assert `hassmqtt.is_some()`.
- `pages.rs`: update the two `StartupPage` tests for the `Option<bool>` fields; assert `new(.., true, false)` yields `influxdb = Some(false)`, `hassmqtt = None`.

**Verification before pushing (exactly what CI runs):** `cross clippy`, `cross check`, `cross test` — all green. (A full on-device run isn't possible off the Raspberry Pi because of `rppal`/serial; this is the same verification bar as the rest of the repo.)

## 9. Out of scope

- TLS transport (chosen: plaintext + optional user/pass).
- Making rpict/linky **sources** optional → tracked in [#30](https://github.com/ncolomer/energy-monitor/issues/30). The dynamic icon row already supports it.
