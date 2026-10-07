import Foundation

/// The wire format shared with `receiver/src/protocol.rs`. Keep the two in sync.
///
/// TCP: phone connects → receiver replies `[token u32 le]` → the session lasts until TCP closes.
/// UDP: every audio datagram is `[token u32 le][seq u32 le][samples i16 le]`.
nonisolated enum Wire {
    typealias Token = UInt32

    /// Both TCP (control) and UDP (audio); fixed on both ends.
    static let port: UInt16 = 47300

    static let tokenLength = MemoryLayout<Token>.size

    static func parseToken(_ reply: Data) -> Token? {
        guard reply.count == tokenLength else { return nil }
        return reply.withUnsafeBytes { Token(littleEndian: $0.loadUnaligned(as: Token.self)) }
    }

    static func audioDatagram(token: Token, seq: UInt32, pcm: Data) -> Data {
        var datagram = Data(capacity: 2 * MemoryLayout<UInt32>.size + pcm.count)
        withUnsafeBytes(of: token.littleEndian) { datagram.append(contentsOf: $0) }
        withUnsafeBytes(of: seq.littleEndian) { datagram.append(contentsOf: $0) }
        datagram.append(pcm)
        return datagram
    }
}
