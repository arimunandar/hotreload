import SwiftUI

struct TodoFilterBar: View {
    @ObservedObject var store: TodoStore

    var body: some View {
        VStack(spacing: 12) {
            // Filter pills
            HStack(spacing: 6) {
                ForEach(TodoFilter.allCases, id: \.self) { filter in
                    Button {
                        withAnimation(.spring(response: 0.3)) { store.filter = filter }
                    } label: {
                        Text(filter.label)
                            .font(.subheadline.weight(.medium))
                            .padding(.horizontal, 16)
                            .padding(.vertical, 8)
                            .background(store.filter == filter ? Color.purple : Color(.systemGray6))
                            .foregroundStyle(store.filter == filter ? .white : .secondary)
                            .clipShape(Capsule())
                    }
                    .buttonStyle(.plain)
                }
                Spacer()
            }

            // Category chips
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    CategoryChip(name: "All", isSelected: store.selectedCategory == nil) {
                        withAnimation(.spring(response: 0.3)) { store.selectedCategory = nil }
                    }
                    ForEach(TodoItem.categories, id: \.self) { cat in
                        CategoryChip(name: cat, isSelected: store.selectedCategory == cat) {
                            withAnimation(.spring(response: 0.3)) { store.selectedCategory = cat }
                        }
                    }
                }
            }
        }
    }
}

private struct CategoryChip: View {
    let name: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(name)
                .font(.caption.weight(.medium))
                .padding(.horizontal, 14)
                .padding(.vertical, 7)
                .background(isSelected ? Color.purple.opacity(0.15) : Color(.systemGray6))
                .foregroundStyle(isSelected ? .purple : .secondary)
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}
