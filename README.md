# micless

Use your iPhone as a microphone for your Mac over Wi-Fi.

The phone captures its mic and streams raw PCM to a receiver on the Mac. The receiver writes the audio into shared memory, and a CoreAudio driver exposes it as an input device named **micless**. Any app (Zoom, QuickTime, OBS, …) can then pick it like a regular microphone.

```
  iPhone                    Mac
┌─────────┐  TCP: control  ┌──────────┐  shared memory  ┌────────┐
│   ios   │ ─────────────▶ │ receiver │ ──────────────▶ │ driver │ ──▶ apps
│   app   │ ─────────────▶ │          │  /micless.ring  │        │
└─────────┘  UDP: audio    └──────────┘                 └────────┘
```

## Components

| Dir         | What it is |
|-------------|------------|
| `ios/`      | SwiftUI app. Captures the mic, converts it to 48 kHz mono s16le and sends it in 10 ms packets. |
| `receiver/` | Rust binary. Accepts the phone's control connection, receives the audio datagrams and pushes the samples into the ring. |
| `ring/`     | Rust library. A lock-free single-producer/single-consumer ring buffer in POSIX shared memory (`/micless.ring`). |
| `driver/`   | Rust `cdylib`. An AudioServerPlugIn (HAL driver) loaded by `coreaudiod` that reads from the ring and serves it as an input device. |

All three Rust crates use only `std` and `libc`.

## Protocol

Both channels use port **47300**. The format is hardcoded on both ends and must stay in sync between `receiver/src/protocol.rs` and `ios/micless/Wire.swift`.

- **TCP (control):** the phone connects, and the receiver replies with a random 4-byte session token (`u32` LE). The session lasts until the TCP connection closes. A new phone connecting replaces the current session.
- **UDP (audio):** each datagram is `[token u32 LE][seq u32 LE][480 × i16 LE samples]`, which is 10 ms of 48 kHz mono audio. The receiver drops datagrams whose token doesn't match the current session.

## Running

Requirements: macOS, a Rust toolchain (edition 2024), Xcode, and an iPhone on the same Wi-Fi network as the Mac.

1. **Install the driver** (needs `sudo`; it restarts `coreaudiod`):

   ```sh
   cd driver && ./install.sh
   ```

   This builds the plug-in, wraps it in `Micless.driver`, ad-hoc signs it and copies it to `/Library/Audio/Plug-Ins/HAL/`.

2. **Start the receiver:**

   ```sh
   cd receiver && cargo run --release
   ```

   It prints the address to enter in the phone app.

3. **Run the iOS app** from `ios/micless.xcodeproj` on your phone, enter the Mac's IP and tap the mic button. Allow the microphone and local network permissions when asked.

4. Choose **micless** as the input device in any app on the Mac.

### Notes

- The driver attaches to the ring when the device starts I/O. If the receiver wasn't running when `coreaudiod` loaded the driver, starting it and then reselecting the input is enough.
- Driver logs go to the system log: `log stream --predicate 'eventMessage CONTAINS "micless"'`.
- To uninstall: `sudo rm -rf /Library/Audio/Plug-Ins/HAL/Micless.driver && sudo killall coreaudiod`.
