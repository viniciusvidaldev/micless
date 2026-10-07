import SwiftUI

struct ContentView: View {
    @State private var session = MicSession()
    /// Last address used; empty on first launch, then pre-filled with the Wi-Fi prefix.
    @AppStorage("receiver.host") private var host = ""
    @FocusState private var addressFocused: Bool

    private var isActive: Bool {
        switch session.state {
        case .connecting, .handshaking, .streaming: true
        case .disconnected, .failed: false
        }
    }

    private var hasValidAddress: Bool {
        LocalNetwork.isValidIPv4(host)
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 28) {
                    MicButton(state: session.state, level: session.level) {
                        isActive ? session.disconnect() : connect()
                    }
                    .disabled(!isActive && !hasValidAddress)
                    .opacity(!isActive && !hasValidAddress ? 0.45 : 1)
                    .padding(.top, 16)

                    status

                    if isActive {
                        if session.state == .streaming {
                            stats
                                .transition(.opacity.combined(with: .move(edge: .top)))
                        }
                    } else {
                        addressCard
                            .transition(.opacity.combined(with: .move(edge: .bottom)))
                    }

                    logCard
                }
                .padding(.horizontal, 20)
                .padding(.bottom, 32)
                .animation(.smooth, value: session.state)
                .animation(.smooth, value: hasValidAddress)
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle("micless")
            .scrollDismissesKeyboard(.interactively)
            .sensoryFeedback(trigger: session.state) { _, new in
                switch new {
                case .streaming: .success
                case .failed: .error
                default: nil
                }
            }
            .onAppear(perform: prefillAddress)
        }
    }

    // MARK: - sections

    private var status: some View {
        VStack(spacing: 6) {
            Text(statusTitle)
                .font(.title2.weight(.semibold))
            Text(statusDetail)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .lineLimit(3)
        }
        .frame(maxWidth: .infinity)
        .contentTransition(.opacity)
    }

    private var addressCard: some View {
        Card(title: "computer", systemImage: "desktopcomputer") {
            HStack(spacing: 12) {
                TextField("192.168.1.10", text: $host)
                    .font(.title2.monospacedDigit().weight(.medium))
                    .keyboardType(.numbersAndPunctuation)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .submitLabel(.go)
                    .focused($addressFocused)
                    .onSubmit(connect)
                    .onChange(of: host) { _, new in
                        // Only digits and dots belong in an IPv4 address.
                        let cleaned = new.filter { $0.isNumber || $0 == "." }
                        if cleaned != new { host = cleaned }
                    }

                Image(systemName: hasValidAddress ? "checkmark.circle.fill" : "circle.dashed")
                    .font(.title2)
                    .foregroundStyle(hasValidAddress ? Color.green : Color.secondary)
                    .contentTransition(.symbolEffect(.replace))
            }

            Text("Same Wi-Fi as this phone · port \(String(Wire.port))")
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
    }

    private var stats: some View {
        HStack(spacing: 12) {
            StatTile(title: "time") {
                TimelineView(.periodic(from: .now, by: 1)) { context in
                    Text(elapsed(at: context.date))
                }
            }
            StatTile(title: "packets") {
                Text(session.packetsSent, format: .number)
            }
            StatTile(title: "level") {
                LevelBar(level: session.level)
            }
        }
    }

    private var logCard: some View {
        Card(title: "log", systemImage: "text.alignleft") {
            DisclosureGroup(session.log.last ?? "nothing yet") {
                Text(session.log.joined(separator: "\n"))
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
                    .padding(.top, 8)
            }
            .font(.footnote)
            .foregroundStyle(.secondary)
            .tint(.secondary)
        }
    }

    // MARK: - text

    private var statusTitle: String {
        switch session.state {
        case .disconnected: hasValidAddress ? "Tap to stream" : "Where's your computer?"
        case .connecting: "Connecting…"
        case .handshaking: "Starting session…"
        case .streaming: "Live"
        case .failed: "Disconnected"
        }
    }

    private var statusDetail: String {
        switch session.state {
        case .disconnected:
            hasValidAddress ? "Your mic will play on \(host)" : "Enter the IP of the computer running the receiver"
        case .connecting, .handshaking: host
        case .streaming: "Streaming to \(host) · 48 kHz mono"
        case .failed(let reason): reason
        }
    }

    // MARK: - actions

    private func prefillAddress() {
        if host.isEmpty, let prefix = LocalNetwork.wifiPrefix() {
            host = prefix
        }
        if !hasValidAddress {
            addressFocused = true
        }
    }

    private func connect() {
        guard hasValidAddress else { return }
        addressFocused = false
        session.connect(host: host)
    }

    private func elapsed(at date: Date) -> String {
        guard let since = session.streamingSince else { return "0:00" }
        let seconds = max(Int(date.timeIntervalSince(since)), 0)
        return String(format: "%d:%02d", seconds / 60, seconds % 60)
    }
}

// MARK: - components

/// The big round button: tap to connect / disconnect. Rings pulse with the mic level while live.
private struct MicButton: View {
    let state: SessionState
    let level: Float
    let action: () -> Void

    private let size: CGFloat = 168

    private var tint: Color {
        switch state {
        case .disconnected: .accentColor
        case .connecting, .handshaking: .orange
        case .streaming: .green
        case .failed: .red
        }
    }

    private var symbol: String {
        switch state {
        case .streaming: "mic.fill"
        case .failed: "mic.slash.fill"
        default: "mic"
        }
    }

    var body: some View {
        Button(action: action) {
            ZStack {
                if state == .streaming {
                    ForEach(0..<2) { ring in
                        Circle()
                            .fill(tint.opacity(ring == 0 ? 0.18 : 0.10))
                            .scaleEffect(1 + CGFloat(level) * (ring == 0 ? 0.25 : 0.45))
                    }
                }

                Circle()
                    .fill(tint.gradient)
                    .shadow(color: tint.opacity(0.35), radius: 16, y: 8)

                if state == .connecting || state == .handshaking {
                    ProgressView()
                        .controlSize(.large)
                        .tint(.white)
                } else {
                    Image(systemName: symbol)
                        .font(.system(size: 56, weight: .semibold))
                        .foregroundStyle(.white)
                        .contentTransition(.symbolEffect(.replace))
                }
            }
            .frame(width: size, height: size)
            .animation(.easeOut(duration: 0.12), value: level)
        }
        .buttonStyle(PressScaleStyle())
        .accessibilityLabel(state == .streaming ? "Stop streaming" : "Start streaming")
    }
}

private struct PressScaleStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed ? 0.94 : 1)
            .animation(.spring(duration: 0.2), value: configuration.isPressed)
    }
}

private struct Card<Content: View>: View {
    let title: String
    let systemImage: String
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label(title, systemImage: systemImage)
                .font(.caption.weight(.semibold))
                .textCase(.uppercase)
                .foregroundStyle(.secondary)
            content
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground), in: .rect(cornerRadius: 16))
    }
}

private struct StatTile<Value: View>: View {
    let title: String
    @ViewBuilder let value: Value

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.caption.weight(.semibold))
                .textCase(.uppercase)
                .foregroundStyle(.secondary)
            value
                .font(.title3.monospacedDigit().weight(.semibold))
                .frame(height: 24)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground), in: .rect(cornerRadius: 14))
    }
}

private struct LevelBar: View {
    let level: Float

    var body: some View {
        GeometryReader { geo in
            ZStack(alignment: .leading) {
                Capsule().fill(.quaternary)
                Capsule()
                    .fill(.green.gradient)
                    .frame(width: max(geo.size.width * CGFloat(level), 6))
            }
        }
        .frame(height: 8)
        .animation(.easeOut(duration: 0.12), value: level)
    }
}

#Preview {
    ContentView()
}
