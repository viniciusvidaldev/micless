import Darwin

nonisolated enum LocalNetwork {
    /// The first three numbers of the phone's Wi-Fi IPv4 address, e.g. "192.168.1.",
    /// so the user usually only has to type the last one. `nil` when not on Wi-Fi.
    static func wifiPrefix() -> String? {
        var list: UnsafeMutablePointer<ifaddrs>?
        guard getifaddrs(&list) == 0, let first = list else { return nil }
        defer { freeifaddrs(list) }

        for entry in sequence(first: first, next: { $0.pointee.ifa_next }) {
            let interface = entry.pointee
            // en0 is Wi-Fi on iPhone.
            guard String(cString: interface.ifa_name) == "en0",
                  let address = interface.ifa_addr,
                  address.pointee.sa_family == UInt8(AF_INET) else { continue }

            let ip = address.withMemoryRebound(to: sockaddr_in.self, capacity: 1) {
                UInt32(bigEndian: $0.pointee.sin_addr.s_addr)
            }
            return "\(ip >> 24).\(ip >> 16 & 0xff).\(ip >> 8 & 0xff)."
        }
        return nil
    }

    static func isValidIPv4(_ text: String) -> Bool {
        let parts = text.split(separator: ".", omittingEmptySubsequences: false)
        return parts.count == 4 && parts.allSatisfy { part in
            // No leading zeros: "087" would be read as octal or rejected as a hostname.
            !part.isEmpty && part.count <= 3 && part.allSatisfy(\.isASCII) && UInt8(part) != nil
                && (part == "0" || !part.hasPrefix("0"))
        }
    }
}
