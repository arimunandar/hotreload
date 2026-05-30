import Foundation
import SwiftUI

enum TodoPriority: String, CaseIterable, Codable {
    case low, medium, high
    
    var color: Color {
        switch self {
        case .low: return .green
        case .medium: return .orange
        case .high: return .red
        }
    }
    
    var icon: String {
        switch self {
        case .low: return "arrow.down.circle"
        case .medium: return "equal.circle"
        case .high: return "arrow.up.circle.fill"
        }
    }
    
    var label: String { rawValue.capitalized }
}

enum TodoFilter: String, CaseIterable {
    case all, active, completed
    var label: String { rawValue.capitalized }
}

struct TodoItem: Identifiable {
    let id = UUID()
    var title: String
    var notes: String
    var isCompleted: Bool
    var priority: TodoPriority
    var createdAt: Date
    var category: String
    
    static let categories = ["Work", "Personal", "Shopping", "Health", "Learning"]
    
    static let samples: [TodoItem] = [
        TodoItem(title: "Build hot reload CLI", notes: "Rust + Swift integration", isCompleted: true, priority: .high, createdAt: Date(), category: "Work"),
        TodoItem(title: "Write unit tests", notes: "Cover compiler and injector", isCompleted: false, priority: .medium, createdAt: Date(), category: "Work"),
        TodoItem(title: "Buy groceries", notes: "Milk, eggs, bread", isCompleted: false, priority: .low, createdAt: Date(), category: "Shopping"),
        TodoItem(title: "Morning run", notes: "5km at the park", isCompleted: true, priority: .medium, createdAt: Date(), category: "Health"),
        TodoItem(title: "Read Swift docs", notes: "Dynamic replacement chapter", isCompleted: false, priority: .low, createdAt: Date(), category: "Learning"),
        TodoItem(title: "Fix UI bugs", notes: "Navigation and layout issues", isCompleted: false, priority: .high, createdAt: Date(), category: "Work"),
        TodoItem(title: "Plan weekend trip", notes: "Research destinations", isCompleted: false, priority: .low, createdAt: Date(), category: "Personal"),
        TodoItem(title: "Update resume", notes: "Add recent projects", isCompleted: false, priority: .medium, createdAt: Date(), category: "Personal"),
    ]
}
