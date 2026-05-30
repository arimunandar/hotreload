import SwiftUI

class TodoStore: ObservableObject {
    @Published var items: [TodoItem] = TodoItem.samples
    @Published var filter: TodoFilter = .all
    @Published var searchText: String = ""
    @Published var selectedCategory: String? = nil
    
    var filteredItems: [TodoItem] {
        var result = items
        switch filter {
        case .all: break
        case .active: result = result.filter { !$0.isCompleted }
        case .completed: result = result.filter { $0.isCompleted }
        }
        if !searchText.isEmpty {
            result = result.filter { $0.title.localizedCaseInsensitiveContains(searchText) || $0.notes.localizedCaseInsensitiveContains(searchText) }
        }
        if let cat = selectedCategory {
            result = result.filter { $0.category == cat }
        }
        return result
    }
    
    var completedCount: Int { items.filter(\.isCompleted).count }
    var activeCount: Int { items.filter { !$0.isCompleted }.count }
    var completionRate: Double { items.isEmpty ? 0 : Double(completedCount) / Double(items.count) }
    
    func toggle(_ item: TodoItem) {
        if let idx = items.firstIndex(where: { $0.id == item.id }) {
            items[idx].isCompleted.toggle()
        }
    }
    
    func delete(_ item: TodoItem) {
        items.removeAll { $0.id == item.id }
    }
    
    func add(title: String, notes: String, priority: TodoPriority, category: String) {
        items.insert(TodoItem(title: title, notes: notes, isCompleted: false, priority: priority, createdAt: Date(), category: category), at: 0)
    }
}
