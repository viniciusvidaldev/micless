import AVFoundation

/// Captures the microphone and hands out fixed-size packets of raw PCM:
/// 48 kHz, mono, signed 16-bit little-endian. The receiver hardcodes the same format.
///
/// `nonisolated` because the tap callback runs on a realtime audio thread, not the main actor.
nonisolated final class AudioCapture {
    static let sampleRate: Double = 48_000
    /// 10 ms of audio = 480 samples = 960 bytes, which fits in a single UDP datagram (MTU ~1500).
    static let samplesPerPacket = 480
    static let bytesPerPacket = samplesPerPacket * MemoryLayout<Int16>.size

    /// Called on the audio thread with exactly `bytesPerPacket` bytes and the packet's index,
    /// counting from 0 at `start()`.
    var onPacket: (@Sendable (Data, UInt32) -> Void)?

    private let engine = AVAudioEngine()
    private let outputFormat = AVAudioFormat(
        commonFormat: .pcmFormatInt16,
        sampleRate: sampleRate,
        channels: 1,
        interleaved: true
    )!
    private var converter: AVAudioConverter?
    /// Converted samples waiting to fill a whole packet. Only touched on the audio thread.
    private var pending = Data()
    private var packetIndex: UInt32 = 0

    func start() throws {
        #if !os(macOS)
        let session = AVAudioSession.sharedInstance()
        try session.setCategory(.record, mode: .default)
        try session.setPreferredSampleRate(Self.sampleRate)
        try session.setActive(true)
        #endif

        let input = engine.inputNode
        // Whatever the hardware gives us (often 48 kHz float32, but not guaranteed).
        let inputFormat = input.outputFormat(forBus: 0)
        guard inputFormat.sampleRate > 0,
              let converter = AVAudioConverter(from: inputFormat, to: outputFormat) else {
            throw CaptureError.noInput
        }
        self.converter = converter
        pending.removeAll()
        packetIndex = 0

        input.installTap(onBus: 0, bufferSize: 1024, format: inputFormat) { [weak self] buffer, _ in
            self?.process(buffer)
        }
        engine.prepare()
        try engine.start()
    }

    func stop() {
        engine.inputNode.removeTap(onBus: 0)
        engine.stop()
        #if !os(macOS)
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
        #endif
    }

    private func process(_ buffer: AVAudioPCMBuffer) {
        guard let converter else { return }

        let ratio = outputFormat.sampleRate / buffer.format.sampleRate
        let capacity = AVAudioFrameCount((Double(buffer.frameLength) * ratio).rounded(.up))
        guard let converted = AVAudioPCMBuffer(pcmFormat: outputFormat, frameCapacity: capacity) else { return }

        // The converter pulls input through this block; hand it our one buffer, then say "that's all for now".
        var handedOver = false
        var error: NSError?
        converter.convert(to: converted, error: &error) { _, status in
            if handedOver {
                status.pointee = .noDataNow
                return nil
            }
            handedOver = true
            status.pointee = .haveData
            return buffer
        }
        if error != nil { return }

        let samples = converted.int16ChannelData![0]
        pending.append(Data(bytes: samples, count: Int(converted.frameLength) * MemoryLayout<Int16>.size))

        // The tap delivers buffers of whatever size it likes; slice them into fixed packets.
        while pending.count >= Self.bytesPerPacket {
            onPacket?(Data(pending.prefix(Self.bytesPerPacket)), packetIndex)
            pending.removeFirst(Self.bytesPerPacket)
            packetIndex &+= 1
        }
    }

    /// Loudness of a packet from 0 (silence, or below -60 dB) to 1 (full scale), for a level meter.
    static func level(of pcm: Data) -> Float {
        let sumOfSquares = pcm.withUnsafeBytes { raw in
            raw.bindMemory(to: Int16.self).reduce(Float(0)) { sum, sample in
                let x = Float(sample) / Float(Int16.max)
                return sum + x * x
            }
        }
        let rms = (sumOfSquares / Float(max(pcm.count / 2, 1))).squareRoot()
        let decibels = 20 * log10(max(rms, 1e-6))
        return min(max((decibels + 60) / 60, 0), 1)
    }

    enum CaptureError: LocalizedError {
        case noInput
        var errorDescription: String? { "no microphone input available" }
    }
}
