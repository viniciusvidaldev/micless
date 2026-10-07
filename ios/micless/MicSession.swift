import AVFoundation
import Foundation
import Network
import Observation

enum SessionState: Equatable {
    case disconnected
    case connecting
    case handshaking
    case streaming
    case failed(String)
}

/// One session with the receiver: TCP for control, UDP for the audio stream.
/// See `Wire` for the byte layout.
@Observable
final class MicSession {
    private(set) var state: SessionState = .disconnected
    private(set) var log: [String] = []
    private(set) var packetsSent = 0
    /// Mic loudness, 0...1, refreshed ~20 times a second while streaming.
    private(set) var level: Float = 0
    private(set) var streamingSince: Date?

    private var tcp: NWConnection?
    private var udp: NWConnection?
    private var audio: AudioCapture?
    private var lastSendError: String?

    func connect(host: String) {
        teardown()

        let remoteHost = NWEndpoint.Host(host)
        let remotePort = NWEndpoint.Port(integerLiteral: Wire.port)

        state = .connecting
        append("connecting to \(host):\(Wire.port)…")

        // Same port number for both: TCP and UDP ports are separate namespaces.
        let udp = NWConnection(host: remoteHost, port: remotePort, using: .udp)
        let tcp = NWConnection(host: remoteHost, port: remotePort, using: .tcp)
        self.udp = udp
        self.tcp = tcp

        // Every callback checks it still belongs to the *current* connection,
        // so a late event from a previous session can't clobber this one.
        tcp.stateUpdateHandler = { [weak self, weak tcp] new in
            guard let self, let tcp, tcp === self.tcp else { return }
            self.tcpStateChanged(new)
        }
        udp.stateUpdateHandler = { [weak self, weak udp] new in
            guard let self, let udp, udp === self.udp else { return }
            self.udpStateChanged(new)
        }

        // Callbacks are delivered on the main queue, so they can touch UI state directly.
        tcp.start(queue: .main)
        udp.start(queue: .main)
    }

    func disconnect() {
        let wasActive = state != .disconnected
        teardown()
        state = .disconnected
        if wasActive { append("disconnected") }
    }

    // MARK: - control (TCP)

    private func tcpStateChanged(_ new: NWConnection.State) {
        switch new {
        case .ready:
            append("tcp ready")
            awaitToken()
        case .waiting(let error):
            fail("tcp waiting: \(error.localizedDescription)")
        case .failed(let error):
            fail("tcp failed: \(error.localizedDescription)")
        default:
            break
        }
    }

    private func awaitToken() {
        guard let tcp else { return }
        state = .handshaking
        append("waiting for session token…")

        tcp.receive(minimumIncompleteLength: Wire.tokenLength, maximumLength: Wire.tokenLength) {
            [weak self, weak tcp] data, _, _, error in
            guard let self, let tcp, tcp === self.tcp else { return }
            if let error {
                self.fail("tcp recv: \(error.localizedDescription)")
                return
            }
            guard let data, let token = Wire.parseToken(data) else {
                self.fail("receiver closed before sending a token")
                return
            }
            self.append("got session token")
            self.startStreaming(token: token)
        }
    }

    /// The receiver never sends anything after the token, so any read completing means the session is over:
    /// it hung up, or another phone took over.
    private func watchForClose() {
        tcp?.receive(minimumIncompleteLength: 1, maximumLength: 1) { [weak self, weak tcp] _, _, _, error in
            guard let self, let tcp, tcp === self.tcp else { return }
            self.fail(error.map { "tcp: \($0.localizedDescription)" } ?? "receiver ended the session")
        }
    }

    // MARK: - audio (UDP)

    private func udpStateChanged(_ new: NWConnection.State) {
        switch new {
        case .ready:
            append("udp ready")
        case .waiting(let error):
            fail("udp waiting: \(error.localizedDescription)")
        case .failed(let error):
            fail("udp failed: \(error.localizedDescription)")
        default:
            break
        }
    }

    private func startStreaming(token: Wire.Token) {
        Task { [weak self] in
            guard await AVAudioApplication.requestRecordPermission() else {
                self?.fail("microphone permission denied")
                return
            }
            // Bail if the user hit disconnect while the permission prompt was up.
            guard let self, state == .handshaking, let udp else { return }

            let audio = AudioCapture()
            audio.onPacket = { [weak self] pcm, seq in
                // Runs on the audio thread. NWConnection.send is safe to call from any thread.
                let datagram = Wire.audioDatagram(token: token, seq: seq, pcm: pcm)
                udp.send(content: datagram, completion: .contentProcessed { [weak self] error in
                    guard let error else { return }
                    Task { @MainActor [weak self] in self?.sendFailed(error) }
                })

                // The UI only needs ~20 updates a second, not one per 10 ms packet.
                if seq % 5 == 4 {
                    let level = AudioCapture.level(of: pcm)
                    Task { @MainActor [weak self] in self?.progress(sent: Int(seq) + 1, level: level) }
                }
            }
            do {
                try audio.start()
            } catch {
                fail("audio: \(error.localizedDescription)")
                return
            }
            self.audio = audio
            state = .streaming
            streamingSince = .now
            append("streaming 48 kHz mono s16le")
            watchForClose()
        }
    }

    private func progress(sent: Int, level: Float) {
        guard state == .streaming else { return }
        packetsSent = sent
        self.level = level
    }

    private func sendFailed(_ error: NWError) {
        guard state == .streaming else { return }
        // Errors tend to repeat every packet; log each distinct one once.
        let message = "udp send failed: \(error.localizedDescription)"
        if message != lastSendError {
            lastSendError = message
            append(message)
        }
    }

    // MARK: - helpers

    private func teardown() {
        audio?.stop()
        audio = nil
        tcp?.cancel()
        udp?.cancel()
        tcp = nil
        udp = nil
        packetsSent = 0
        level = 0
        streamingSince = nil
        lastSendError = nil
    }

    private func fail(_ reason: String) {
        teardown()
        state = .failed(reason)
        append(reason)
    }

    private func append(_ line: String) {
        log.append(line)
        if log.count > 100 {
            log.removeFirst(log.count - 100)
        }
    }
}
