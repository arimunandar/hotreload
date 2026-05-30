import SwiftUI
import HotReloadKit

@main
struct DemoApp: App {
    init() {
        // Configure hot reload — starts the TCP injection server on port 8899
        HotReload.configure(port: 8899)
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .enableInjection() // Required: enables view re-evaluation on injection
        }
    }
}
