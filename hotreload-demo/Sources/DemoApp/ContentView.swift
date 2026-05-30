import SwiftUI
import HotReloadKit

struct ContentView: View {
    @ObserveInjection var redraw
    @StateObject private var store = TodoStore()
    @State private var showAddSheet = false

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 20) {
                    GreetingBanner(store: store)
                        .padding(.horizontal, 16)

                    TodoStatsView(store: store)
                        .padding(.horizontal, 16)

                    TodoFilterBar(store: store)
                        .padding(.horizontal, 16)

                    if store.filteredItems.isEmpty {
                        TodoEmptyView(filter: store.filter, onAdd: { showAddSheet = true })
                            .padding(.horizontal, 16)
                    } else {
                        LazyVStack(spacing: 12) {
                            ForEach(store.filteredItems) { item in
                                NavigationLink(destination: TodoDetailView(store: store, item: item)) {
                                    TodoRowView(
                                        item: item,
                                        onToggle: { withAnimation(.spring(response: 0.3)) { store.toggle(item) } },
                                        onDelete: { withAnimation { store.delete(item) } }
                                    )
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.horizontal, 16)
                    }
                }
                .padding(.vertical, 16)
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle("Todos")
            .searchable(text: $store.searchText, prompt: "Search todos...")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button { showAddSheet = true } label: {
                        Image(systemName: "plus.circle.fill")
                            .font(.title3)
                            .foregroundStyle(.purple)
                    }
                }
            }
            .sheet(isPresented: $showAddSheet) {
                AddTodoSheet(store: store)
            }
        }
        .id(redraw as? UInt64 ?? 0)
    }
}
